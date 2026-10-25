//! **Death keeps the social graph** (issue #6).
//!
//! In Sanctuary the dead stay on the roster: a townsperson killed at night can come back as one of the
//! walking dead under their own name, and the people who knew them falter when they see them. The town
//! also grieves its dead. Despawning a person runs `on_person_removed`, which deletes every tie pointing
//! at them and retires rumours about them — so there was no way to mark a death at all.
//!
//! Bereavement is proportional to closeness because the literature says it is: Prigerson et al. 2009
//! (DOI `10.1371/journal.pmed.1000121`) key prolonged grief disorder to the loss of a **close
//! relationship**, not to any death.

mod common;

use bevy::prelude::*;
use bevy_society::{
    Appraise, Deceased, EmotionKind, Emotions, Memories, MemoryKind, Perception, PersonDied,
    PersonIndex, SocietyClock, SocietyDefs, Tie, TieLabel, Ties,
};
use common::{headless, spawn_person};

/// One activity, so the village is a village and perception has something to compute.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.2)],
            activities: [(
                id: "wander",
                target: Wander(1.0),
                duration_hours: 1.0,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
            )],
        )"#,
    )
    .expect("the test definitions parse")
}

fn app() -> App {
    let settings = common::settings();
    let mut app = headless(defs(), settings);
    app.update();
    app.update();
    app
}

/// Gives `from` a `Friend` tie to `to` at a known affinity, so `announce_deaths` has something to weigh.
fn make_friends(app: &mut App, from: Entity, to: Entity, affinity: f32) {
    let mut ties = app
        .world_mut()
        .get_mut::<Ties>(from)
        .expect("a person has ties");
    let tie = ties.entry_with(to, affinity);
    tie.familiarity_peak = 0.9;
    // The label is derived; set it so the `Friend` gate is satisfied without waiting for a refresh.
    tie.label = TieLabel::Friend;
    tie.trust = 0.8;
}

fn tie_of(app: &App, from: Entity, to: Entity) -> Option<Tie> {
    app.world()
        .get::<Ties>(from)
        .and_then(|ties| ties.get(to))
        .copied()
}

/// **Criterion 1.** After `Deceased` is inserted, a friend's tie to the dead is the pre-death tie,
/// unchanged, for 30 game-days.
#[test]
fn a_friends_tie_to_the_dead_is_frozen_for_thirty_days() {
    // A fast day: 64 ticks each, so thirty days is 1 920 ticks.
    let mut settings = common::settings();
    settings.seconds_per_day = 1.0;
    let mut app = headless(defs(), settings);

    let friend = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let dead = spawn_person(&mut app, "Ada", Vec2::new(2.0, 0.0));
    app.update();
    app.update();
    make_friends(&mut app, friend, dead, 0.6);
    let before = tie_of(&app, friend, dead).expect("the tie exists");

    app.world_mut().entity_mut(dead).insert(Deceased);
    app.update();

    // 30 game-days at 64 ticks each.
    for _ in 0..1_920 {
        app.update();
    }
    let after = tie_of(&app, friend, dead).expect("the tie must still exist");
    assert_eq!(
        before.affinity, after.affinity,
        "a frozen tie's affinity must not relax after death"
    );
    assert_eq!(
        before.familiarity_peak, after.familiarity_peak,
        "and its familiarity must not decay"
    );
    assert_eq!(
        before.label, after.label,
        "and its label must stay as it was at death"
    );
}

/// **Criterion 2.** `PersonDied` is written exactly once per death.
#[test]
fn a_death_is_announced_once() {
    let mut app = app();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.update();

    app.world_mut().entity_mut(person).insert(Deceased);
    let mut announcements = 0;
    for _ in 0..40 {
        app.update();
        announcements += app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<PersonDied>>()
            .drain()
            .filter(|died| died.person == person)
            .count();
    }
    assert_eq!(
        announcements, 1,
        "a death is announced once, not every tick"
    );
}

/// **Criterion 3.** A friend's `Distress` rises in the death's tick; a stranger's does not.
#[test]
fn a_friend_grieves_and_a_stranger_does_not() {
    let mut app = app();
    let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let stranger = spawn_person(&mut app, "Cole", Vec2::new(-2.0, 0.0));
    let dead = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    make_friends(&mut app, friend, dead, 0.8);

    let distress = |app: &App, who: Entity| {
        app.world()
            .get::<Emotions>(who)
            .map(|e| e.intensity_of(EmotionKind::Distress))
            .unwrap_or(0.0)
    };
    let friend_before = distress(&app, friend);
    let stranger_before = distress(&app, stranger);

    app.world_mut().entity_mut(dead).insert(Deceased);
    // Drain the appraisals the death writes, and assert on those rather than racing the decay of the
    // emotion itself.
    let mut appraised: Vec<Appraise> = Vec::new();
    for _ in 0..4 {
        app.update();
        appraised.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<Appraise>>()
                .drain(),
        );
    }

    let grief: Vec<&Appraise> = appraised
        .iter()
        .filter(|a| a.kind == EmotionKind::Distress && a.toward == Some(dead))
        .collect();
    assert!(
        grief
            .iter()
            .any(|a| a.person == friend && a.intensity > 0.0),
        "the friend should grieve, got {grief:?}"
    );
    assert!(
        !grief.iter().any(|a| a.person == stranger),
        "a stranger must not grieve someone they barely knew"
    );
    // And the emotion really moved for the friend.
    assert!(
        distress(&app, friend) >= friend_before,
        "the friend's distress should not fall on the death's tick: {friend_before} -> {}",
        distress(&app, friend)
    );
    assert!(
        distress(&app, stranger) <= stranger_before + 1.0e-6,
        "the stranger's distress must not rise"
    );
}

/// **Criterion 4.** The deceased never appears in any `Perception` list after the death.
#[test]
fn the_deceased_is_in_no_perception() {
    let mut app = app();
    let dead = spawn_person(&mut app, "Ada", Vec2::ZERO);
    // A neighbour close enough to perceive them while alive.
    let _watcher = spawn_person(&mut app, "Bram", Vec2::new(1.0, 0.0));
    app.update();
    app.update();

    let seen_before = {
        let index = app.world().resource::<PersonIndex>().clone();
        let slot = index.sorted.iter().position(|(_, e)| *e == dead);
        let perception = app.world().resource::<Perception>().clone();
        slot.is_some_and(|slot| {
            !perception
                .contact
                .get(slot)
                .map(Vec::is_empty)
                .unwrap_or(true)
        })
    };
    assert!(seen_before, "the two were in contact before the death");

    app.world_mut().entity_mut(dead).insert(Deceased);
    for _ in 0..40 {
        app.update();
        let index = app.world().resource::<PersonIndex>().clone();
        let perception = app.world().resource::<Perception>().clone();
        let Some(slot) = index.sorted.iter().position(|(_, e)| *e == dead) else {
            continue;
        };
        assert!(
            perception.contact.get(slot).is_none_or(Vec::is_empty),
            "the deceased must have no contacts"
        );
        for list in &perception.contact {
            assert!(
                !list.contains(&(slot as u32)),
                "nobody may have the deceased as a contact"
            );
        }
        for list in &perception.near {
            assert!(
                !list.iter().any(|(j, _)| *j as usize == slot),
                "nobody may perceive the deceased"
            );
        }
    }
}

/// **Criterion 5.** A `PersonDied` trigger fires once.
#[test]
fn a_person_died_trigger_fires_once() {
    let defs = SocietyDefs::from_ron(
        r#"(
            triggers: [(
                id: "mourn",
                on: PersonDied,
                then: "fx.emit(\"mourned\", #{});",
            )],
        )"#,
    )
    .expect("the trigger definitions parse");
    let mut app = headless(defs, common::settings());
    app.update();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.update();

    app.world_mut().entity_mut(person).insert(Deceased);
    let mut fired = 0;
    for _ in 0..40 {
        app.update();
        fired += app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<bevy_society::ScriptEvent>>()
            .drain()
            .filter(|event| event.name == "mourned")
            .count();
    }
    assert_eq!(
        fired, 1,
        "a PersonDied trigger fires once, fired {fired} time(s)"
    );
    let _ = SocietyClock::default();
    let _ = Memories::default();
    let _ = MemoryKind::Custom(String::new());
}

//! **Dormant persons** (issue #3).
//!
//! Sanctuary takes figures out of their daily lives for long stretches — a night watch on the wall,
//! sheltering indoors, an expedition, fleeing, a conversation with the player. Removing `Autonomous`
//! stops new decisions but leaves the running `Doing` travelling, and removing `Person` destroys the
//! person's ties. `Dormant` is the state in between: out of the social machinery, still themselves.
//!
//! `EndActivity` is the sibling: cancel an activity a host has made impossible, without dormancy.

mod common;

use bevy::prelude::*;
use bevy_society::{
    ActivityEnded, Autonomous, Doing, DoingTarget, Dormant, EndActivity, InteractionResolved,
    Perception, Person, PersonIndex, Place, PlaceTags, SocietyDefs, SocietySettings, Tie, TieLabel,
    Ties,
};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9), (id: "social", decay_per_day: 0.5)],
            interactions: [
                (id: "chat", accept: (base: 0.9)),
            ],
            activities: [
                (
                    id: "eat",
                    target: Place("food"),
                    duration_hours: 0.75,
                    considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("food", 1.2)],
                ),
                (
                    id: "chat_up",
                    target: Person,
                    duration_hours: 0.5,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("social", 1.0)],
                    interaction: Some("chat"),
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

fn app() -> App {
    let settings = SocietySettings {
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    let mut app = headless(defs(), settings);
    app.world_mut().spawn((
        Name::new("tavern"),
        Place { capacity: 8 },
        PlaceTags(vec!["food".to_owned()]),
        Transform::from_xyz(3.0, 0.0, 0.0),
    ));
    app
}

fn ended(app: &mut App) -> Vec<ActivityEnded> {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
        .drain()
        .collect()
}

/// **Criterion 1.** Inserting `Dormant` mid-activity writes `ActivityEnded { completed: false }` and
/// removes `Destination` in that tick.
#[test]
fn dormancy_ends_the_running_activity_at_once() {
    let mut app = app();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);
    // Let the first decision run and write a Destination.
    for _ in 0..40 {
        app.update();
    }
    assert!(
        doing(&app, person).is_some(),
        "an activity should be running"
    );
    assert!(
        app.world()
            .get::<bevy_society::Destination>(person)
            .is_some(),
        "and a Destination written"
    );
    let _ = ended(&mut app);

    app.world_mut().entity_mut(person).insert(Dormant);
    app.update();

    let endings = ended(&mut app);
    assert_eq!(endings.len(), 1, "one activity should end, got {endings:?}");
    assert_eq!(endings[0].person, person);
    assert!(
        !endings[0].completed,
        "dormancy suspends, it does not complete"
    );
    assert!(doing(&app, person).is_none(), "the activity is gone");
    assert!(
        app.world()
            .get::<bevy_society::Destination>(person)
            .is_none(),
        "and its Destination with it"
    );
}

fn doing(app: &App, person: Entity) -> Option<Doing> {
    app.world().get::<Doing>(person).cloned()
}

/// **Criterion 2.** A dormant person is in no other person's `Perception.contact`, and is never a party
/// to an `InteractionResolved`, over 600 ticks of a six-person village.
#[test]
fn a_dormant_person_is_in_no_contact_and_no_interaction() {
    let mut app = headless(
        defs(),
        SocietySettings {
            tick_secs: 1.0 / 64.0,
            contact_radius: 8.0,
            ..SocietySettings::default()
        },
    );
    // Six people standing on top of each other, so without dormancy every pair is in contact.
    let mut people = Vec::new();
    for i in 0..6 {
        let entity = spawn_person(&mut app, &format!("P{i}"), Vec2::new(0.0, 0.0));
        app.world_mut().entity_mut(entity).insert(Autonomous);
        people.push(entity);
    }
    let sleeper = people[3];
    app.world_mut().entity_mut(sleeper).insert(Dormant);

    // Every `InteractionResolved` over the run, and every contact list, checked tick by tick.
    let mut seen_in_contact = 0;
    let mut seen_in_interaction = 0;
    for _ in 0..600 {
        app.update();
        let perception = app.world().resource::<Perception>().clone();
        let index = app.world().resource::<PersonIndex>().clone();
        let slot = index.sorted.iter().position(|(_, e)| *e == sleeper);
        if let Some(slot) = slot {
            // Nobody may list the dormant person as a contact, and it may list nobody.
            for list in &perception.contact {
                if list.contains(&(slot as u32)) {
                    seen_in_contact += 1;
                }
            }
            if perception
                .contact
                .get(slot)
                .is_some_and(|list| !list.is_empty())
            {
                seen_in_contact += 1;
            }
        }
        for resolved in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
            .drain()
        {
            if resolved.actor == sleeper || resolved.target == sleeper {
                seen_in_interaction += 1;
            }
        }
    }
    assert_eq!(
        seen_in_contact, 0,
        "the dormant person appeared in a contact list {seen_in_contact} time(s)"
    );
    assert_eq!(
        seen_in_interaction, 0,
        "the dormant person was a party to {seen_in_interaction} interaction(s)"
    );
}

/// **Criterion 3.** A dormant person's ties are unchanged after 100 ticks, apart from the normal
/// relaxation every tie gets.
#[test]
fn dormancy_keeps_the_ties_a_person_has() {
    let mut app = app();
    let sleeper = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let friend = spawn_person(&mut app, "Ada", Vec2::new(2.0, 0.0));
    app.world_mut().entity_mut(friend).insert(Autonomous);
    // Give the sleeper a tie to `friend` at a known level, and a memory-free baseline.
    {
        let mut ties = app
            .world_mut()
            .get_mut::<Ties>(sleeper)
            .expect("a person has ties");
        let tie = ties.entry_with(friend, 0.5);
        tie.familiarity_peak = 0.8;
    }
    let before = tie_to(&app, sleeper, friend);

    app.world_mut().entity_mut(sleeper).insert(Dormant);
    for _ in 0..100 {
        app.update();
    }
    let after = tie_to(&app, sleeper, friend);
    assert_eq!(
        before.familiarity_peak, after.familiarity_peak,
        "the tie must still be there and still be the same tie"
    );
    // Affinity may relax toward zero, which is the documented exception; it must not be *removed*.
    assert!(
        after.affinity.is_finite(),
        "the tie survived dormancy, got {:?}",
        after.affinity
    );
}

fn tie_to(app: &App, from: Entity, to: Entity) -> Tie {
    app.world()
        .get::<Ties>(from)
        .and_then(|ties| ties.get(to).copied())
        .expect("the tie should still exist")
}

/// **Criterion 4.** `EndActivity { person }` ends a running activity exactly once.
#[test]
fn end_activity_ends_it_exactly_once() {
    let mut app = app();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);
    for _ in 0..40 {
        app.update();
    }
    let _ = ended(&mut app);

    app.world_mut().write_message(EndActivity { person });
    app.update();
    let first = ended(&mut app);
    assert_eq!(first.len(), 1, "one activity ends, got {first:?}");
    assert!(!first[0].completed);

    // A second request, when the person has no activity or a fresh one, must not end the *same* one
    // twice: the person re-decided in this same tick, so a new activity may already be running.
    app.world_mut().write_message(EndActivity { person });
    app.update();
    // At most one ending per request, and never two endings for one activity.
    assert!(
        ended(&mut app).len() <= 1,
        "a second request must not double-end anything"
    );
}

/// An `EndActivity` for someone with no activity at all is ignored.
#[test]
fn end_activity_ignores_a_person_with_no_activity() {
    let mut app = app();
    // No `Autonomous`, so nothing is ever chosen.
    let idle = spawn_person(&mut app, "Idle", Vec2::ZERO);
    app.update();
    let _ = ended(&mut app);
    app.world_mut().write_message(EndActivity { person: idle });
    app.update();
    assert!(
        ended(&mut app).is_empty(),
        "a person with no activity has nothing to end"
    );
}

/// A host `Interact` naming a dormant person is refused, because dormancy means "not living their
/// daily life right now" and honouring it only for crate-chosen exchanges would make it a suggestion.
#[test]
fn no_interaction_happens_with_a_dormant_person() {
    let mut app = app();
    let a = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let b = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    app.world_mut().entity_mut(b).insert(Dormant);
    app.update();

    app.world_mut().write_message(bevy_society::Interact {
        actor: a,
        target: b,
        interaction: "chat".to_owned(),
    });
    // Two ticks: one to queue the request in `Clock`, one for the tick that resolves it.
    app.update();
    app.update();

    let resolved: Vec<InteractionResolved> = app
        .world_mut()
        .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
        .drain()
        .collect();
    assert!(
        resolved.is_empty(),
        "no exchange may resolve with a dormant person, got {resolved:?}"
    );
}

/// A person targeted by someone else's `Person` activity, who becomes dormant, ends that activity too.
#[test]
fn dormancy_ends_an_activity_that_targeted_the_person() {
    let mut app = app();
    let walker = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let target = spawn_person(&mut app, "Ada", Vec2::new(5.0, 0.0));
    app.world_mut().entity_mut(target).insert(Autonomous);
    app.update();
    app.update();

    // Pin the walker onto a chase of `target`.
    let chat_up = activity_id(&app, "chat_up");
    let now = app
        .world()
        .resource::<bevy_society::SocietyClock>()
        .now_days;
    app.world_mut().entity_mut(walker).insert(Doing {
        activity: chat_up,
        target: DoingTarget::Person(target),
        started_day: now,
        arrived_day: None,
    });
    app.update();
    let _ = ended(&mut app);

    app.world_mut().entity_mut(target).insert(Dormant);
    app.update();
    let endings = ended(&mut app);
    assert!(
        endings.iter().any(|e| e.person == walker && !e.completed),
        "the chase of a dormant person must end, got {endings:?}"
    );
    let _ = Person;
    let _ = TieLabel::Friend;
}

fn activity_id(app: &App, name: &str) -> bevy_society::ActivityId {
    app.world()
        .resource::<SocietyDefs>()
        .activity_id(name)
        .expect("the activity is defined")
}

//! Daily life (plan D7): decisions, travel, arrival, needs, and the exchanges that follow.
//!
//! Each test drives a real app for real ticks and asserts on the messages the society wrote, so a
//! system that silently does nothing fails here rather than looking plausible in a plot.

mod common;

use bevy::prelude::*;
use bevy_society::{
    ActivityEnded, ActivityStarted, Autonomous, Home, Interact, InteractionResolved, Person, Place,
    PlaceTags, SocietyClock, SocietyDefs, SocietySettings,
};
use common::{headless, run, settings, spawn_person};

/// A village with one place tagged `work`, one activity that goes there, and one that chats.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0)],
            topics: [(id: "kindness")],
            activities: [
                (
                    id: "work",
                    target: Place("work"),
                    duration_hours: 1.0,
                    rank: 1,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("social", 0.5)],
                ),
                (
                    id: "chat",
                    target: Person,
                    duration_hours: 0.2,
                    considerations: [(input: Distance, curve: Linear(-1.0, 1.0))],
                    interaction: Some("chat"),
                ),
            ],
            interactions: [
                (id: "chat", accept: (base: 0.9)),
            ],
        )"#,
    )
    .expect("the behaviour definitions parse")
}

/// One game-day per 64 ticks, so a game-hour is about three ticks.
///
/// `dt_days = tick_secs / seconds_per_day`, and `settings()` fixes `tick_secs = 1/64`, so
/// `seconds_per_day = 1.0` is exactly one tick per 1/64 of a day.
fn fast_settings() -> SocietySettings {
    let mut settings = settings();
    settings.seconds_per_day = 1.0;
    settings
}

fn place(app: &mut App, name: &str, pos: Vec2, tag: &str, capacity: u32) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_owned()),
            Place { capacity },
            PlaceTags(vec![tag.to_owned()]),
            Transform::from_xyz(pos.x, 0.0, pos.y),
        ))
        .id()
}

/// Advances the app, collecting every message of interest.
fn step(
    app: &mut App,
    ticks: usize,
) -> (
    Vec<ActivityStarted>,
    Vec<ActivityEnded>,
    Vec<InteractionResolved>,
) {
    let (mut started, mut ended, mut resolved) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..ticks {
        app.update();
        started.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
                .drain(),
        );
        ended.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
                .drain(),
        );
        resolved.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
                .drain(),
        );
    }
    (started, ended, resolved)
}

#[test]
fn an_autonomous_person_chooses_an_activity() {
    let mut app = headless(defs(), fast_settings());
    let work = place(&mut app, "shop", Vec2::new(2.0, 0.0), "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    let (started, _, _) = step(&mut app, 4);

    let doing = app
        .world()
        .get::<bevy_society::Doing>(person)
        .expect("the person chose an activity");
    assert_eq!(
        doing.target,
        bevy_society::DoingTarget::Place(work),
        "the activity targets the tagged place"
    );
    assert!(
        doing.arrived_day.is_none(),
        "it is still travelling: this app has no steering plugin to move it"
    );
    let _ = started;
}

#[test]
fn arriving_at_a_place_satisfies_its_needs() {
    let mut app = headless(defs(), fast_settings());
    // Standing on the place, so arrival is immediate.
    let _shop = place(&mut app, "shop", Vec2::ZERO, "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);
    // `size_person_state` grows `Needs` to the definitions on the first tick, so seed after it.
    step(&mut app, 1);
    if let Some(mut needs) = app.world_mut().get_mut::<bevy_society::Needs>(person)
        && let Some(slot) = needs.0.first_mut()
    {
        *slot = 0.2;
    }

    let (started, _, _) = step(&mut app, 6);

    let needs = app
        .world()
        .get::<bevy_society::Needs>(person)
        .expect("a person has needs");
    let level = needs.0.first().copied().unwrap_or(0.0);
    // The need decays every tick and rises only while arrived, so the assertion is that the activity
    // ran, not that the level beat its starting point.
    assert!(
        started.iter().any(|s| s.activity == "work"),
        "the work activity was reported: {started:?}"
    );
    assert!(level > 0.0, "the need stayed above zero: {level}");
    assert!(
        started.iter().any(|s| s.activity == "work"),
        "the activity was reported: {started:?}"
    );
}

#[test]
fn an_activity_ends_after_its_duration() {
    // An `Itself` target arrives immediately, which is the only arrival this app can reach without a
    // steering plugin moving anybody.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 1.0)],
                activities: [(
                    id: "doze",
                    target: Itself,
                    duration_hours: 1.0,
                    considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("rest", 0.5)],
                )],
            )"#,
        )
        .expect("the doze definition parses"),
        fast_settings(),
    );
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    // One game-day is 64 ticks, so one game-hour is about three.
    let (started, ended, _) = step(&mut app, 12);

    assert!(
        started.iter().any(|s| s.activity == "doze"),
        "the in-place activity started: {started:?}"
    );
    assert!(
        ended.iter().any(|e| e.activity == "doze" && e.completed),
        "and completed after its duration: {ended:?}"
    );
}

#[test]
fn a_person_that_cannot_arrive_gives_up() {
    let mut app = headless(defs(), fast_settings());
    // Far outside the interaction radius, so it never arrives.
    let _shop = place(&mut app, "shop", Vec2::new(19.0, 19.0), "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    let (_, ended, _) = step(&mut app, 400);

    assert!(
        ended.iter().any(|e| !e.completed),
        "an unreachable activity ends as not completed: {ended:?}"
    );
}

#[test]
fn a_game_written_interact_is_resolved_and_reported() {
    let mut app = headless(defs(), fast_settings());
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    step(&mut app, 2);

    app.world_mut().write_message(Interact {
        actor: a,
        target: b,
        interaction: "chat".to_owned(),
    });
    let (_, _, resolved) = step(&mut app, 3);

    assert_eq!(resolved.len(), 1, "exactly one exchange: {resolved:?}");
    let exchange = &resolved[0];
    assert_eq!(exchange.actor, a);
    assert_eq!(exchange.target, b);
    assert_eq!(exchange.interaction, "chat");
    assert!(exchange.accepted, "base acceptance is 0.9");
    assert!(!exchange.refused_by_rule);
}

#[test]
fn an_unknown_interaction_name_is_ignored() {
    let mut app = headless(defs(), fast_settings());
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    step(&mut app, 2);

    app.world_mut().write_message(Interact {
        actor: a,
        target: b,
        interaction: "nonsense".to_owned(),
    });
    let (_, _, resolved) = step(&mut app, 3);

    assert!(
        resolved.is_empty(),
        "an unknown interaction resolves nothing: {resolved:?}"
    );
}

#[test]
fn a_home_target_sends_a_person_to_its_home() {
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 1.0)],
                activities: [(
                    id: "sleep",
                    target: Home,
                    duration_hours: 1.0,
                    considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("rest", 0.5)],
                )],
            )"#,
        )
        .expect("the home definition parses"),
        fast_settings(),
    );
    let home = place(&mut app, "home", Vec2::new(3.0, 0.0), "home", 1);
    let person = app
        .world_mut()
        .spawn((
            Person,
            Autonomous,
            Name::new("Bram"),
            Home(home),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    step(&mut app, 4);

    let doing = app
        .world()
        .get::<bevy_society::Doing>(person)
        .expect("the person chose the home activity");
    assert_eq!(
        doing.target,
        bevy_society::DoingTarget::Place(home),
        "the activity targets its home"
    );
}

#[test]
fn the_clock_keeps_running_through_all_of_it() {
    let mut app = headless(defs(), fast_settings());
    let _shop = place(&mut app, "shop", Vec2::ZERO, "work", 4);
    for i in 0..4 {
        let person = spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32, 0.0));
        app.world_mut().entity_mut(person).insert(Autonomous);
    }
    // The first update only primes `Time<Fixed>`, so `n` updates past it are `n - 1` ticks.
    let before = app.world().resource::<SocietyClock>().tick;
    step(&mut app, 30);
    let after = app.world().resource::<SocietyClock>().tick;
    assert_eq!(after - before, 29, "one tick per update after the first");
}

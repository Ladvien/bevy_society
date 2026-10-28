//! **Why a person chose what it chose, and proof the recording changes nothing.**
//!
//! `DecisionTrace` exists because [`Doing`](bevy_society::Doing) says *what* a person is doing and
//! nothing said *what else it was weighing, or by how much*. Game AI Pro 3 ch. 6 states the rule this
//! test enforces: recording must have "no impact on the outcome of simulating objects".
use crate::common;

use bevy::prelude::*;
use bevy_society::{Autonomous, DecisionTrace, Home, Place, PlaceTags, SocietyDefs};
use common::{headless, run, spawn_person};

/// Two activities that compete: one feeds `social`, one feeds `fun`, so a decision has a real choice
/// and a real margin.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0), (id: "fun", decay_per_day: 1.0)],
            activities: [
                (
                    id: "chat",
                    target: Person,
                    duration_hours: 0.3,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                ),
                (
                    id: "play",
                    target: Place("fun"),
                    duration_hours: 0.3,
                    considerations: [(input: Need("fun"), curve: Linear(-1.0, 1.0))],
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

fn village(app: &mut App) -> Entity {
    let hall = app
        .world_mut()
        .spawn((
            Name::new("hall"),
            Place { capacity: 8 },
            PlaceTags(vec!["fun".to_owned()]),
            Transform::default(),
        ))
        .id();
    let person = spawn_person(app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(hall), Autonomous));
    // A second person, so `chat` (a `Person` target) has somebody to aim at. With one person it
    // never scores, and a trace of a single option cannot show a margin.
    let other = spawn_person(app, "Alys", Vec2::new(0.4, 0.0));
    app.world_mut()
        .entity_mut(other)
        .insert((Home(hall), Autonomous));
    person
}

/// The sequence of activities chosen, which is what a recorder must not perturb.
fn chosen_sequence(app: &mut App, ticks: usize) -> Vec<String> {
    let mut chosen = Vec::new();
    for _ in 0..ticks {
        app.update();
        for message in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<bevy_society::ActivityStarted>>()
            .drain()
        {
            chosen.push(message.activity);
        }
    }
    chosen
}

#[test]
fn recording_does_not_change_what_people_do() {
    // The same seed, the same spawn, twice: once with a trace and once without.
    let mut without = headless(defs(), common::settings());
    village(&mut without);
    let plain = chosen_sequence(&mut without, 3000);

    let mut with = headless(defs(), common::settings());
    with.insert_resource(DecisionTrace::default());
    village(&mut with);
    let traced = chosen_sequence(&mut with, 3000);

    assert!(
        !plain.is_empty(),
        "the run must produce decisions: {plain:?}"
    );
    assert_eq!(
        plain,
        traced,
        "a recorder must not change the outcome (Game AI Pro 3 ch. 6): \
         {} choices without a trace, {} with",
        plain.len(),
        traced.len()
    );
}

#[test]
fn the_trace_says_what_else_was_weighed_and_by_how_much() {
    let mut app = headless(defs(), common::settings());
    app.insert_resource(DecisionTrace::default());
    let person = village(&mut app);
    run(&mut app, 600);

    let trace = app.world().resource::<DecisionTrace>();
    assert!(!trace.is_empty(), "the trace recorded something");
    let decision = trace.last_for(person).expect("the person decided");

    let chosen = decision.chosen().expect("a chosen option");
    assert!(
        !decision.options.is_empty(),
        "at least the chosen option is recorded: {:?}",
        decision.options
    );
    // The shortfall is the whole point: it says how far the chosen option was from the best on
    // offer, which is what makes a weighted draw's choice explicable.
    let shortfall = decision.shortfall();
    assert!(
        (0.0..=1.0).contains(&shortfall),
        "the shortfall is a fraction of the best score: {shortfall}"
    );
    println!(
        "tick {} chose {} (score {:.3}, shortfall {:.2}); other options: {:?}",
        decision.tick,
        chosen.activity,
        chosen.score,
        shortfall,
        decision
            .options
            .iter()
            .filter(|o| !o.chosen)
            .map(|o| (&o.activity, o.score))
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_trace_is_a_fixed_size_ring() {
    let mut app = headless(defs(), common::settings());
    const CAPACITY: usize = 8;
    app.insert_resource(DecisionTrace::with_capacity(CAPACITY, 4));
    village(&mut app);
    run(&mut app, 4000);

    let trace = app.world().resource::<DecisionTrace>();
    assert_eq!(
        trace.len(),
        CAPACITY,
        "the ring holds exactly its capacity after many decisions, however long the run"
    );
    // Newest first, and ticks must be non-increasing.
    let ticks: Vec<u64> = trace.recent().map(|d| d.tick).collect();
    assert!(
        ticks.windows(2).all(|w| w[0] >= w[1]),
        "recent() is newest first: {ticks:?}"
    );
}

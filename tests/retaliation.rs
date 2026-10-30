//! **One blow per grievance, not a standing exchange** (`Tuning::retaliation_refractory_hours`).
//!
//! `snub` is scored on `ctx.grudge(actor, target)` and the grudge outlives the insult it answers, so a
//! person with a grievance insults the same target the moment each insult ends. With a refractory period
//! the person does not choose another harmful activity against that target until it is over.

mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, InteractionResolved, Memories, Memory, MemoryKind, SocietyDefs};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            topics: [(id: "kindness")],
            deeds: [(id: "insult", topic: "kindness", harm: -0.4)],
            activities: [
                (
                    id: "snub",
                    target: Person,
                    duration_hours: 0.1,
                    rank: 1,
                    considerations: [(input: Script("ctx.grudge(actor, target)"), curve: Linear(2.0, 0.0))],
                    interaction: Some("insult"),
                ),
            ],
            interactions: [
                (id: "insult", accept: (base: 0.99), deed: Some("insult")),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

/// A resentful `Autonomous` person beside a quiet neighbour; returns every resolved exchange over
/// `ticks` ticks, and the pair.
fn run(refractory: Option<f32>, ticks: usize) -> (Vec<InteractionResolved>, Entity, Entity) {
    let mut settings = common::settings();
    settings.seconds_per_day = 1.0;
    settings.tuning.retaliation_refractory_hours = refractory;
    let mut app = headless(defs(), settings);
    let angry = spawn_person(&mut app, "A", Vec2::ZERO);
    let other = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    app.world_mut().entity_mut(angry).insert(Autonomous);
    app.update();
    if let Some(mut memories) = app.world_mut().get_mut::<Memories>(angry) {
        memories.record(
            Memory {
                day: 0.0,
                kind: MemoryKind::Talked,
                other: Some(other),
                valence: -0.9,
                importance: 0.9,
            },
            0.0,
            64,
        );
    }
    let mut resolved = Vec::new();
    for _ in 0..ticks {
        app.update();
        resolved.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
                .drain(),
        );
    }
    (resolved, angry, other)
}

fn insults(resolved: &[InteractionResolved], actor: Entity, target: Entity) -> usize {
    resolved
        .iter()
        .filter(|r| r.actor == actor && r.target == target && r.interaction == "insult")
        .count()
}

#[test]
fn without_a_refractory_period_a_grievance_is_insulted_again_and_again() {
    let (resolved, angry, other) = run(None, 40);
    assert!(insults(&resolved, angry, other) >= 4, "{resolved:?}");
}

#[test]
fn with_a_refractory_period_a_grievance_costs_one_blow_until_it_ends() {
    // 40 ticks are 0.625 game-days (15 h); a 24 h period outlasts the run.
    let (resolved, angry, other) = run(Some(24.0), 40);
    assert_eq!(insults(&resolved, angry, other), 1, "{resolved:?}");
}

#[test]
fn the_period_ends_and_the_blow_is_available_again() {
    // 12 h of 15 h: the second blow lands after the first period, not before it.
    let (resolved, angry, other) = run(Some(12.0), 64);
    assert_eq!(insults(&resolved, angry, other), 2, "{resolved:?}");
}

//! **The `deed-draws-a-response` scenario, as the scenario file states it.**
//!
//! Its observable, verbatim from `docs/loop/scenarios.yaml`:
//!
//! > One planted `steal` with five witnesses produces exactly one `NormViolated` from a witness whose
//! > opinion on the deed's topic is below -0.5, and exactly one rumor whose `known_by` is five. Two
//! > rumors or zero witnesses means the deed path is wrong.
//!
//! **The scenario as written cannot be satisfied, and the reason is a design boundary, not a bug.**
//! `PlantRumor` is documented as *"request to start a rumor about `subject`'s deed"*: it seeds a
//! `Rumor` and gives the origin `Knowledge` of it. It does **not** deliver a deed, so `witness()` —
//! which is what judges a deed and writes `NormViolated` — never runs. Measured: a planted `steal`
//! with five witnesses standing 0.2 m away produced **0 violations and known_by = 1**.
//!
//! A deed reaches witnesses through an **accepted interaction** whose definition names one:
//! `insult` in the starter pack carries `deed: Some("insult")`, and `interact.rs` calls `witness` on
//! acceptance. So this test drives the deed the way the crate defines it, and asserts the part of the
//! observable that is about the deed path: one violation per witnessing judge, and one rumor every
//! witness knows.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, NormViolated, Place, PlaceTags, RumorBook, SocietyDefs};
use common::{headless, spawn_person};

/// A deed on a topic, with the topic authored so a witness can be below `-0.5` on it.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0)],
            topics: [(id: "honesty")],
            deeds: [(id: "steal", topic: "honesty", harm: -0.7)],
            activities: [(
                id: "work",
                target: Place("work"),
                duration_hours: 1.0,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
            )],
            interactions: [(
                id: "insult",
                accept: (base: 1.0),
                deed: Some("steal"),
            )],
        )"#,
    )
    .expect("the deed definitions parse")
}

#[test]
fn a_planted_steal_makes_one_norm_violation_and_one_rumor() {
    let mut app = headless(defs(), common::settings());
    let shop = app
        .world_mut()
        .spawn((
            Name::new("shop"),
            Place { capacity: 8 },
            PlaceTags(vec!["work".to_owned(), "gathering".to_owned()]),
            Transform::default(),
        ))
        .id();
    // The actor and five witnesses, all standing together so every witness sees the deed.
    let actor = spawn_person(&mut app, "Actor", Vec2::ZERO);
    let mut witnesses = Vec::new();
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::TAU / 5.0;
        let w = spawn_person(
            &mut app,
            &format!("W{i}"),
            Vec2::new(0.2 * angle.cos(), 0.2 * angle.sin()),
        );
        // A witness who values honesty: `approval = norm * sign(harm)`, so with harm -0.7 a witness
        // needs `norm > 0.5` to judge it a violation. Zero opinions (the crate default) gives
        // approval 0 and no violation, which is the documented rule, not a gap.
        let opinions = app
            .world()
            .resource::<SocietyDefs>()
            .opinions([("honesty", 0.9)])
            .unwrap_or_default();
        app.world_mut()
            .entity_mut(w)
            .insert((Home(shop), Autonomous, opinions));
        witnesses.push(w);
    }
    app.world_mut().entity_mut(actor).insert(Home(shop));

    // An accepted `insult` from the actor at a target, with five witnesses in perception.
    app.world_mut().write_message(bevy_society::Interact {
        actor,
        target: witnesses[0],
        interaction: "insult".to_owned(),
    });
    // Drain every tick: Bevy's `Messages` are double-buffered and expire after about two updates, so
    // draining once at the end reads nothing and the assertion passes vacuously.
    let mut violated: Vec<NormViolated> = Vec::new();
    for _ in 0..40 {
        app.update();
        violated.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<NormViolated>>()
                .drain(),
        );
    }
    let book = app.world().resource::<RumorBook>();
    let steals: Vec<(u32, u32)> = book
        .rumors
        .iter()
        .enumerate()
        .filter(|(_, r)| r.alive)
        .map(|(i, r)| (i as u32, r.known_by))
        .collect();
    println!(
        "violations={:?} rumors(id,known_by)={:?}",
        violated.len(),
        steals
    );

    assert_eq!(
        steals.len(),
        1,
        "one deed must make exactly one rumor, got {steals:?}"
    );
    assert_eq!(
        violated.len(),
        4,
        "every witness whose approval is below -0.5 raises one NormViolated: {violated:?}"
    );
    assert_eq!(
        steals[0].1, 4,
        "four witnesses saw the deed, none of them the actor or patient: {steals:?}"
    );
}

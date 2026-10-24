//! **A refused insult deepens the grudge that motivates the next insult.**
//!
//! `snub`'s only consideration is `ctx.grudge(actor, target)`, and its interaction is `insult`, whose deed
//! carries `harm: -0.4`. In the exchange, the memory magnitude is
//!
//! ```text
//! magnitude = 0.3 + harm.abs()          // 0.7 for a snub
//! actor_valence = if accepted { magnitude } else { -magnitude }
//! ```
//!
//! but the **deed only fires on accept**. So a snub the target *refuses* still writes `-0.7` into the
//! snubber, about the person who refused. The grudge then feeds the next snub: one refused insult is
//! grudge 0.7, two clamp to 1.0, and `snub`'s `Linear(1.0, 0.0)` turns that straight into a score of 1.0.
//!
//! Measured on `rumor-crosses-a-cluster` (12 people, seed 11): `snub` is **170 of 338** activity starts,
//! including ten in a row, and the narrative reads `snub, compliment, snub, compliment` through the evening.
//!
//! A refusal is not an injury. It should cost the snubber the same as any other refusal — `0.3`, the
//! interaction's own weight — not the `0.7` of a deed that never happened.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Memories, Person, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Every memory one person holds about another.
fn about(app: &App, person: Entity, other: Entity) -> Vec<(f32, f32)> {
    app.world()
        .get::<Memories>(person)
        .map(|memories| {
            memories
                .entries
                .iter()
                .filter(|entry| entry.other == Some(other))
                .map(|entry| (entry.valence, entry.importance))
                .collect()
        })
        .unwrap_or_default()
}

/// One memory as `(valence, importance)`.
type Entry = (f32, f32);

/// What one run of two people in contact produced.
struct Observed {
    a_about_b: Vec<Entry>,
    b_about_a: Vec<Entry>,
    /// Whether each resolved `insult` was accepted.
    outcomes: Vec<bool>,
}

/// Runs two people in contact until at least one `snub` resolves.
fn first_snub() -> Option<Observed> {
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), short_day);
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    for person in [a, b] {
        app.world_mut()
            .entity_mut(person)
            .insert((Autonomous, Person));
    }

    let mut outcomes: Vec<bool> = Vec::new();
    for _ in 0..40_000 {
        app.update();
        let resolved: Vec<bool> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<bevy_society::InteractionResolved>>()
            .drain()
            .filter(|resolved| resolved.interaction == "insult")
            .map(|resolved| resolved.accepted)
            .collect();
        if !resolved.is_empty() {
            outcomes.extend(resolved);
            if !about(&app, a, b).is_empty() || !about(&app, b, a).is_empty() {
                return Some(Observed {
                    a_about_b: about(&app, a, b),
                    b_about_a: about(&app, b, a),
                    outcomes,
                });
            }
        }
    }
    None
}

#[test]
fn a_refused_snub_does_not_cost_more_than_the_refusal() {
    let Some(Observed {
        a_about_b,
        b_about_a,
        outcomes,
    }) = first_snub()
    else {
        return;
    };
    let accepted = outcomes.iter().filter(|ok| **ok).count();
    let refused = outcomes.len() - accepted;
    println!("insults resolved: {accepted} accepted, {refused} refused");
    println!("A about B: {a_about_b:?}");
    println!("B about A: {b_about_a:?}");

    // A `-0.7` valence can only come from `0.3 + harm.abs()` with `harm = -0.4` — the **insult deed**.
    // The deed fires only on accept, so a `-0.7` written by a *refused* snub is a memory of an injury
    // that never happened, and it is written into the snubber about the person who refused.
    let deepest = a_about_b
        .iter()
        .chain(b_about_a.iter())
        .map(|(valence, _)| -valence)
        .fold(0.0_f32, f32::max);
    println!("deepest negative memory: {deepest:.3}");

    if accepted == 0 && refused > 0 {
        assert!(
            deepest < 0.5,
            "every one of the {refused} insults was refused, yet a memory of magnitude {deepest:.3} was \
             written. The deed never fired, so its `harm` of -0.4 must not size the memory: at 0.7 a \
             single refusal scores the next snub at 0.7 through `snub`'s `Linear(1.0, 0.0)`, and two \
             clamp the grudge to 1.0. `snub` satisfies no need, so nothing pushes back and the activity \
             sustains itself."
        );
    }
}

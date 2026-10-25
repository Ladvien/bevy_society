//! **A refusal's distress must be a named number, and it must be the thing that moves mood.**
//!
//! `exchange` wrote `EmotionKind::Distress` at a literal `0.3` when the someone it approached refused.
//! That was the crate's only unnamed emotion intensity — a deed's emotion comes from the deed file's
//! `harm`, and `groups.rs` names its contest constants — and it is the largest single source of negative
//! emotion in a village: measured, **112 distress appraisals against 10 joy** over 6 000 ticks and 8
//! people, in a pack where half of all interactions are refused.
//!
//! The ratio, not the magnitude, is what leaves mood's pleasure at or below zero: the PAD virtual centre
//! sits negative, `Mood::step` pulls pleasure toward it scaled by emotion intensity, and the personality
//! baseline is near zero — so pleasure is pulled *down* every tick and can only ever return to zero.
#![cfg(feature = "dialogue")]

mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, InteractionResolved, Person, Place, PlaceTags, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Eight people around three places, in contact, for `ticks`.
fn village(distress: f32, ticks: usize) -> (f32, u32, u32) {
    let mut day = settings();
    day.seconds_per_day = 64.0;
    day.start_hour = 15.0;
    day.tuning.travel_timeout_hours = Some(4.0);
    day.tuning.rejection_distress = distress;
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), day);
    let places: Vec<Entity> = [(-3.0_f32, 0.0), (3.0, 0.0), (0.0, 3.0)]
        .iter()
        .map(|(x, z)| {
            app.world_mut()
                .spawn((
                    Name::new("place"),
                    Place { capacity: 8 },
                    PlaceTags(vec![
                        "gathering".to_owned(),
                        "food".to_owned(),
                        "work".to_owned(),
                    ]),
                    Transform::from_xyz(*x, 0.0, *z),
                ))
                .id()
        })
        .collect();
    let mut people = Vec::new();
    for i in 0..8 {
        let home = places[i % places.len()];
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new((i % 3) as f32 * 0.6, (i / 3) as f32 * 0.6),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Autonomous, Person, Home(home)));
        people.push(person);
    }

    let (mut accepted, mut refused) = (0u32, 0u32);
    let mut pleasure_sum = 0.0_f64;
    let mut samples = 0u32;
    for tick in 0..ticks {
        app.update();
        for resolved in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
            .drain()
        {
            if resolved.accepted {
                accepted += 1;
            } else {
                refused += 1;
            }
        }
        // Sample the second half, so the opening transient does not dominate.
        if tick > ticks / 2 && tick % 50 == 0 {
            for &person in &people {
                if let Some(mood) = app.world().get::<bevy_society::Mood>(person) {
                    pleasure_sum += f64::from(mood.pleasure);
                    samples += 1;
                }
            }
        }
    }
    (
        (pleasure_sum / f64::from(samples.max(1))) as f32,
        accepted,
        refused,
    )
}

#[test]
fn a_refusal_distress_of_zero_leaves_mood_better_than_the_default() {
    let (with, accepted_a, refused_a) = village(0.3, 4000);
    let (without, accepted_b, refused_b) = village(0.0, 4000);
    println!("distress 0.3: mean pleasure {with:+.4} ({accepted_a} accepted, {refused_a} refused)");
    println!(
        "distress 0.0: mean pleasure {without:+.4} ({accepted_b} accepted, {refused_b} refused)"
    );

    // The comparison is only meaningful if the same number of refusals happened.
    assert!(
        refused_a > 0 && refused_b > 0,
        "both runs must contain refusals: {refused_a} and {refused_b}"
    );
    assert!(
        without > with,
        "removing the refusal distress ({without:+.4}) must leave mood no worse than the default \
         ({with:+.4}). If the two are equal the constant is not reaching `Mood::step` at all"
    );
}

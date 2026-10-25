//! **A society's ordinary pleasant interactions must register emotionally.**
//!
//! `village-evening` ends with mood pinned negative, and the cause is a balance, not a single number. Two
//! halves of it are in the definitions file:
//!
//! 1. **43% of accepted interactions pay no emotion at all** — they are `chat` and `gossip`, the two most
//!    common, which pay `social` to both parties and nothing to either party's mood. Only `compliment`
//!    gives Joy, and only to the *target*, so the person who initiates a pleasant exchange gains no affect
//!    from it.
//! 2. An insult pays the target **Anger 0.5**; a compliment pays **Joy 0.3**. The negative is 1.67x the
//!    positive at comparable frequency: measured over 20 000 ticks, 324 accepted insults and 375 accepted
//!    compliments.
//!
//! Cacioppo, Cacioppo & Gollan 2013 (DOI `10.1017/S0140525X13002537`) say the negative channel has the
//! steeper **gain** — so (2) is right as a model — but also that the positive channel has the higher
//! **intercept**, which the crate now has. What neither the paper nor the crate asks for is silence: a
//! pleasant conversation is an *appraisal*, and a pack that gives it no emotion is not modelling a flat
//! affect system, it is declining to model one.
mod common;

use bevy::prelude::*;
use bevy_society::{
    Appraise, Autonomous, Home, InteractionResolved, Person, Place, PlaceTags, SocietyDefs,
};
use common::{headless, settings, spawn_person};

/// Runs the village and counts appraisals **and** accepted interactions, so the two can be compared.
fn run(ticks: usize) -> (std::collections::BTreeMap<String, u32>, u32, u32) {
    let mut day = settings();
    day.seconds_per_day = 64.0;
    day.start_hour = 15.0;
    day.tuning.travel_timeout_hours = Some(4.0);
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), day);
    let at = [(0.0_f32, 0.0), (6.5, -5.0), (-6.5, -5.0), (6.5, 5.0)];
    let places: Vec<Entity> = at
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
    for i in 0..24 {
        let column = i / 8;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(at[column].0, at[column].1),
        );
        app.world_mut().entity_mut(person).insert((
            Autonomous,
            Person,
            Home(places[column % places.len()]),
        ));
    }

    // Keyed by the variant's name: `EmotionKind` is `Eq` but not `Ord`, and the counts are for a
    // readable failure message as much as for the assertion.
    let mut counts: std::collections::BTreeMap<String, u32> = Default::default();
    let (mut accepted, mut refused) = (0u32, 0u32);
    for _ in 0..ticks {
        app.update();
        for appraisal in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<Appraise>>()
            .drain()
        {
            *counts.entry(format!("{:?}", appraisal.kind)).or_default() += 1;
        }
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
    }
    (counts, accepted, refused)
}

#[test]
fn an_accepted_conversation_is_an_appraisal() {
    let (counts, accepted, refused) = run(20_000);
    let of = |names: [&str; 6]| -> u32 {
        names
            .iter()
            .map(|name| counts.get(*name).copied().unwrap_or(0))
            .sum()
    };
    let joy = of(["Joy", "Gratitude", "Admiration", "Liking", "Pride", "Hope"]);
    let negative = of([
        "Distress",
        "Anger",
        "Reproach",
        "Fear",
        "Shame",
        "Disliking",
    ]);
    let total = joy + negative;
    println!("{accepted} accepted, {refused} refused; positive {joy}, negative {negative}");
    println!("all appraisals: {counts:?}");

    assert!(total > 0, "the village must appraise something");
    assert!(
        joy > 0,
        "20 000 ticks and not one positive appraisal: {counts:?}"
    );
    // **Every refusal is an appraisal and no acceptance is.** Measured, `Distress` equals the refusal
    // count exactly (1 216 of 1 216), while `chat` and `gossip` — 43% of accepted interactions — pay no
    // emotion at all, and `compliment` pays only the *target*. So the most common pleasant thing in the
    // village is affectively silent while every unpleasant thing is not, and the negative:positive ratio
    // lands at 3.6:1 without the model deciding on any bias.
    let distress = counts.get("Distress").copied().unwrap_or(0);
    assert_eq!(
        distress, refused,
        "every refusal should be exactly one distress appraisal and no acceptance should be one: \
         {distress} distress against {refused} refusals"
    );
    assert!(
        accepted > 0,
        "the village must accept some interactions: {accepted}"
    );
    // A bias is a steeper gain on a channel that fires. This is the other channel not firing: 1 338
    // accepted interactions produced 443 positive appraisals, fewer than one for every three.
    assert!(
        joy * 3 >= accepted,
        "only {joy} positive appraisals from {accepted} accepted interactions. Cacioppo et al. 2013 \
         give the negative channel the steeper *gain*; they do not give the positive channel silence, \
         and a pack where an accepted conversation pays no emotion is declining to model positive \
         affect rather than modelling a bias"
    );
}

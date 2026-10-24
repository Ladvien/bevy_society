//! **A person walks to the nearest place that serves an activity, not a random one.**
//!
//! `targets_for` returns **every** `Place` carrying the activity's tag, and scoring gives them all the same
//! score — a `Place` target has no `Distance` term in the decision. They therefore tie, and the tie is
//! broken with a **uniform random** pull from the seeded RNG:
//!
//! ```ignore
//! let index = rng.0.below(chosen.targets.len().max(1));
//! let target = *chosen.targets.get(index)?;
//! ```
//!
//! So a person with a tavern two metres away and another across the village picks between them by coin
//! flip. Measured over 17 game-days of `village-evening`, **62.5% of all person-ticks are spent
//! travelling**, and only 14.7% are spent asleep against the 33.3% one 8-hour sleep a day allows.
//!
//! Nearest is the right tie-break for a `Place`: the destination does not move, so there is nothing to
//! randomise for variety. `Person` targets keep the uniform pull — chatting with whichever neighbour comes
//! up is not a defect.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Doing, DoingTarget, Home, Person, Place, PlaceTags, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Two taverns: one 1 m away, one 9 m away. Both serve `food`.
fn two_taverns() -> (bevy::app::App, Entity, Entity, Entity) {
    let defs = SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9, initial: 0.2)],
            activities: [(
                id: "eat",
                target: Place("food"),
                duration_hours: 1.0,
                considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))],
                satisfies: [("food", 1.2)],
            )],
        )"#,
    )
    .expect("the definitions parse");
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    let mut app = headless(defs, short_day);
    let near = app
        .world_mut()
        .spawn((
            Name::new("near"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(1.0, 0.0, 0.0),
        ))
        .id();
    let far = app
        .world_mut()
        .spawn((
            Name::new("far"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(9.0, 0.0, 0.0),
        ))
        .id();
    let person = spawn_person(&mut app, "P0", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(near), Autonomous, Person));
    (app, near, far, person)
}

#[test]
fn a_person_walks_to_the_nearer_of_two_places() {
    let (mut app, near, far, person) = two_taverns();
    let mut picks = std::collections::BTreeMap::<String, u32>::new();

    // Re-decide many times by knocking the person out of `Doing` whenever it arrives, so each iteration is
    // a fresh decision with the same two candidates.
    for _ in 0..6000 {
        app.update();
        if let Some(doing) = app.world().get::<Doing>(person) {
            let chosen = match doing.target {
                DoingTarget::Place(place) => place,
                _ => continue,
            };
            *picks
                .entry(if chosen == near {
                    "near".to_owned()
                } else {
                    "far".to_owned()
                })
                .or_default() += 1;
            // Force a fresh decision on the next tick.
            app.world_mut()
                .entity_mut(person)
                .remove::<(Doing, bevy_society::Destination)>();
        }
    }
    let near_count = picks.get("near").copied().unwrap_or(0);
    let far_count = picks.get("far").copied().unwrap_or(0);
    println!(
        "6000 decisions between a 1 m tavern and a 9 m one: near {near_count}, far {far_count}"
    );
    assert!(
        near_count + far_count > 100,
        "the decision must have run: {picks:?}"
    );
    assert_eq!(
        far_count, 0,
        "with the destination fixed and the near place available, a person must never choose the far \
         one by coin flip: near {near_count}, far {far_count}"
    );
    let _ = far;
}

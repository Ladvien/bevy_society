//! **`Signal::Distance` must report a real distance for a `Place`, not "as far as it can see".**
//!
//! For a `Person` target the signal is genuine: the decision holds every neighbour's position and returns
//! `distance / perception_radius`. For a `Place` or `Point` it returns `None`, and `None` becomes **1.0** —
//! "as far as it can see", the maximum. So a definition that asks `Distance` on a place target is scored as
//! if that place were at the edge of perception, always.
//!
//! Measured: adding `(input: Distance, curve: Linear(-1.0, 1.0))` to the starter pack's `eat` — which targets
//! `Place("food")` — took `food` from a mean of **0.402** to **0.034**, at zero for **93%** of ticks. The
//! activity was not made travel-aware; it was **disabled**, because `Linear(-1, 1)` at input 1.0 is exactly
//! 0.0 and a zero consideration zeroes the whole score.
//!
//! So today a `Place` activity cannot price its walk at all, and the definitions file gives no warning: a
//! host who writes the obvious thing silently loses the activity. `docs/loop/needs-decision/` records this
//! as the blocker for pricing travel.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Person, Place, PlaceTags, SocietyDefs};
use common::{headless, settings, spawn_person};

/// One activity targeting `Place("food")`, scored partly on `Distance`, with a need that decays so the
/// activity keeps being chosen.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9, initial: 0.2)],
            activities: [(
                id: "eat",
                target: Place("food"),
                duration_hours: 0.5,
                considerations: [
                    (input: Need("food"), curve: Linear(-1.0, 1.0)),
                    (input: Distance, curve: Linear(-1.0, 1.0)),
                ],
                satisfies: [("food", 1.2)],
            )],
        )"#,
    )
    .expect("the definitions parse")
}

/// Both taverns at once: one adjacent, one across the village. A person must prefer the near one.
fn eats_near_and_far() -> (u32, u32) {
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    let mut app = headless(defs(), short_day);
    let near = app
        .world_mut()
        .spawn((
            Name::new("near"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(1.0, 0.0, 0.0),
        ))
        .id();
    app.world_mut().spawn((
        Name::new("far"),
        Place { capacity: 8 },
        PlaceTags(vec!["food".to_owned()]),
        Transform::from_xyz(7.0, 0.0, 0.0),
    ));
    let person = spawn_person(&mut app, "P0", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(near), Autonomous, Person));

    let mut near_eats = 0;
    let mut far_eats = 0;
    for _ in 0..6000 {
        app.update();
        // Which place was chosen? `Doing.target` says.
        if let Some(doing) = app.world().get::<bevy_society::Doing>(person)
            && let bevy_society::DoingTarget::Place(chosen) = doing.target
        {
            if chosen == near {
                near_eats += 1;
            } else {
                far_eats += 1;
            }
            app.world_mut()
                .entity_mut(person)
                .remove::<(bevy_society::Doing, bevy_society::Destination)>();
        }
    }
    (near_eats, far_eats)
}

#[test]
fn a_place_one_metre_away_scores_better_than_one_across_the_village() {
    // `perception_radius` is 8 m, so 1 m is 0.125 of perception and 7 m is 0.875.
    let (near, far) = eats_near_and_far();
    println!("eats chosen: adjacent tavern {near}, distant tavern {far}");

    assert!(
        near > 0,
        "a `Place` activity asking for `Distance` must still be reachable: {near}"
    );
    assert!(
        near > far * 3,
        "an adjacent place must score far better than a distant one — {near} against {far}. \
         If both are zero the signal reported the maximum for every place, which disables the activity; \
         if both are equal it is not being read"
    );
}

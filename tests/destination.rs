//! **A person with a `Destination` gets closer to it, with or without the steering feature.**
//!
//! `Destination`'s own doc comment promises this:
//!
//! > Inserted and updated while travelling and removed on arrival or when the activity ends. Games
//! > **without the steering plugin move persons themselves toward this point.**
//!
//! Measured: they did not. A person given `Destination(6.5, -5.0)` sat at `(0, 0)` for 600 ticks and
//! their activity timed out. Every `Place`- and `Home`-targeted activity in a 24-person village then
//! failed while every `Person`-targeted one succeeded, because a `Person` target walks to *them*:
//!
//! | activity | target | started | completed | interrupted |
//! |---|---|---|---|---|
//! | chat, gossip, compliment, snub, wander | `Person`, `Point` | 339 | **339** | 0 |
//! | eat | `Place` | 62 | 14 | **43** |
//! | sleep | `Home` | 79 | 24 | **48** |
//! | work | `Place` | 64 | 6 | **53** |
//!
//! and 89% of `Place`/`Home` activities never arrived at all (26 arrivals against 228 starts).
mod common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Doing, Home, Place, PlaceTags, SocietyDefs,
};
use common::{headless, settings, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9)],
            activities: [(
                id: "eat",
                target: Place("food"),
                duration_hours: 0.75,
                considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))],
                satisfies: [("food", 1.2)],
            )],
        )"#,
    )
    .expect("the definitions parse")
}

/// How far the person has walked from the origin after `ticks`, and whether it arrived.
fn walk(ticks: usize) -> (f32, bool) {
    let mut app = headless(defs(), settings());
    let home = app
        .world_mut()
        .spawn((
            Name::new("home"),
            Place { capacity: 4 },
            PlaceTags(vec![]),
            Transform::default(),
        ))
        .id();
    app.world_mut().spawn((
        Name::new("tavern"),
        Place { capacity: 8 },
        PlaceTags(vec!["food".to_owned()]),
        Transform::from_xyz(6.5, 0.0, -5.0),
    ));
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(home), Autonomous));

    for _ in 0..ticks {
        app.update();
    }
    let moved = app
        .world()
        .get::<Transform>(person)
        .map(|t| Vec2::new(t.translation.x, t.translation.z).length())
        .unwrap_or(0.0);
    let arrived = app
        .world()
        .get::<Doing>(person)
        .is_some_and(|d| d.arrived_day.is_some());
    (moved, arrived)
}

#[test]
fn a_person_walks_toward_its_destination() {
    // 400 ticks is enough for an 8.2 m walk at 1.4 m/s over a 1/64 s step, with room to spare.
    let (moved, arrived) = walk(400);
    println!("moved {moved:.2} m from the origin; arrived {arrived}");
    assert!(
        moved > 0.5,
        "a person with a Destination must move toward it: moved {moved:.3} m in 400 ticks"
    );
}

#[test]
fn and_arrives_so_a_place_activity_can_complete() {
    let (_, arrived) = walk(700);
    assert!(
        arrived,
        "the walk must finish, or every Place-targeted activity times out"
    );
}

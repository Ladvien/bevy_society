//! **A person must not walk through a building.**
//!
//! `steer_movers` passes an empty wall list to `bevy_orca::new_velocity` — the parameter is there, the
//! crate fills it with `&[]`. So the only thing a walker avoids is another walker, and a villager
//! crossing the village walks straight through the well.
//!
//! This is not only a demo problem. `docs/guide/concepts.md` says steering gets people to where they
//! are going; a host that puts a building in the way — a tavern, a wall, a stall — has no way to say
//! so, and the person's `Transform` ends up inside it. The README clips show the result: villagers
//! standing inside a 3 m block with half a body showing.
//!
//! The fix is a component the host attaches to anything solid, and the radius that solid should push
//! out to. `bevy_orca` already takes hard wall half-planes, so nothing new is needed in the solver.
#![cfg(feature = "steering")]

use crate::common;

use bevy::prelude::*;
use bevy_society::{Obstacle, Person, SocialMover, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Builds a world with one walker heading at a solid block, and reports the closest it got.
fn closest_approach(obstacle: Option<Obstacle>) -> f32 {
    let mut day = settings();
    day.seconds_per_day = 240.0;
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), day);
    let person = spawn_person(&mut app, "P0", Vec2::new(-6.0, 0.0));
    // **Not** `Autonomous`: a host that drives a person itself must be able to, and the decision
    // system would otherwise replace the `Destination` below on every society tick.
    app.world_mut()
        .entity_mut(person)
        .insert((Person, SocialMover::default()));

    // A 3 m block centred on the origin, so its face is 1.5 m from the centre.
    let mut block = app.world_mut().spawn((
        Name::new("block"),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Mesh3d::default(),
    ));
    if let Some(obstacle) = obstacle {
        block.insert(obstacle);
    }

    // Send the walker at the far side of the block.
    app.world_mut()
        .entity_mut(person)
        .insert(bevy_society::Destination(Vec2::new(6.0, 0.0)));

    let mut closest = f32::MAX;
    for _ in 0..4000 {
        app.update();
        if let Some(transform) = app.world().get::<Transform>(person) {
            let at = Vec2::new(transform.translation.x, transform.translation.z);
            closest = closest.min(at.length());
        }
    }
    closest
}

#[test]
fn a_walker_does_not_cross_a_solid_block() {
    let with = closest_approach(Some(Obstacle { radius: 2.0 }));
    let without = closest_approach(None);
    println!("closest approach: with an Obstacle {with:.2} m, without {without:.2} m");
    assert!(
        without < 1.0,
        "the control must reach the block: without an `Obstacle` the walker got to {without:.2} m, \
         so the walker is not moving at all and this test proves nothing"
    );

    assert!(
        with >= 1.4,
        "the walker came within {with:.2} m of a block whose face is 1.5 m from its centre, so it \
         walked through it. `Obstacle` is the component that says a thing is solid"
    );
}

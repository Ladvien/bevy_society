//! Steering (plan D10): ORCA-driven movement toward a `Destination`.
#![cfg(feature = "steering")]

mod common;

use bevy::prelude::*;
use bevy_society::{Destination, Person, SocialMover, SocietyDefs, SocietySettings};

/// Spawn a root-level person that walks toward `goal` (plane coordinates `(x, z)`).
fn spawn_mover(app: &mut bevy::app::App, start: Vec3, goal: Option<Vec2>) -> Entity {
    let mut entity = app.world_mut().spawn((
        Person,
        SocialMover::default(),
        Transform::from_translation(start),
    ));
    if let Some(goal) = goal {
        entity.insert(Destination(goal));
    }
    entity.id()
}

#[test]
fn opposing_movers_swap_ends_without_colliding() {
    let mut app = common::headless(SocietyDefs::default(), SocietySettings::default());
    let a = spawn_mover(
        &mut app,
        Vec3::new(-3.0, 0.0, 0.0),
        Some(Vec2::new(3.0, 0.0)),
    );
    let b = spawn_mover(
        &mut app,
        Vec3::new(3.0, 0.0, 0.0),
        Some(Vec2::new(-3.0, 0.0)),
    );

    let mut min_gap = f32::MAX;
    for _ in 0..640 {
        app.update();
        let pa = app
            .world()
            .get::<Transform>(a)
            .expect("mover a")
            .translation;
        let pb = app
            .world()
            .get::<Transform>(b)
            .expect("mover b")
            .translation;
        min_gap = min_gap.min(pa.distance(pb));
    }

    assert!(
        min_gap >= 0.6,
        "movers came within {min_gap} m of each other"
    );
    let pa = app
        .world()
        .get::<Transform>(a)
        .expect("mover a")
        .translation;
    let pb = app
        .world()
        .get::<Transform>(b)
        .expect("mover b")
        .translation;
    assert!(
        pa.distance(Vec3::new(3.0, 0.0, 0.0)) <= 0.8,
        "mover a did not arrive, ended at {pa}"
    );
    assert!(
        pb.distance(Vec3::new(-3.0, 0.0, 0.0)) <= 0.8,
        "mover b did not arrive, ended at {pb}"
    );
}

#[test]
fn mover_without_destination_stays_put() {
    let mut app = common::headless(SocietyDefs::default(), SocietySettings::default());
    let start = Vec3::new(1.0, 0.0, -2.0);
    let still = spawn_mover(&mut app, start, None);

    for _ in 0..64 {
        app.update();
    }

    let at = app
        .world()
        .get::<Transform>(still)
        .expect("mover")
        .translation;
    assert_eq!(at, start, "a mover with no destination must not move");
}

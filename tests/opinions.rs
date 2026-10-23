//! Opinion dynamics (plan D5): conformity to a strong group mean, and the influence one accepted
//! exchange has on both sides.

mod common;

use bevy::prelude::*;
use bevy_society::{Opinion, Opinions, Personality, SocietyDefs, SocietySettings};
use common::{headless, run, settings, spawn_person};

/// Settings where one game-day is 64 ticks, so `run(app, 65)` advances about one game-day.
fn day_settings() -> SocietySettings {
    let mut s = settings();
    s.seconds_per_day = 1.0;
    s
}

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(r#"(topics: [(id: "honesty")])"#)
        .expect("the one-topic definition parses")
}

/// Spawns a person at `pos` with the given honesty opinion and agreeableness.
fn spawn_with(app: &mut App, name: &str, pos: Vec2, honesty: f32, agreeableness: f32) -> Entity {
    let entity = spawn_person(app, name, pos);
    if let Some(mut opinions) = app.world_mut().get_mut::<Opinions>(entity) {
        opinions.0 = vec![Opinion::at(honesty)];
    }
    if let Some(mut personality) = app.world_mut().get_mut::<Personality>(entity) {
        personality.agreeableness = agreeableness;
    }
    entity
}

fn honesty(app: &App, entity: Entity) -> f32 {
    app.world()
        .get::<Opinions>(entity)
        .expect("a person has opinions")
        .value(0)
}

#[test]
fn conformity_pulls_an_agreeable_person_toward_a_confident_group() {
    let mut app = headless(defs(), day_settings());
    run(&mut app, 1);

    // Four persons within a metre of each other, so all four count as contacts of each other.
    for (i, name) in ["A", "B", "C"].iter().enumerate() {
        spawn_with(&mut app, name, Vec2::new(i as f32, 0.0), 0.8, 0.5);
    }
    let loner = spawn_with(&mut app, "D", Vec2::new(0.0, 1.0), 0.0, 1.0);
    assert!(
        honesty(&app, loner).abs() < 1e-6,
        "starts at 0.0 before any tick"
    );

    run(&mut app, 65);

    let moved = honesty(&app, loner);
    // The closed form after 65 ticks of dt = 1/64 day is about 0.076; `n/(n+3)` with n = 3 and
    // agreeableness 1.0 make it slower than the raw rate.
    assert!(
        moved > 0.05 && moved < 0.8,
        "the agreeable person moved toward the group, not past it: {moved}"
    );
}

#[test]
fn conformity_ignores_a_lukewarm_group() {
    let mut app = headless(defs(), day_settings());
    run(&mut app, 1);
    for (i, name) in ["A", "B", "C"].iter().enumerate() {
        spawn_with(&mut app, name, Vec2::new(i as f32, 0.0), 0.2, 0.5);
    }
    let loner = spawn_with(&mut app, "D", Vec2::new(0.0, 1.0), 0.0, 1.0);

    run(&mut app, 65);

    assert_eq!(
        honesty(&app, loner),
        0.0,
        "a group mean of 0.2 is inside the |m| > 0.3 gate, so nothing moves"
    );
}

#[test]
fn a_person_alone_does_not_conform() {
    let mut app = headless(defs(), day_settings());
    run(&mut app, 1);
    let lonely = spawn_with(&mut app, "A", Vec2::ZERO, 0.0, 1.0);
    run(&mut app, 1);

    run(&mut app, 65);

    assert_eq!(
        honesty(&app, lonely),
        0.0,
        "conformity needs n >= 3 contacts"
    );
}

#[test]
fn a_missing_transform_does_not_stop_the_tick() {
    // A person with no transform sits at the origin for perception; nothing panics.
    let mut app = headless(defs(), day_settings());
    run(&mut app, 1);
    let a = spawn_with(&mut app, "A", Vec2::ZERO, 0.9, 0.5);
    app.world_mut()
        .entity_mut(a)
        .remove::<(Transform, GlobalTransform)>();
    run(&mut app, 3);
    assert_eq!(honesty(&app, a), 0.9, "no contacts, no conformity");
}

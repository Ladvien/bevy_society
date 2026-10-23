//! Shared test harness: a headless society app and person spawning.
#![allow(dead_code)]

use std::time::Duration;

use bevy::MinimalPlugins;
use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::math::{Vec2, Vec3};
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;
use bevy::transform::components::Transform;
use bevy_society::{Person, SocietyDefs, SocietyPlane, SocietyPlugins, SocietySettings};

/// Settings with one society tick per fixed step at 64 Hz, so each `app.update()` after the first
/// runs exactly one tick.
pub fn settings() -> SocietySettings {
    SocietySettings {
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    }
}

/// A headless app running the whole society with the given definitions and settings. The app is
/// finished (plugins' `finish` checks have run) but not yet updated.
pub fn headless(defs: SocietyDefs, settings: SocietySettings) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 64.0,
        )))
        .insert_resource(settings)
        .insert_resource(defs)
        .add_plugins(SocietyPlugins);
    app.finish();
    app.cleanup();
    app
}

/// Spawns a named person at `pos` on the society's plane and returns its entity.
pub fn spawn_person(app: &mut App, name: &str, pos: Vec2) -> Entity {
    let plane = app.world().resource::<SocietySettings>().plane;
    let translation = match plane {
        SocietyPlane::XZ => Vec3::new(pos.x, 0.0, pos.y),
        SocietyPlane::XY => Vec3::new(pos.x, pos.y, 0.0),
    };
    app.world_mut()
        .spawn((
            Person,
            Name::new(name.to_owned()),
            Transform::from_translation(translation),
        ))
        .id()
}

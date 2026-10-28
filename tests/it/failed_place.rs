//! **A place whose walk just failed is not chosen again at once** (`Tuning::failed_place_memory_hours`).
//!
//! A `Place` target is the nearest tied place, so a person whose nearest tavern cannot be reached chose it
//! again the tick its walk ended. With a memory the next choice is the next-nearest place.

use crate::common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Doing, DoingTarget, Home, Locomotion, Place, PlaceTags, SocietyClock, SocietyDefs,
    SocietySettings,
};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9)],
            activities: [
                (
                    id: "eat",
                    target: Place("food"),
                    duration_hours: 0.75,
                    considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("food", 1.2)],
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

/// A host-walked person who never moves, a near tavern at 8 m and a far one at 12 m.
fn app(memory: Option<f32>) -> (App, Entity, Entity, Entity) {
    let mut settings = SocietySettings {
        locomotion: Locomotion::Host,
        tick_secs: 1.0 / 64.0,
        seconds_per_day: 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = None;
    settings.tuning.stall_timeout_hours = Some(1.0);
    settings.tuning.failed_place_memory_hours = memory;
    let mut app = headless(defs(), settings);
    let home = app
        .world_mut()
        .spawn((
            Name::new("home"),
            Place { capacity: 4 },
            PlaceTags(vec![]),
            Transform::default(),
        ))
        .id();
    let near = app
        .world_mut()
        .spawn((
            Name::new("near"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(8.0, 0.0, 0.0),
        ))
        .id();
    let far = app
        .world_mut()
        .spawn((
            Name::new("far"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(12.0, 0.0, 0.0),
        ))
        .id();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(home), Autonomous));
    (app, person, near, far)
}

/// The place each successive walk of `person` aimed at, over `hours` game-hours.
fn walked_places(app: &mut App, person: Entity, hours: f64) -> Vec<Entity> {
    // The first update sets the clock to its start of day, so measure from after it.
    app.update();
    let end = app.world().resource::<SocietyClock>().now_days + hours / 24.0;
    let mut seen: Vec<Entity> = Vec::new();
    let mut last_start = f64::NEG_INFINITY;
    while app.world().resource::<SocietyClock>().now_days < end {
        app.update();
        if let Some(doing) = app.world().get::<Doing>(person)
            && let DoingTarget::Place(place) = doing.target
            && doing.started_day > last_start
        {
            last_start = doing.started_day;
            seen.push(place);
        }
    }
    seen
}

#[test]
fn without_memory_the_same_unreachable_place_is_chosen_every_window() {
    let (mut app, person, near, _) = app(None);
    let seen = walked_places(&mut app, person, 4.5);
    assert!(
        seen.len() >= 2,
        "expected several walks, saw {}",
        seen.len()
    );
    assert!(seen.iter().all(|place| *place == near), "{seen:?}");
}

#[test]
fn a_failed_place_is_skipped_for_the_memory_window_then_forgiven() {
    let (mut app, person, near, far) = app(Some(2.5));
    // Walk 1 (near) fails at ~1 h, walk 2 (far) fails at ~2 h, then both are remembered until
    // ~3.5 h and ~4.5 h: near is forgiven first.
    let seen = walked_places(&mut app, person, 5.0);
    assert_eq!(seen.first(), Some(&near), "{seen:?}");
    assert_eq!(seen.get(1), Some(&far), "{seen:?}");
    assert_eq!(seen.get(2), Some(&near), "{seen:?}");
}

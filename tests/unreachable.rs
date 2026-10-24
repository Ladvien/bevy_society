//! **A place the timeout cannot reach is named at startup.**
//!
//! `Tuning::travel_timeout_hours` is game hours; `SocialMover::speed` is metres per **real** second
//! at the fixed step. Two clocks, and a host sets both with no way to know they must agree. That
//! coupling is `needs-decision/travel-time-in-two-clocks.md`, still open — but the *silence* is not a
//! decision the crate may defer. It knows the speed, the timeout, the day length and where the places
//! are, so it can say when a walk cannot finish.
//!
//! Measured on the village layout at `seconds_per_day = 4`:
//!
//! | walk | ticks | timeout allows |
//! |---|---|---|
//! | 8.2 m | 308 | 426 |
//! | 13 m | **527** | **426** |
//!
//! So the far corner always times out: 78 of 108 `eat`s and 44 of 108 `sleep`s were interrupted, and
//! the three needs those activities serve sat at 0.000 for the run.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Place, PlaceTags, SocietyDefs, SocietySettings};
use common::capture::{LogCapture, log_plugin};

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

/// A headless app with log capture, one place `distance` metres away, and the given settings.
fn app_with(settings: SocietySettings, distance: f32) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, log_plugin()))
        .insert_resource(settings)
        .insert_resource(defs());
    let plugins = bevy_society::SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();
    let tavern = app
        .world_mut()
        .spawn((
            Name::new("tavern"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(distance, 0.0, 0.0),
        ))
        .id();
    let person = common::spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(tavern), Autonomous));
    app
}

#[test]
fn a_place_beyond_reach_is_named_at_startup() {
    // The village's own numbers: a 40-hour timeout at a four-second day is 426 ticks, which at 1.4 m/s
    // over a 1/64 s step is only 9.3 m — under the 10 m floor a layout needs.
    let settings = SocietySettings {
        seconds_per_day: 4.0,
        tick_secs: 1.0 / 64.0,
        tuning: bevy_society::Tuning {
            travel_timeout_hours: 40.0,
            ..Default::default()
        },
        ..common::settings()
    };
    let app = app_with(settings, 30.0);

    let captured = app
        .world()
        .get_resource::<LogCapture>()
        .expect("the capture layer inserted its resource")
        .drain();
    let warning: Vec<&String> = captured
        .iter()
        .filter(|event| event.message.contains("metres of walking"))
        .map(|event| &event.message)
        .collect();
    assert_eq!(
        warning.len(),
        1,
        "one startup warning about short travel reach: {captured:?}"
    );
    assert!(
        warning[0].contains("travel_timeout_hours"),
        "the warning names the setting to change: {}",
        warning[0]
    );
    println!("{}", warning[0]);
}

#[test]
fn a_place_within_reach_is_not_warned_about() {
    // A day long enough that the same timeout buys hundreds of metres: no warning.
    let settings = SocietySettings {
        seconds_per_day: 240.0,
        tick_secs: 1.0 / 64.0,
        tuning: bevy_society::Tuning {
            travel_timeout_hours: 40.0,
            ..Default::default()
        },
        ..common::settings()
    };
    let app = app_with(settings, 4.0);
    let captured = app
        .world()
        .get_resource::<LogCapture>()
        .expect("the capture layer inserted its resource")
        .drain();
    let warnings: Vec<&String> = captured
        .iter()
        .filter(|event| event.message.contains("metres of walking"))
        .map(|event| &event.message)
        .collect();
    assert!(
        warnings.is_empty(),
        "a day long enough to walk across must not be warned about: {warnings:?}"
    );
}

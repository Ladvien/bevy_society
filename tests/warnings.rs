//! **The startup warnings, asserted.** Until now both were pinned only by the facts they key on,
//! because a headless test app had no logger: deleting either `warn!` left every test green.
//! `common::capture` records every log event into a resource a test can read.
//!
//! The `tracing` subscriber is **process-global and settable once**, so a test binary may build only
//! one capturing app. Both warnings are therefore asserted from a single run.
mod common;

use bevy::prelude::*;
use bevy_society::{SocietyDefs, SocietySettings};
use common::capture::{LogCapture, log_plugin};
use common::settings;

/// A headless app with log capture, the given defs and settings, finished so `finish()` has run.
fn app_with(defs: SocietyDefs, settings: SocietySettings) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        log_plugin(),
    ))
    .insert_resource(settings)
    .insert_resource(defs);
    let plugins = bevy_society::SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();
    app
}

/// One activity that is short *and* uses a `Field` consideration with a `Gradient` target, so one app
/// exercises both warnings.
fn both_offending_defs() -> SocietyDefs {
    #[cfg(feature = "fields")]
    let ron = r#"(
        needs: [(id: "social", decay_per_day: 1.0)],
        activities: [
            (
                id: "snub",
                target: Itself,
                duration_hours: 0.1,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
            ),
            (
                id: "follow",
                target: Gradient(Crowd, true),
                duration_hours: 1.0,
                considerations: [(input: Field(Crowd), curve: Linear(1.0, 0.0))],
            ),
        ],
    )"#;
    // Without the `fields` feature a `Field` input and a `Gradient` target are rejected at compile
    // time, so only the duration warning can fire.
    #[cfg(not(feature = "fields"))]
    let ron = r#"(
        needs: [(id: "social", decay_per_day: 1.0)],
        activities: [(
            id: "snub",
            target: Itself,
            duration_hours: 0.1,
            considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
        )],
    )"#;
    SocietyDefs::from_ron(ron).expect("the offending definition parses")
}

#[test]
fn every_startup_warning_is_asserted_from_one_app() {
    // At 4 s per day one tick is 0.094 game-hours, so a 0.1-hour activity lasts 1.1 ticks.
    let settings = SocietySettings {
        seconds_per_day: 4.0,
        ..settings()
    };
    let app = app_with(both_offending_defs(), settings);
    let capture = app
        .world()
        .get_resource::<LogCapture>()
        .expect("the capture layer inserted its resource");
    let captured = capture.drain();

    let duration: Vec<&String> = captured
        .iter()
        .filter(|event| event.message.contains("shorter than two society ticks"))
        .map(|event| &event.message)
        .collect();
    assert_eq!(
        duration.len(),
        1,
        "exactly one degenerate-duration warning: {captured:?}"
    );
    assert!(
        duration[0].contains("snub"),
        "the duration warning names the activity: {}",
        duration[0]
    );
    assert!(
        duration[0].contains("1.1 ticks"),
        "the duration warning carries the arithmetic: {}",
        duration[0]
    );

    #[cfg(feature = "fields")]
    {
        let field: Vec<&String> = captured
            .iter()
            .filter(|event| event.message.contains("social-field inputs"))
            .map(|event| &event.message)
            .collect();
        assert_eq!(
            field.len(),
            1,
            "exactly one unscored-field warning: {captured:?}"
        );
        assert!(
            field[0].contains("follow"),
            "the field warning names the activity: {}",
            field[0]
        );
        assert!(
            field[0].contains("1 definitions"),
            "one offending activity is one offender, not two: {}",
            field[0]
        );
    }
}

#[test]
fn the_starter_pack_warns_about_nothing() {
    // A host on the crate's own defaults must see neither warning. This app cannot capture (the
    // subscriber is already set), so it asserts on the checks themselves.
    let defs = SocietyDefs::starter().expect("the starter pack parses");
    assert!(
        !defs.has_gradient_activity(),
        "the starter pack has no Gradient activity"
    );
    let field_readers: Vec<&str> = defs
        .activity_signals()
        .into_iter()
        .filter(|(_, signals)| {
            signals
                .iter()
                .any(|s| matches!(s, bevy_society::Signal::Field(_)))
        })
        .map(|(name, _)| name)
        .collect();
    assert!(
        field_readers.is_empty(),
        "the starter pack must not trigger the field warning: {field_readers:?}"
    );
    // And no starter activity is degenerate at the default 240 s day, where a tick is 0.025 h.
    let tick_hours = f64::from(settings().tick_secs) / 240.0 * 24.0;
    let short: Vec<&str> = defs
        .activity_durations()
        .into_iter()
        .skip(1)
        .filter(|(_, hours)| f64::from(*hours) / tick_hours < 2.0)
        .map(|(name, _)| name)
        .collect();
    assert!(
        short.is_empty(),
        "no starter activity is degenerate at the default day: {short:?}"
    );
}

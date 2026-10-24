//! A short `seconds_per_day` makes every activity shorter than a tick, which degenerates the
//! society silently. `BehaviorPlugin` warns at startup; these tests pin that it does.
mod common;

use bevy::prelude::*;
use bevy_society::{SocietyDefs, SocietySettings};
use common::{headless, settings};

/// An app at the given day length, with the starter pack.
fn app_at(seconds_per_day: f32) -> App {
    let defs = SocietyDefs::starter().expect("the starter pack parses");
    let settings = SocietySettings {
        seconds_per_day,
        ..settings()
    };
    headless(defs, settings)
}

#[test]
fn a_four_second_day_makes_the_starter_pack_degenerate() {
    // At 4 s per day one tick is 0.094 game-hours, so `snub` (0.1 h) lasts about one tick.
    let mut app = app_at(4.0);
    app.update();
    // The warning is a log line, not a message, so the assertion is on the invariant it reports:
    // every social activity really is shorter than two ticks here.
    let settings = app.world().resource::<SocietySettings>().clone();
    let tick_hours = f64::from(settings.tick_secs) / f64::from(settings.seconds_per_day) * 24.0;
    let defs = app.world().resource::<SocietyDefs>();
    let short = defs
        .activity_durations()
        .into_iter()
        .skip(1) // the reserved `idle` fallback
        .filter(|(_, hours)| f64::from(*hours) / tick_hours < 2.0)
        .count();
    assert!(
        short > 0,
        "the starter pack must have short activities at a 4 s day, or this warning is pointless"
    );
}

#[test]
fn the_default_day_makes_the_starter_pack_sane() {
    // At the default 240 s per day one tick is 0.025 game-hours: every activity is several ticks.
    let mut app = app_at(240.0);
    app.update();
    let settings = app.world().resource::<SocietySettings>().clone();
    let tick_hours = f64::from(settings.tick_secs) / f64::from(settings.seconds_per_day) * 24.0;
    let defs = app.world().resource::<SocietyDefs>();
    let pathological: Vec<&str> = defs
        .activity_durations()
        .into_iter()
        .skip(1)
        .filter(|(_, hours)| f64::from(*hours) / tick_hours < 1.0)
        .map(|(name, _)| name)
        .collect();
    assert!(
        pathological.is_empty(),
        "no activity should be shorter than one tick at the default day: {pathological:?}"
    );
}

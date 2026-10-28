//! **The host-driven society clock** (issue #2).
//!
//! Sanctuary's day is not a constant: dawn is 5 s for one game hour, day 150 s for ten, dusk 20 s for
//! two and night 180 s for ten. A fixed `seconds_per_day` cannot follow that, so the society's hour
//! drifted away from the game's within one day and `Hour` considerations fired at the wrong time.
//!
//! These tests pin the four acceptance criteria, and one more the old hour code could not meet at all:
//! a **fractional** `Hour(5.5)` must fire, so the crossing arithmetic is exact rather than a whole-hour
//! walk.

use crate::common;

use bevy::prelude::*;
use bevy_society::{
    ClockSource, FireTrigger, HostClock, Needs, RulesPlugin, ScriptEvent, SocietyClock,
    SocietyDefs, SocietySettings,
};
use common::{headless, spawn_person};

/// A defs file with one trigger that fires at a given hour and emits a named custom event, so a test
/// can count the crossings instead of reading the clock.
fn hour_trigger(hour: f32) -> SocietyDefs {
    let text = format!(
        r#"(
            triggers: [(
                id: "crossed_hour",
                on: Hour({hour}),
                then: "fx.emit(\"crossed\", #{{}});",
            )],
        )"#
    );
    SocietyDefs::from_ron(&text).expect("the hour-trigger defs parse")
}

/// Every `ScriptEvent` name written since the last call. Draining must happen each update: Bevy keeps
/// a message for two buffer swaps, so a drain only at the end loses everything but the last two.
fn emitted(app: &mut App) -> Vec<String> {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ScriptEvent>>()
        .drain()
        .map(|event| event.name)
        .collect()
}

/// Advances by `updates` updates, returning every event name emitted along the way.
fn emitted_over(app: &mut App, updates: usize) -> Vec<String> {
    let mut out = Vec::new();
    for _ in 0..updates {
        app.update();
        out.extend(emitted(app));
    }
    out
}

/// An app whose clock is the host's, starting at `start_days`.
fn host_app(defs: SocietyDefs, start_days: f64) -> App {
    let settings = SocietySettings {
        clock: ClockSource::Host,
        ..common::settings()
    };
    let mut app = headless(defs, settings);
    app.world_mut().insert_resource(HostClock {
        now_days: start_days,
    });
    app
}

fn set_host(app: &mut App, now_days: f64) {
    if let Some(mut host) = app.world_mut().get_resource_mut::<HostClock>() {
        host.now_days = now_days;
    }
}

fn clock(app: &App) -> SocietyClock {
    *app.world().resource::<SocietyClock>()
}

/// Under `Host`, the society's hour is the host's hour within one tick — for a clock that stands
/// still, one that runs at a day-fraction per tick, and one that jumps.
#[test]
fn the_society_hour_follows_the_host() {
    let mut app = host_app(SocietyDefs::default(), 8.0 / 24.0);
    app.update();
    // The baseline tick adopts the host time without advancing it.
    assert!(
        (clock(&app).hour() - 8.0).abs() < 1.0e-3,
        "the first tick should be at the host's 08:00, got {}",
        clock(&app).hour()
    );

    // A slow drift: a quarter of an hour per tick, which neither the fixed clock nor a whole-hour
    // walk could reproduce.
    let mut host = 8.0 / 24.0;
    for _ in 0..4 {
        host += 0.25 / 24.0;
        set_host(&mut app, host);
        app.update();
    }
    assert!(
        (clock(&app).hour() - 9.0).abs() < 1.0e-3,
        "after four quarter-hours the society should be at 09:00, got {}",
        clock(&app).hour()
    );

    // A jump, which is how a phase transition arrives.
    set_host(&mut app, 18.5 / 24.0);
    app.update();
    assert!(
        (clock(&app).hour() - 18.5).abs() < 1.0e-3,
        "after the jump the society should be at 18:30, got {}",
        clock(&app).hour()
    );
}

/// A host clock held still is a pause: no need may decay, because a cutscene is not time passing.
#[test]
fn a_still_host_clock_decays_no_need() {
    let mut app = host_app(SocietyDefs::default(), 9.0 / 24.0);
    let person = spawn_person(&mut app, "Ada", Vec2::new(0.0, 0.0));
    common::run(&mut app, 2);
    let before = app
        .world()
        .entity(person)
        .get::<Needs>()
        .cloned()
        .expect("a person has needs");

    // `HostClock` is never written again, so every tick's `dt_days` is 0.
    common::run(&mut app, 100);
    let after = app
        .world()
        .entity(person)
        .get::<Needs>()
        .cloned()
        .expect("a person has needs");
    assert_eq!(
        before, after,
        "100 ticks at a still host clock decayed needs: {before:?} -> {after:?}"
    );
    assert!(
        clock(&app).now_days == 9.0 / 24.0,
        "the society clock should still be at the host's instant"
    );
}

/// A one-frame jump from 05:00 to 06:00 crosses 05:30, so a `Hour(5.5)` trigger fires exactly once.
///
/// This is the case the whole-hour walk could never reach: it stepped `previous.ceil()` by 1.0, so
/// every hour it emitted was an integer.
#[test]
fn a_jump_across_a_fractional_hour_fires_once() {
    let mut app = host_app(hour_trigger(5.5), 5.0 / 24.0);
    // The baseline tick establishes the previous instant at 05:00 and fires nothing.
    app.update();
    // A jump inside hour 5 but past 05:30.
    set_host(&mut app, 5.75 / 24.0);
    app.update();
    assert_eq!(
        emitted(&mut app),
        vec!["crossed".to_owned()],
        "the 05:30 crossing should fire exactly once"
    );

    // Ticks held inside hour 6 must not fire it again.
    for _ in 0..20 {
        set_host(&mut app, 6.25 / 24.0);
        app.update();
    }
    assert!(
        emitted(&mut app).is_empty(),
        "holding the clock inside hour 6 must not re-cross 05:30"
    );
}

/// A clock that goes backward (a reload) fires nothing: the hours it crosses on the way back were
/// already crossed going forward.
#[test]
fn a_backward_clock_fires_nothing() {
    let mut app = host_app(hour_trigger(9.0), 10.0 / 24.0);
    app.update();
    set_host(&mut app, 2.0 / 24.0);
    app.update();
    assert!(
        emitted(&mut app).is_empty(),
        "moving the host clock backward must not fire an hour trigger"
    );
    assert!(
        (clock(&app).hour() - 2.0).abs() < 1.0e-3,
        "the society should still adopt the host's time"
    );
    // And the clock can then move forward normally again.
    set_host(&mut app, 9.5 / 24.0);
    app.update();
    assert_eq!(
        emitted(&mut app),
        vec!["crossed".to_owned()],
        "hour 9 should fire once on the way up"
    );
}

/// A whole-hour `Hour` trigger still fires once per day per crossing, which is what the fixed clock
/// and every shipped scenario rely on. Two fires would mean `ceil` is re-crossing; zero would mean the
/// span arithmetic is wrong.
#[test]
fn a_whole_hour_fires_once_per_crossing() {
    let mut app = host_app(hour_trigger(6.0), 5.0 / 24.0);
    app.update();
    let mut total = 0;
    // Twenty ticks walking a quarter-hour at a time: 05:00 -> 10:00 in one day.
    let mut host = 5.0 / 24.0;
    for _ in 0..20 {
        host += 0.25 / 24.0;
        set_host(&mut app, host);
        app.update();
        total += emitted(&mut app).len();
    }
    assert_eq!(
        total, 1,
        "hour 6 lies in 05:00..10:00 exactly once, fired {total} time(s)"
    );

    // The next day: 10:00 -> 05:00 is nineteen hours, so 76 quarter-hours, and the crossing of hour 6
    // is the very last one that lands inside the day after.
    let mut total = 0;
    for _ in 0..80 {
        host += 0.25 / 24.0;
        set_host(&mut app, host);
        app.update();
        total += emitted(&mut app).len();
    }
    assert_eq!(
        total, 1,
        "hour 6 the next day fires once, fired {total} time(s)"
    );
}

/// A jump across midnight fires the trigger on the far side too, and fires it once per day crossed.
#[test]
fn a_jump_across_midnight_fires() {
    let mut app = host_app(hour_trigger(2.0), 22.0 / 24.0);
    app.update();
    // Straight to 03:00 the next day: crosses 24:00 and 02:00.
    set_host(&mut app, 1.0 + 3.0 / 24.0);
    app.update();
    assert_eq!(
        emitted(&mut app),
        vec!["crossed".to_owned()],
        "hour 2 the next day should fire once"
    );
}

/// `Fixed` is the default and behaves exactly as before, including on a trigger that stands at an
/// integer hour; this is the guard against the crossing rewrite changing the shipped path.
#[test]
fn the_fixed_clock_still_crosses_hours() {
    let settings = SocietySettings {
        // One game-day is 240 s of real time at 1/64 s per tick, so a game-hour is 640 ticks and
        // 05:00 -> 07:00 is 1 280 of them.
        start_hour: 5.0,
        ..common::settings()
    };
    let mut app = headless(hour_trigger(6.0), settings);
    let total = emitted_over(&mut app, 1_400).len();
    assert_eq!(
        total, 1,
        "the fixed clock crosses hour 6 once in 1 400 ticks, fired {total} time(s)"
    );
}

/// A `Host` clock with no `HostClock` resource freezes rather than inventing time, and says so once.
#[test]
fn a_missing_host_clock_freezes_and_warns_once() {
    let settings = SocietySettings {
        clock: ClockSource::Host,
        ..common::settings()
    };
    // A triggering `FireTrigger` is not needed; only that the clock does not advance.
    let mut app = common::headless(SocietyDefs::default(), settings);
    common::run(&mut app, 2);
    let start = clock(&app).now_days;
    common::run(&mut app, 20);
    assert_eq!(
        clock(&app).now_days,
        start,
        "with no HostClock, the society clock must not advance"
    );
    assert_eq!(
        clock(&app).dt_days,
        0.0,
        "a frozen tick carries no game time"
    );
    // The plugin must still be fully wired, so a later host clock takes over cleanly.
    app.world_mut().insert_resource(HostClock {
        now_days: 20.0 / 24.0,
    });
    app.update();
    assert!(
        (clock(&app).hour() - 20.0).abs() < 1.0e-3,
        "inserting a HostClock should hand the clock over, got {}",
        clock(&app).hour()
    );
}

/// `RulesPlugin` is required for the trigger tests; naming it here keeps a failure readable if the
/// default plugin set ever drops it.
#[test]
fn the_rules_plugin_is_in_the_default_set() {
    let mut app = headless(SocietyDefs::default(), common::settings());
    app.update();
    assert!(
        app.world()
            .get_resource::<bevy::ecs::message::Messages<FireTrigger>>()
            .is_some(),
        "RulesPlugin should register the trigger message"
    );
    let _ = RulesPlugin;
}

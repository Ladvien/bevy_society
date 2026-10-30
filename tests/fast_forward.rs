//! **Fast-forward**: running a society a number of days in one call gives the society a live app
//! has after the same ticks, and refuses the cases it cannot advance.

mod common;

use bevy::prelude::*;
use bevy_society::{
    ClockSource, FastForwardError, PlacementFile, SocietyClock, SocietyDefs, SocietySettings,
    fast_forward,
};
use common::hash::hash;
use common::{headless, settings};

const DEFS: &str = r#"(
    needs: [(id: "hunger", decay_per_day: 1.0)],
    activities: [(
        id: "grind", target: Bound("work"), duration_hours: 2.0,
        considerations: [(input: Need("hunger"), curve: Linear(-1.0, 1.0))],
        satisfies: [("hunger", 0.3)],
    )],
)"#;

const PLACEMENT: &str = r#"(
    places: [
        (id: "mill", at: (10.0, 0.0), capacity: 3, tags: ["work"]),
        (id: "cottage", at: (-4.0, 2.0), capacity: 2),
    ],
    persons: [
        (name: "Ada", home: Some("cottage"), bound: [("work", "mill")], needs: [("hunger", 0.9)]),
        (name: "Bram", home: Some("cottage"), bound: [("work", "mill")], needs: [("hunger", 0.7)]),
    ],
)"#;

fn village() -> App {
    let mut s: SocietySettings = settings();
    s.seconds_per_day = 4.0;
    let mut app = headless(SocietyDefs::from_ron(DEFS).unwrap(), s);
    PlacementFile::from_ron(PLACEMENT)
        .unwrap()
        .spawn(app.world_mut())
        .unwrap();
    // A live app has run `Startup` (the RNG is seeded there) before anything else touches it.
    app.update();
    app
}

#[test]
fn fast_forward_matches_a_live_run_of_the_same_ticks() {
    let mut fast = village();
    let done = fast_forward(fast.world_mut(), 2.0).unwrap();
    // 2 days at 4 s a day and 1/64 s a tick.
    assert_eq!(done.ticks, 512);
    assert!((done.to_days - done.from_days - 2.0).abs() < 1e-3);

    let mut live = village();
    while live.world().resource::<SocietyClock>().tick < 512 {
        live.update();
    }
    assert_eq!(hash(&fast), hash(&live));

    // Moved on: not the society it started as.
    assert_ne!(hash(&fast), hash(&village()));
}

#[test]
fn fast_forward_twice_equals_once_and_zero_days_is_a_no_op() {
    let mut once = village();
    fast_forward(once.world_mut(), 2.0).unwrap();
    let mut twice = village();
    assert_eq!(fast_forward(twice.world_mut(), 0.0).unwrap().ticks, 0);
    fast_forward(twice.world_mut(), 1.0).unwrap();
    fast_forward(twice.world_mut(), 1.0).unwrap();
    assert_eq!(hash(&once), hash(&twice));
}

#[test]
fn fast_forward_refuses_what_it_cannot_advance() {
    let mut app = village();
    assert_eq!(
        fast_forward(app.world_mut(), -1.0),
        Err(FastForwardError::BadDays(-1.0))
    );
    assert!(matches!(
        fast_forward(app.world_mut(), f32::NAN),
        Err(FastForwardError::BadDays(_))
    ));
    assert_eq!(
        fast_forward(&mut World::new(), 1.0),
        Err(FastForwardError::NoSociety)
    );

    // Before its first update the society is not started.
    let mut fresh = headless(SocietyDefs::from_ron(DEFS).unwrap(), settings());
    assert_eq!(
        fast_forward(fresh.world_mut(), 1.0),
        Err(FastForwardError::NotStarted)
    );

    let mut s = settings();
    s.clock = ClockSource::Host;
    let mut host = headless(SocietyDefs::from_ron(DEFS).unwrap(), s);
    host.update();
    assert_eq!(
        fast_forward(host.world_mut(), 1.0),
        Err(FastForwardError::HostClock)
    );
}

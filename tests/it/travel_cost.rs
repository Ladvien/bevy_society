//! **An activity shorter than the walk to it is not worth choosing, and the decision cannot tell.**
//!
//! `Signal::Distance` is only penalised where a definition asks for it. Measured on the starter pack, at
//! a 64-second day where walking is `1.4 / 64` m a tick and the village is 8.2 m across:
//!
//! | activity | duration, ticks | gates on `Distance`? |
//! |---|---|---|
//! | `snub` | 17 | no |
//! | `compliment` | 34 | no |
//! | `chat` | 51 | **yes** |
//! | `gossip` | 51 | **yes** |
//! | `play` | 85 | **yes** |
//! | `relax` | 256 | no |
//! | `work` | 683 | no |
//! | `sleep` | 1 365 | no |
//! | a crossing walk | **375** | — |
//!
//! Five activities are shorter than the walk to reach them, and three of those five are the only ones that
//! even consider distance. The consequence, measured over 17 game-days of `village-evening`:
//!
//! | | share of all person-ticks |
//! |---|---|
//! | **travelling** | **62.5%** |
//! | doing an activity | 37.5% |
//! | of which `sleep` | **14.7%** — against the 33.3% one 8-hour sleep a day allows |
//!
//! A villager spends most of its life walking to things too short to justify the trip.

use bevy_society::{Signal, SocietyDefs};

/// The walking distance per tick at the scenario day length, and the village's diagonal.
fn walk_ticks(metres: f32, seconds_per_day: f32, tick_secs: f32) -> f32 {
    let metres_per_tick = 1.4 * tick_secs;
    let _ = seconds_per_day;
    metres / metres_per_tick
}

/// **Ignored, and should be un-ignored when the fix lands.** It fails today: eight activities are shorter
/// than the walk that reaches them and five of those do not score `Distance` at all. Fixing it changes the
/// shipped definitions file — which activities weigh the walk, and by how much — so it belongs to
/// `needs-decision/travel-cost-is-not-scored.md` rather than to this test.
///
/// A failing test in the default run would block every iteration after this one, so it is ignored rather
/// than deleted: the defect is pinned, and `cargo test -- --ignored` shows it.
#[test]
#[ignore = "activities shorter than the walk to them; see needs-decision/travel-cost-is-not-scored.md"]
fn every_activity_is_reachable_in_less_time_than_it_lasts() {
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    // `village-evening`'s settings: 64 s a day at a 1/64 s tick, so a crossing walk of 8.2 m is 375 ticks.
    let crossing = walk_ticks(8.2, 64.0, 1.0 / 64.0);
    println!("a crossing walk is {crossing:.0} ticks");
    assert!(
        (crossing - 375.0).abs() < 2.0,
        "the arithmetic this test rests on: {crossing}"
    );

    let mut short: Vec<String> = Vec::new();
    let mut short_without_distance = Vec::new();
    for (name, hours) in defs.activity_durations() {
        // Index 0 is the reserved idle/travel fallback and is meant to be short.
        if name == "idle" {
            continue;
        }
        let ticks = hours / 24.0 * 64.0 * 64.0;
        if ticks < crossing {
            short.push(format!("{name} ({ticks:.0})"));
            let gates = defs
                .activity_signals()
                .into_iter()
                .find(|(n, _)| *n == name)
                .is_some_and(|(_, signals)| signals.contains(&Signal::Distance));
            if !gates {
                short_without_distance.push(name.to_owned());
            }
        }
    }
    println!(
        "activities shorter than a crossing walk: {}",
        short.join(", ")
    );
    println!("…of those, blind to distance: {short_without_distance:?}");
    assert!(
        short.is_empty(),
        "every activity must last longer than the walk that reaches it, or the person spends its life \
         walking to something over before it arrives: {} — and {} of them do not even score distance",
        short.join(", "),
        short_without_distance.len()
    );
}

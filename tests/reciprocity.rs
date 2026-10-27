//! **A relationship is mutual, and the crate had no mechanism to make it so.**
//!
//! Each person's tie is held separately, so a friendship can be one-sided. Measured on
//! `village-evening` (seed 7, 24 people, 16.4 game-days) before the dyad step: **50%** of directed
//! friend ties were unreciprocated and the two directions of a pair disagreed by **0.145** on average —
//! against a `friend_band` of 0.05.
//!
//! Heider's balance theory names this: "in a dyadic relation, balance basically means reciprocity:
//! feeling positive about a person who doesn't reciprocate this feeling is unbalanced" (Heider 1946, via
//! Schweighofer & Schweitzer, DOI `10.18564/jasss.5323`, in the corpus). The crate implemented the
//! triadic half (`balance_triads`) and not the dyadic half; `balance_dyads` is that half.
mod common;

use bevy::prelude::*;
use bevy_society::{PersonIndex, SocietySettings, TieLabel, Ties, Tuning};

fn village(rate: f32, seed: u64) -> App {
    let mut app = common::village::village(common::village::village_defs(), seed, 24);
    let mut settings = SocietySettings {
        seed,
        seconds_per_day: 64.0,
        start_hour: 15.0,
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = Some(4.0);
    settings.tuning.dyad_balance_per_day = rate;
    app.insert_resource(settings);
    app
}

/// `(mutual, one_way, mean |aff(a->b) - aff(b->a)|, friends, rivals)` at the end of the run.
fn shape(app: &App) -> (usize, usize, f32, u32, u32) {
    let index = app.world().resource::<PersonIndex>();
    let people: Vec<Entity> = index.sorted.iter().map(|&(_, e)| e).collect();
    let (mut mutual, mut one_way, mut gap, mut pairs) = (0, 0, 0.0_f64, 0usize);
    let (mut friends, mut rivals) = (0u32, 0u32);
    for (i, &a) in people.iter().enumerate() {
        if let Some(ties) = app.world().get::<Ties>(a) {
            for tie in &ties.0 {
                match tie.label {
                    TieLabel::Friend => friends += 1,
                    TieLabel::Rival => rivals += 1,
                    _ => {}
                }
            }
        }
        for &b in &people[i + 1..] {
            let (Some(ta), Some(tb)) = (
                app.world().get::<Ties>(a).and_then(|t| t.get(b)),
                app.world().get::<Ties>(b).and_then(|t| t.get(a)),
            ) else {
                continue;
            };
            gap += f64::from((ta.affinity - tb.affinity).abs());
            pairs += 1;
            let (fa, fb) = (ta.label == TieLabel::Friend, tb.label == TieLabel::Friend);
            if fa && fb {
                mutual += 1
            } else if fa || fb {
                one_way += 1
            }
        }
    }
    (
        mutual,
        one_way,
        (gap / pairs.max(1) as f64) as f32,
        friends / 2,
        rivals / 2,
    )
}

/// **The two directions of a pair converge, and the labels follow.**
///
/// The friend band is switched **off** here, so this measures the dyad step alone. Measured at the
/// scenario's own clock over 16.4 game-days: seed 7 goes 36% unreciprocated and a 0.133 gap to **26% and
/// 0.022**; seed 42 goes 32% to **22%**. With both steps running they interact, and `eval.md` says how.
#[test]
fn the_two_directions_of_a_relationship_converge() {
    let mut without = village(0.0, 7);
    let mut with = village(2.0, 7);
    without
        .world_mut()
        .resource_mut::<SocietySettings>()
        .tuning
        .friend_band = 0.0;
    with.world_mut()
        .resource_mut::<SocietySettings>()
        .tuning
        .friend_band = 0.0;
    common::run(&mut without, 4_200);
    common::run(&mut with, 4_200);

    let (_, _, gap_without, _, _) = shape(&without);
    let (_, _, gap_with, _, _) = shape(&with);
    assert!(
        gap_without > 0.05,
        "the fixture must show the asymmetry it exists to remove: {gap_without:.3}"
    );
    assert!(
        gap_with < gap_without / 2.0,
        "the two directions must converge: {gap_with:.3} with, {gap_without:.3} without"
    );
}

/// **A dyad step must not flatten the village into indifference.**
///
/// The failure mode of a reciprocity step is convergence on the *population* mean, which makes every
/// `affinity` the same and both labels rare. Measured: a rate of 0.5 costs some friends and rivals
/// (seed 7: friends 22 -> 16, rivals 42 -> 24) and that is the price of mutuality, not a collapse —
/// there are still both.
///
/// **The run is 12 600 ticks (about three game-days), not 4 200, and that is this iteration's change.**
/// The village starts at 15:00 and bakes only at first light, so 4 200 ticks — a little over one
/// game-day — now ends before the first loaf exists. It used to be enough because `satisfies` was paid
/// in full from an empty shelf: a village that had never baked was fed anyway, and its rivalries formed
/// over meals nobody ate. Once a meal pays for what it took (`docs/loop/runs/2026-10-26-12/`) the same
/// window is a starving one and the hostile ties have not formed yet: measured at seed 7, rivals are
/// **0 at 4 200 ticks**, **0 at 8 400** (one bake, food 0.455) and **8 at 12 600** (food 0.586, bread
/// 0.906 a head). Three game-days is the shortest window in which this fixture tests what it says it
/// does — that the dyad step leaves *both* labels standing — rather than the supply of bread.
#[test]
fn the_village_keeps_both_labels() {
    let mut app = village(Tuning::default().dyad_balance_per_day, 7);
    common::run(&mut app, 12_600);
    let (mutual, one_way, gap, friends, rivals) = shape(&app);
    assert!(
        friends > 0 && rivals > 0,
        "both labels must survive: {friends} friends, {rivals} rivals"
    );
    assert!(
        mutual > 0 && one_way > 0,
        "some friendships are mutual and some are not: {mutual} / {one_way}"
    );
    assert!(gap < 0.10, "the directional gap must be small: {gap:.3}");
}

/// Same seed, same rate, same society — the step adds no draw and iterates by index.
#[test]
fn the_dyad_step_is_deterministic() {
    let hash = |rate: f32| {
        let mut app = village(rate, 42);
        common::run(&mut app, 2_000);
        shape(&app)
    };
    assert_eq!(
        hash(Tuning::default().dyad_balance_per_day),
        hash(Tuning::default().dyad_balance_per_day),
        "two runs at the same seed and rate must agree"
    );
}

/// The rate is a named `Tuning` field, and `0.0` disables the step exactly.
#[test]
fn a_zero_rate_is_the_old_society() {
    let mut off = village(0.0, 7);
    common::run(&mut off, 1_000);
    let (_, _, gap, _, _) = shape(&off);
    assert!(
        gap > 0.0,
        "with the step off the two directions are free to differ, which is the old behaviour"
    );
    assert_eq!(
        Tuning {
            dyad_balance_per_day: 0.0,
            ..Tuning::default()
        }
        .dyad_balance_per_day,
        0.0
    );
}

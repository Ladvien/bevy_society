//! Determinism (plan D12): the same seed and the same spawn sequence give the same society, bit for
//! bit, and a different seed gives a different one.
//!
//! The hash itself is `common::hash`, shared with `tests/save.rs`: the save test's claim — that the
//! 100 ticks after a restore are the 100 ticks that would have followed without one — is a
//! determinism claim about the same society, so the two must measure with one instrument.

use crate::common;

use bevy::prelude::*;
use bevy_society::SocietyClock;
use common::hash::hash;

fn run(app: &mut App, ticks: usize) -> u64 {
    for _ in 0..ticks {
        app.update();
    }
    // A society that never ticked would hash identically under any seed, so the clock is part of the
    // evidence that this test measured anything at all.
    assert!(
        app.world().resource::<SocietyClock>().tick > (ticks as u64) / 2,
        "the society must actually advance"
    );
    hash(app)
}

#[test]
fn the_same_seed_gives_the_same_society() {
    let mut first = common::village::village(common::village::village_defs(), 7, 24);
    let mut second = common::village::village(common::village::village_defs(), 7, 24);
    let a = run(&mut first, 3000);
    let b = run(&mut second, 3000);
    assert_eq!(
        a, b,
        "the same seed and spawn sequence give the same society"
    );
    assert_ne!(a, 0, "the hash covers a non-empty society");
}

#[test]
fn a_different_seed_gives_a_different_society() {
    // The seed reaches the hashed state through the per-tick draws: personality sampling at spawn
    // and, once the behaviour plugin decides activities, one draw per decision. Until then two
    // villages of the same spawn sequence hash alike, so this asserts the whole path is seeded.
    let mut seven = common::village::village(common::village::village_defs(), 7, 24);
    let mut eight = common::village::village(common::village::village_defs(), 8, 24);
    let a = run(&mut seven, 3000);
    let b = run(&mut eight, 3000);
    assert_ne!(a, b, "the seed is part of the society");
}

//! **Exposure's equilibrium must be measured against the friendship threshold, not chosen in isolation.**
//!
//! `mere_exposure_per_day` and `affinity_relax_per_day` are two first-order processes on one variable, so
//! affinity settles where they balance:
//!
//! ```text
//! equilibrium = exposure / (exposure + relax) = 0.05 / (0.05 + 0.01) = 0.833
//! ```
//!
//! `TieLabel::classify` calls a tie a **Friend** at affinity `0.4` — less than half that equilibrium. So a
//! rate that looks unremarkable on its own makes **proximity sufficient for friendship**, and the two rates
//! were never checked against each other.
//!
//! Two things this file deliberately does **not** claim. First, that exposure is what sets a village's friend
//! count: measured, lowering it to 0.02, 0.01 and 0.005 moved friends only **22 → 15 → 15 → 16**. What does
//! is the number of *exchanges* — a pair becomes a Friend after **45** of them, with no diminishing return,
//! which is why two separate mechanisms that added contact each produced a clique. Second, that the
//! equilibrium is a bug: `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` records the three
//! ways to change it, and none is decided here.
//!
//! What is worth pinning is the *arithmetic*, because it is invisible from either rate alone and a future
//! change to one of them silently moves it.
use bevy_society::{TieLabel, Tuning};

#[test]
fn exposure_equilibrium_is_stated_and_compared_with_the_friend_threshold() {
    let tuning = Tuning::default();
    let exposure = tuning.mere_exposure_per_day;
    let relax = tuning.affinity_relax_per_day;
    let equilibrium = exposure / (exposure + relax);
    println!("exposure {exposure}, relax {relax} -> equilibrium affinity {equilibrium:.3}");

    // The two rates really are first-order and opposite in sign, so the quotient is the fixed point.
    assert!(exposure > 0.0 && relax > 0.0);

    // The equilibrium is a **fixed point**: at that value exposure adds exactly what relaxation removes, so
    // the tie stops moving. That is arithmetic, true whatever the rates are, and it is what neither rate
    // reveals on its own.
    let added = exposure * (1.0 - equilibrium);
    let removed = relax * equilibrium;
    assert!(
        (added - removed).abs() < 1e-6,
        "exposure adds {added:.6} and relaxation removes {removed:.6} at affinity {equilibrium:.3}; they must \
         balance for this to be the equilibrium"
    );

    // What the equilibrium *means* for the Friend threshold is recorded, not asserted: whether proximity alone
    // should suffice for friendship is `needs-decision/friendship-has-no-diminishing-return.md`'s question, and
    // pinning it here would block the fix that file recommends.
    let at_equilibrium = TieLabel::classify(equilibrium, 1.0, &tuning);
    println!(
        "a tie resting at the equilibrium reads {at_equilibrium:?}; the 0.4 Friend threshold is {}% below it",
        100.0 * (1.0 - 0.4 / equilibrium)
    );
    assert_eq!(
        TieLabel::classify(1.0, 1.0, &tuning),
        TieLabel::Friend,
        "the thresholds must still admit a maximally-affine, long-known tie as a Friend"
    );
}

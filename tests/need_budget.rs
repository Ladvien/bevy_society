//! **A need nothing can supply is named at startup.**
//!
//! Every need decays per game-day; activities pay `satisfies · duration_hours` per use. A need whose
//! best source pays less than a quarter of its decay needs more than four uses a day, which no activity
//! shape makes plausible — especially one gated to a few hours or behind a walk.
//!
//! Measured on the starter pack, in a 24-person village over 32.8 game-days: `fun` decays 0.8/day, its
//! only source `relax` pays 0.6 per use, and a person needs **1.33** of them a day while managing
//! **0.10**. `fun` sits at zero for a third of every run.
//!
//! **Raising `relax` to fix it was tried and reverted.** At `satisfies 0.55` (0.825 per use, 0.97 uses a
//! day) `fun`'s zero fraction halves, 33.8% → 18.8%, but its *mean* falls, 0.061 → 0.051, and every
//! other need falls too: the mean of all four goes 0.127 → 0.118. The village is over-subscribed —
//! 24 people, four places, one evening — so raising one supply only crowds out another. The rate change
//! is reverted and the crate now **names** the imbalance instead of guessing at a number for it.
mod common;

use bevy_society::{DefsFile, SocietyDefs};

#[test]
fn every_need_has_a_source() {
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    let file = DefsFile::starter().expect("the starter pack parses");
    let _ = file;
    for need in ["social", "rest", "food", "fun"] {
        let decay = defs
            .need_decay(need)
            .unwrap_or_else(|| panic!("{need} is defined"));
        let (activity, per_use) = defs
            .best_source(need)
            .unwrap_or_else(|| panic!("{need} must have a source, or it can never be met"));
        let uses_per_day = decay / per_use;
        println!(
            "{need:7} decay {decay:.2}/day, best source {activity:6} pays {per_use:.2}/use \
             -> {uses_per_day:.2} uses/day"
        );
        assert!(per_use > 0.0, "{need}'s best source must supply something");
    }
}

#[test]
fn the_budget_exposes_the_one_need_that_starves() {
    // The crate's own warning threshold is four uses a day. `social` needs 2.78 and gets them, because
    // a chat targets a `Person` and costs three ticks. This asserts the arithmetic, not a behaviour:
    // the behaviour is in `tests/needs.rs` and the evidence dir.
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    let fun_decay = defs.need_decay("fun").expect("fun is defined");
    let (_, per_use) = defs.best_source("fun").expect("fun has a source");
    let uses_per_day = fun_decay / per_use;
    println!("fun needs {uses_per_day:.2} relaxes a day; measured supply is 0.10");
    assert!(
        uses_per_day > 1.0,
        "if this ever becomes <= 1, one evening session covers a day and the imbalance is gone"
    );
}

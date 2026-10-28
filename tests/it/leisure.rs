//! **`fun` needs a source that is cheap to reach, not just a bigger number.**
//!
//! The starter pack's only leisure is `relax`: `Place("gathering")`, 1.5 h, gated 17:00–23:00. It pays
//! 0.6 per use against `fun`'s 0.8/day decay, so a person needs 1.33 uses a day; measured supply in a
//! 24-person village is **0.10**, and `fun` sits at zero for a third of every run.
//!
//! Raising `relax`'s rate was tried and reverted: it halves `fun`'s zero fraction (33.8% → 18.8%) but
//! **lowers** the mean of all four needs (0.127 → 0.118), because the village is over-subscribed and one
//! supply only crowds out another.
//!
//! The difference between `fun` and `social` is *cost per use*, not rate. `social` needs 2.78 chats a day
//! and gets them: `chat` targets a `Person`, so the other person walks to the actor and a use is three
//! ticks. `relax` costs a walk plus sixteen. Self-determination theory says intrinsic motivation arises
//! in need-supportive social contexts (Vansteenkiste & Ryan 2013, DOI 10.1037/a0032359) — doing something
//! enjoyable *with someone*, not sitting alone at a well.
//!
//! So the starter pack gains `play`: a `Person`-targeted leisure activity, modelled on `compliment`'s
//! cost and reachability and sized to `fun`'s decay.

use bevy_society::SocietyDefs;

#[test]
fn a_cheap_source_of_fun_exists_alongside_relax() {
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    let decay = defs.need_decay("fun").expect("fun is defined");
    let (best, per_use) = defs.best_source("fun").expect("fun has a source");
    let uses_per_day = decay / per_use;
    println!("best source for fun is {best}, {per_use:.3}/use -> {uses_per_day:.2} uses/day");

    assert!(
        uses_per_day <= 1.0,
        "one use of {best} must cover a day of fun decay, so an evening does not need 1.33 outings: \
         {uses_per_day:.2} uses/day"
    );

    // And `fun` must have a second source, so an evening is not a single exclusive choice.
    let sources = defs.sources_for("fun");
    println!("activities satisfying fun: {sources:?}");
    assert!(
        sources.len() >= 2,
        "fun needs a second source, so an evening is not a single exclusive choice: {sources:?}"
    );
}

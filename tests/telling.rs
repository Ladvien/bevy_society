//! **A rumour you just heard must be tellable, even when an older equal one competes.**
//!
//! `Knowledge::best_telling` picks the maximum of `|valence| · belief · 0.995^(hours since heard)` and,
//! its own doc says, *"ties go to the lowest `RumorId`"*. A freshly planted rumour therefore loses every
//! tie to an older one of the same valence — and in a village that has been running, older equal-valence
//! rumours are exactly what a teller holds.
//!
//! Measured, 24 people at `village-evening`'s own settings over 17 game-days: a `steal` planted with
//! valence −0.7 among 13 active gossipers reached **0 of 24**. Its weight was 0.700 at age zero; the
//! tellers held older `insult`s at valence −0.4 and `steal`s at −0.7, and the −0.7 ones tied and won on
//! the lower id.
//!
//! The weight's own recency term is what the tie-break should honour: the rumour heard most recently is
//! the one worth telling. Ties on weight now go to the **later** `heard_day`, and only a tie there falls
//! back to the lower id so the result stays deterministic.
mod common;

use bevy_society::{Knowledge, Known, RumorId};

fn known(rumor: u32, belief: f32, heard_day: f32) -> Known {
    Known {
        rumor: RumorId(rumor),
        belief,
        spreader: true,
        heard_day,
    }
}

#[test]
fn a_fresh_rumor_beats_an_older_one_of_equal_weight() {
    let mut knowledge = Knowledge::default();
    // An old steal the teller has held for days, and a fresh one heard this morning. Both have
    // valence -0.7 and belief 1.0, so their weights differ only through recency — and at one game-day
    // of age the decay factor is 0.995^24 = 0.887, which makes the fresh one heavier, not a tie.
    knowledge.insert(known(0, 1.0, 0.0), usize::MAX, 10.0);
    knowledge.insert(known(1, 1.0, 9.9), usize::MAX, 10.0);
    let pick = knowledge.best_telling(10.0, |_| Some(-0.7));
    assert_eq!(
        pick,
        Some(RumorId(1)),
        "the rumour heard most recently must win: an old equal one should not"
    );
}

#[test]
fn and_the_result_is_still_deterministic_when_everything_ties() {
    // Same valence, same belief, same heard day: only then does the id decide, and the lower one wins
    // as before, so two runs of the same society tell the same rumour.
    let mut knowledge = Knowledge::default();
    knowledge.insert(known(7, 1.0, 5.0), usize::MAX, 5.0);
    knowledge.insert(known(3, 1.0, 5.0), usize::MAX, 5.0);
    let pick = knowledge.best_telling(5.0, |_| Some(-0.5));
    assert_eq!(
        pick,
        Some(RumorId(3)),
        "a true tie falls back to the lower id"
    );
}

#[test]
fn an_older_but_stronger_rumor_still_wins() {
    // Recency must not override strength: a valence -0.9 rumour from yesterday beats a -0.1 one from
    // an hour ago, because `|valence|` is the dominant term in the weight.
    let mut knowledge = Knowledge::default();
    knowledge.insert(known(0, 1.0, 0.0), usize::MAX, 10.0);
    knowledge.insert(known(1, 1.0, 9.9), usize::MAX, 10.0);
    let pick = knowledge.best_telling(10.0, |rumor| {
        Some(if rumor == RumorId(0) { -0.9 } else { -0.1 })
    });
    assert_eq!(
        pick,
        Some(RumorId(0)),
        "the stronger rumour wins despite being older"
    );
}

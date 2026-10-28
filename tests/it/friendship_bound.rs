//! **Friendship stays scarce in a large town** (issue #10), which is the bar Sanctuary needs.
//!
//! Sanctuary's town is about 60 named people who work together for 30 or more game days. Grief and the
//! fear of seeing a known face among the dead both key on who is a `Friend`, so if everyone becomes
//! everyone's friend those reactions stop meaning anything.
//!
//! Five mechanisms were measured against this and none worked (`docs/loop/needs-decision/friendship-has-no-diminishing-return.md`):
//! the threshold (`friend_affinity`) never discriminated because affinity reached 1.0 and passed every
//! bar; a saturating interaction term was nearly neutral; a `trust` gate alone did nothing; a
//! contact-triggered exchange turned the village into a clique; and a lower `mere_exposure_per_day` moved
//! friends only 22 → 16. The cause is a **saturating system with a high asymptote**, so the lever is the
//! asymptote: `Tuning::affinity_ceiling` (0.35) bounds what contact can reach, and it sits **below** the
//! `friend_affinity` bar (0.30 is below the ceiling only because the ceiling is what affinity reaches —
//! a friend therefore also needs `friend_trust` 0.5 to pass `TieLabel::classify_with_trust`).
//!
//! **Release-only**, like `tests/it/rumor_book.rs`: 30 game-days at 60 people is minutes in a debug build and
//! the measurement only means something optimised.
//!
//! ```sh
//! cargo test --release --test it friendship_bound::
//! ```

#![cfg(not(debug_assertions))]

use crate::common;

use bevy_society::{PersonIndex, TieLabel, Ties};

/// Sixty people, seed 7, thirty game-days at the crate's default day.
#[test]
fn sixty_people_over_thirty_days_keep_friendship_scarce() {
    let defs = common::village::village_defs();
    let mut app = common::village::village(defs, 7, 60);

    let settings = app
        .world()
        .resource::<bevy_society::SocietySettings>()
        .clone();
    // Thirty game-days of society ticks. One tick is `tick_secs` of fixed-step time.
    let ticks = (30.0 * settings.seconds_per_day / settings.tick_secs).round() as usize;
    for _ in 0..ticks {
        app.update();
    }

    let index = app.world().resource::<PersonIndex>().clone();
    let people = index.sorted.len();
    let mut friends = 0usize;
    let mut rivals = 0usize;
    for &(_, entity) in &index.sorted {
        let Some(ties) = app.world().get::<Ties>(entity) else {
            continue;
        };
        for tie in &ties.0 {
            match tie.label {
                TieLabel::Friend => friends += 1,
                TieLabel::Rival => rivals += 1,
                _ => {}
            }
        }
    }
    let ordered_pairs = people * people.saturating_sub(1);
    let friend_fraction = friends as f64 / ordered_pairs.max(1) as f64;

    println!(
        "friendship_bound: {people} people, {ticks} ticks ({:.1} game-days), \
         {friends} friend ties of {ordered_pairs} ordered pairs ({:.1}%), {rivals} rival ties",
        ticks as f64 * f64::from(settings.tick_secs) / f64::from(settings.seconds_per_day),
        friend_fraction * 100.0,
    );

    // The three acceptance criteria of issue #10.
    assert!(
        friend_fraction < 0.25,
        "friendship must stay scarce: {friends} of {ordered_pairs} ordered pairs is {:.1}%, over 25%",
        friend_fraction * 100.0
    );
    assert!(
        friends >= 6,
        "friendship must still happen: at least one friend tie per ten persons needs 6 at 60 people, \
         got {friends}"
    );
    assert!(
        rivals >= 1,
        "the network must not be inverted: rivals dropped to 0"
    );
}

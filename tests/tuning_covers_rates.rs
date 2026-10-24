//! **Every rate is a named field on `Tuning`** — the crate says so in its README, and two are not.
//!
//! `README.md`, "What you get":
//!
//! > Every formula traces to a paper; **every rate is a named field on `Tuning`**; every default either comes
//! > from a paper or says it is a design default.
//!
//! Two literals in non-test code break that promise, and both are things a host would want to change:
//!
//! | literal | where | what it does |
//! |---|---|---|
//! | `0.4`, `0.3`, `0.2`, `0.1` | `TieLabel::classify` | when a tie becomes a friend, rival, stranger |
//! | `0.05` | `resolve_interactions`, the rejection penalty | affinity lost when an approach is refused |
//!
//! The four `classify` thresholds are documented as "design defaults (no source paper gives them)" — which is
//! exactly the case the rule exists for: a default nobody can source must at least be visible in one place and
//! adjustable in one place. Measured, they are also consequential: `familiarity >= 0.3` is satisfied by
//! **0.907** for a friend and **0.908** for everyone else, so that term discriminates nothing, and no host can
//! currently find that out except by reading the source.
//!
//! This file is about the *surface*, not the values. Nothing here says 0.4 is wrong.
use bevy_society::{TieLabel, Tuning};

#[test]
fn the_label_thresholds_are_on_tuning_and_the_defaults_are_unchanged() {
    let tuning = Tuning::default();

    // The defaults the literals had, now reachable and named.
    assert!(
        (tuning.friend_affinity - 0.4).abs() < 1e-6,
        "the friend affinity default is {}; it was a literal 0.4",
        tuning.friend_affinity
    );
    assert!((tuning.friend_familiarity - 0.3).abs() < 1e-6);
    assert!((tuning.rival_affinity - 0.4).abs() < 1e-6);
    assert!((tuning.rival_familiarity - 0.2).abs() < 1e-6);
    assert!((tuning.stranger_familiarity - 0.1).abs() < 1e-6);

    // `classify` reads them, so a host that raises the bar gets a different label for the same tie.
    assert_eq!(TieLabel::classify(0.45, 0.5, &tuning), TieLabel::Friend);
    let strict = Tuning {
        friend_affinity: 0.9,
        ..Tuning::default()
    };
    assert_eq!(
        TieLabel::classify(0.45, 0.5, &strict),
        TieLabel::Acquaintance,
        "raising `friend_affinity` must demote a tie that was a friend, or the field is not being read"
    );
}

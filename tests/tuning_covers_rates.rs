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

    // **The defaults moved with the clamp** (issue #10). `friend_affinity` and `rival_affinity` sat at
    // 0.4, which is above `affinity_ceiling` (0.35) and therefore unreachable once affinity is bounded —
    // measured, that produced 155 friend ties and **0 rivals**. Both are 0.30 now, and the point of this
    // test is that they are *read*, not that they are any particular number.
    assert!(
        (tuning.affinity_ceiling - 0.35).abs() < 1e-6,
        "the affinity ceiling default is {}",
        tuning.affinity_ceiling
    );
    assert!(
        tuning.friend_affinity <= tuning.affinity_ceiling,
        "the friend bar ({}) must be reachable under the ceiling ({})",
        tuning.friend_affinity,
        tuning.affinity_ceiling
    );
    assert!(
        tuning.rival_affinity <= tuning.affinity_ceiling,
        "and so must the rival bar ({})",
        tuning.rival_affinity
    );
    assert!((tuning.friend_familiarity - 0.3).abs() < 1e-6);
    assert!((tuning.rival_familiarity - 0.2).abs() < 1e-6);
    assert!((tuning.stranger_familiarity - 0.1).abs() < 1e-6);
    assert!((tuning.friend_trust - 0.5).abs() < 1e-6);

    // `classify_with_trust` reads them, so a host that raises the bar gets a different label for the
    // same tie.
    assert_eq!(
        TieLabel::classify_with_trust(0.45, 0.5, 0.8, &tuning),
        TieLabel::Friend
    );
    assert_eq!(
        TieLabel::classify_with_trust(0.45, 0.5, 0.1, &tuning),
        TieLabel::Acquaintance,
        "affinity and familiarity are not enough without trust"
    );
    let strict = Tuning {
        friend_affinity: 0.9,
        ..Tuning::default()
    };
    assert_eq!(
        TieLabel::classify_with_trust(0.45, 0.5, 0.8, &strict),
        TieLabel::Acquaintance,
        "raising `friend_affinity` must demote a tie that was a friend, or the field is not being read"
    );

    // `label_hysteresis` is read too, and it is the fifth label number a host can turn. A tie that is
    // already an Acquaintance keeps the label inside the band and loses it past it; with no band the two
    // are the same test. Measured, the default removes the village's low-end churn outright without
    // moving a single friend or rival count.
    let at_bar = tuning.stranger_familiarity;
    assert!(
        tuning.label_hysteresis > 0.0,
        "the default must have a band, or `Stranger <-> Acquaintance` flips on a 0.0002 decay step again"
    );
    let inside = TieLabel::classify_from(
        TieLabel::Acquaintance,
        0.0,
        at_bar - tuning.label_hysteresis / 2.0,
        0.5,
        &tuning,
    );
    assert_eq!(
        inside,
        TieLabel::Acquaintance,
        "the band holds at the low end"
    );
    let outside = TieLabel::classify_from(
        TieLabel::Acquaintance,
        0.0,
        at_bar - tuning.label_hysteresis - 0.01,
        0.5,
        &tuning,
    );
    assert_eq!(outside, TieLabel::Stranger, "and releases past it");
    let none = Tuning {
        label_hysteresis: 0.0,
        ..Tuning::default()
    };
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Acquaintance,
            0.0,
            at_bar - tuning.label_hysteresis / 2.0,
            0.5,
            &none
        ),
        TieLabel::Stranger,
        "`0.0` must restore the single bar for a host that wants the old behaviour"
    );
}

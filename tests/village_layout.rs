//! **The village's layout is the same everywhere it is built.**
//!
//! The project builds its village in three places: `examples/common/scene.rs` (canonical — the capture frames
//! from it), `examples/evidence.rs` (mirrors it, and is what every measured run reports), and
//! `tests/common/village.rs` (what the tests assert against). They had **drifted**: the test builder stood its
//! tavern at `(8.0, -6.0)` while the other two use `(6.5, -5.0)`, making the farthest place pair **21.3 m**
//! instead of **16.4 m**. A test built on that layout was asserting things about a village nobody watches or
//! measures.
//!
//! The drift also changed the arithmetic a host cares about. Reachable distance is
//! `speed * seconds_per_day * travel_timeout_hours / 24`, so at the `village-evening` scenario's 64 s/day:
//!
//! | layout | farthest pair | needs `travel_timeout_hours` | declared |
//! |---|---|---|---|
//! | canonical | 16.4 m | 4.4 | 4.0 |
//! | drifted | 21.3 m | 5.7 | 4.0 |
//!
//! Both are short of what they need, but by different amounts — which is precisely why the numbers must not be
//! allowed to drift again. `tests/` cannot import `examples/`, so this file pins the literal values and the
//! distance between them.
mod common;

use bevy_society::SocietyDefs;

/// The canonical layout, repeated here because `tests/` cannot reach `examples/`.
///
/// If these fail, the fix is to make whichever copy is wrong match `examples/common/scene.rs` — never to edit
/// this table to match a drifted copy.
const CANONICAL: &[(&str, f32, f32, u32)] = &[
    ("well", 0.0, 0.0, 6),
    ("tavern", 6.5, -5.0, 10),
    ("market", -6.5, -5.0, 8),
    ("workshop", 6.5, 5.0, 6),
];

#[test]
fn the_test_village_stands_where_the_captured_village_stands() {
    let shared = common::village::PLACES;
    assert_eq!(
        shared.len(),
        CANONICAL.len(),
        "the shared village has {} places, the captured village has {}",
        shared.len(),
        CANONICAL.len()
    );
    for (built, canonical) in shared.iter().zip(CANONICAL) {
        let (name, position, tags, capacity) = *built;
        let (want_name, x, y, want_capacity) = *canonical;
        assert_eq!(
            name, want_name,
            "place order differs: {name} against {want_name}"
        );
        assert!(
            (position.x - x).abs() < f32::EPSILON && (position.y - y).abs() < f32::EPSILON,
            "'{name}' stands at ({}, {}), the captured village puts it at ({x}, {y}). A test on this layout \
             would be asserting things about a village nobody watches",
            position.x,
            position.y
        );
        assert_eq!(capacity, want_capacity, "'{name}' capacity differs");
        assert!(
            !tags.is_empty(),
            "'{name}' has no tags, so no activity can find it"
        );
    }
}

#[test]
fn the_village_is_walkable_at_its_own_scenario() {
    // The arithmetic a host is exposed to: at `village-evening`'s 64 s/day, a 4-hour timeout buys 14.9 m. The
    // canonical village's farthest pair is 16.4 m, which needs 4.4 — so the scenario is short by 0.4 hours and
    // the crate says so at startup (`warn_unreachable_places`). This pins the *distance* so a future layout
    // change cannot silently make it far worse.
    let mut worst = 0.0_f32;
    for (_, a, b, _) in CANONICAL {
        for (_, c, d, _) in CANONICAL {
            let distance = ((a - c).powi(2) + (b - d).powi(2)).sqrt();
            worst = worst.max(distance);
        }
    }
    assert!(
        (worst - 16.4).abs() < 0.1,
        "the farthest place pair is {worst:.1} m; the captured village measures 16.4 m. If the layout moved \
         deliberately, update the measurement in `examples/common/scene.rs` and this figure together"
    );
    // And the shared builder must agree, which is what stops a test from measuring a different village.
    let mut shared_worst = 0.0_f32;
    for (_, a, _, _) in common::village::PLACES {
        for (_, b, _, _) in common::village::PLACES {
            shared_worst = shared_worst.max(a.distance(*b));
        }
    }
    assert!(
        (shared_worst - worst).abs() < 0.1,
        "the shared test village spans {shared_worst:.1} m but the canonical one spans {worst:.1} m"
    );
}

#[test]
fn the_villages_own_definitions_compile() {
    // `common::village::village_defs` merges the starter pack with `examples/assets/village.society.ron`. If
    // that file stops agreeing with the pack, every village test fails for a reason unrelated to its subject.
    let defs: SocietyDefs = common::village::village_defs();
    // A definition the village adds must be resolvable, which is what proves the merge did anything.
    assert!(
        defs.activity_id("work").is_some(),
        "the compiled village must resolve its  activity"
    );
}

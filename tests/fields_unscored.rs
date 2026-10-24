//! Field inputs are accepted by the definitions file and not scored by the decision. These tests pin
//! the facts behind the startup warning, so the warning cannot go stale without a failure.
#![cfg(feature = "fields")]

use bevy_society::{ActivityTarget, Signal, SocietyDefs};

/// A definition using both a `Field` consideration and a `Gradient` target.
fn field_defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0)],
            activities: [(
                id: "follow",
                target: Gradient(Crowd, true),
                duration_hours: 1.0,
                considerations: [(input: Field(Crowd), curve: Linear(1.0, 0.0))],
            )],
        )"#,
    )
    .expect("a Gradient target and a Field input compile with the feature on")
}

#[test]
fn a_field_definition_is_accepted() {
    // If this ever starts failing, the compiler rejected the definition and the warning is moot.
    let defs = field_defs();
    assert!(
        defs.activity_durations()
            .iter()
            .any(|(name, _)| *name == "follow"),
        "the Gradient activity compiled"
    );
}

#[test]
fn the_activity_reports_its_field_signal() {
    // What the startup warning keys on: an activity whose considerations read a field.
    let defs = field_defs();
    let offending: Vec<&str> = defs
        .activity_signals()
        .into_iter()
        .filter(|(_, signals)| signals.iter().any(|s| matches!(s, Signal::Field(_))))
        .map(|(name, _)| name)
        .collect();
    assert_eq!(
        offending,
        vec!["follow"],
        "the Field consideration must be visible to the check"
    );
}

#[test]
fn the_activity_reports_its_gradient_target() {
    let defs = field_defs();
    assert!(
        defs.has_gradient_activity(),
        "the Gradient target must be visible to the check"
    );
}

#[test]
fn the_starter_pack_uses_no_field_inputs() {
    // The warning must stay silent for a host on the starter pack, or it is noise.
    let defs = SocietyDefs::starter().expect("the starter pack parses");
    assert!(
        !defs.has_gradient_activity(),
        "the starter pack has no Gradient activity"
    );
    let offenders: Vec<&str> = defs
        .activity_signals()
        .into_iter()
        .filter(|(_, signals)| signals.iter().any(|s| matches!(s, Signal::Field(_))))
        .map(|(name, _)| name)
        .collect();
    assert!(
        offenders.is_empty(),
        "the starter pack must not trigger the warning: {offenders:?}"
    );
    let _ = ActivityTarget::Itself;
}

//! Definitions validation: parsing, merging, id interning and error reporting.

use bevy_society::{DefsError, DefsFile, IDLE_ACTIVITY, SocietyDefs};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn starter_pack_compiles() -> TestResult {
    let defs = SocietyDefs::starter()?;
    assert!(defs.need_id("social").is_some());
    assert!(defs.activity_id("eat").is_some());
    Ok(())
}

#[test]
fn idle_is_pre_registered_at_the_reserved_id() -> TestResult {
    let defs = SocietyDefs::starter()?;
    assert_eq!(defs.activity_id("idle"), Some(IDLE_ACTIVITY));
    assert_eq!(defs.activity_name(IDLE_ACTIVITY), "idle");
    Ok(())
}

#[test]
fn redefining_idle_is_a_duplicate() {
    let text = r#"(activities: [(id: "idle", target: Itself, duration_hours: 1.0)])"#;
    let result = SocietyDefs::from_ron(text);
    assert!(matches!(
        result,
        Err(DefsError::Duplicate { kind: "activity", ref id }) if id == "idle"
    ));
}

#[test]
fn unknown_need_in_consideration_is_reported_with_its_path() {
    let text = r#"(activities: [(
        id: "drink",
        target: Itself,
        duration_hours: 0.5,
        considerations: [(input: Need("thirst"), curve: Linear(-1.0, 1.0))],
    )])"#;
    let result = SocietyDefs::from_ron(text);
    assert!(matches!(
        result,
        Err(DefsError::UnknownId { kind: "need", ref id, ref owner })
            if id == "thirst" && owner == "activity 'drink' consideration 0"
    ));
}

#[test]
fn merging_a_repeated_activity_is_a_duplicate_and_leaves_self_unchanged() -> TestResult {
    let text = r#"(activities: [(id: "eat", target: Itself, duration_hours: 1.0)])"#;
    let mut base = DefsFile::from_ron(text)?;
    let other = DefsFile::from_ron(text)?;
    let merged = base.merge(other);
    assert!(matches!(
        merged,
        Err(DefsError::Duplicate { kind: "activity", ref id }) if id == "eat"
    ));
    // `base` is unchanged: it still compiles and still defines `eat` exactly once.
    let defs = SocietyDefs::compile(base)?;
    assert!(defs.activity_id("eat").is_some());
    Ok(())
}

#[test]
fn script_syntax_error_names_its_trigger() {
    let text = r#"(triggers: [(id: "greet", on: Tick, then: "fx.adjust(actor,")])"#;
    let result = SocietyDefs::from_ron(text);
    assert!(matches!(
        result,
        Err(DefsError::Script { ref owner, .. }) if owner.contains("'greet'")
    ));
}

#[test]
fn bark_with_unknown_placeholder_is_invalid() {
    let text = r#"(activities: [(
        id: "hi",
        target: Itself,
        duration_hours: 1.0,
        barks: ["Hi {nobody}"],
    )])"#;
    let result = SocietyDefs::from_ron(text);
    assert!(matches!(result, Err(DefsError::Invalid { .. })));
}

#[test]
fn affinity_input_on_a_place_activity_is_invalid() {
    let text = r#"(activities: [(
        id: "dine",
        target: Place("food"),
        duration_hours: 1.0,
        considerations: [(input: Affinity, curve: Linear(1.0, 0.0))],
    )])"#;
    let result = SocietyDefs::from_ron(text);
    assert!(matches!(result, Err(DefsError::Invalid { .. })));
}

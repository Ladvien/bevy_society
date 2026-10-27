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

#[test]
fn a_merged_pack_contributes_its_goods() -> TestResult {
    // **A merge carried eleven lists and dropped two.** `goods` and `skills` were the two, so a pack merged
    // in with a new good contributed nothing — and because `compile` resolves every `consumes` and `produces`
    // by name, the failure surfaced as "not defined" against the *consuming* activity rather than against
    // the merge. The village example is exactly this shape: the starter pack plus `village.society.ron`.
    let mut base = DefsFile::from_ron(r#"(goods: [(id: "bread")])"#)?;
    let other =
        DefsFile::from_ron(r#"(goods: [(id: "wool")], skills: [(id: "weave", group: Craft)])"#)?;
    base.merge(other)?;
    let defs = SocietyDefs::compile(base)?;
    assert!(
        defs.good_id("wool").is_some(),
        "a good the merged pack defines must be compiled in; `bread` {} and `wool` {}",
        defs.good_id("bread").is_some(),
        defs.good_id("wool").is_some()
    );
    assert!(
        defs.skill_id("weave").is_some(),
        "a skill the merged pack defines must be compiled in"
    );
    Ok(())
}

#[test]
fn a_good_defined_in_both_packs_is_a_duplicate() {
    // The merge's contract for every other list, now extended to goods: an id in both is an error, not a
    // silent second entry that would give the same good two `GoodId`s.
    let mut base = DefsFile::from_ron(r#"(goods: [(id: "bread")])"#).expect("parses");
    let other = DefsFile::from_ron(r#"(goods: [(id: "bread")])"#).expect("parses");
    assert!(matches!(
        base.merge(other),
        Err(DefsError::Duplicate { kind: "good", ref id }) if id == "bread"
    ));
}

#[test]
fn a_good_only_the_merged_pack_consumes_resolves() -> TestResult {
    // The shape that failed in practice: the **starter** pack names the good and the merged pack brings the
    // activity that uses it, so before the fix the error pointed at the activity and not at the merge.
    let mut base = DefsFile::from_ron(
        r#"(goods: [(id: "bread")], activities: [(
            id: "eat",
            target: Itself,
            duration_hours: 1.0,
            consumes: [("bread", 0.25)],
            satisfies: [("food", 1.0)],
        )], needs: [(id: "food")])"#,
    )?;
    let other = DefsFile::from_ron(
        r#"(goods: [(id: "flour")], activities: [(
            id: "mill",
            target: Itself,
            duration_hours: 1.0,
            produces: [("flour", 2.0)],
            consumes: [("bread", 0.1)],
        )])"#,
    )?;
    base.merge(other)?;
    let defs = SocietyDefs::compile(base)?;
    assert!(defs.good_id("flour").is_some());
    assert!(defs.activity_id("mill").is_some());
    Ok(())
}

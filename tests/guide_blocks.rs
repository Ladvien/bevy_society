//! **The definitions file the guide tells a host to write must parse.**
//!
//! `docs/guide/authoring.md` opens with a complete `DefsFile` — needs, topics, deeds, factions — and then shows
//! fragments to add to it. A host's first move is to copy that block and run. Nothing checked that it parses,
//! and this crate's definitions loader is strict: it rejects unknown fields by name and requires every
//! cross-reference to resolve, so a stale example fails immediately for the reader and silently here.
//!
//! This file extracts the `ron` block from the guide itself rather than restating it. A copy in the test would
//! keep passing after the documentation drifted, which is the failure mode being guarded against.
use bevy_society::SocietyDefs;

/// The first complete `ron` block in `docs/guide/authoring.md`.
fn guide_block() -> String {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/guide/authoring.md"
    ))
    .expect("the authoring guide is in the repository");
    let start = source
        .find("```ron\n")
        .expect("the guide opens a ron block");
    let rest = &source[start + "```ron\n".len()..];
    let end = rest.find("```").expect("the ron block closes");
    rest[..end].to_owned()
}

#[test]
fn the_guides_first_definitions_file_parses() {
    let block = guide_block();
    assert!(
        block.contains("needs") && block.contains("factions"),
        "the block extracted is not the complete file the guide opens with:\n{block}"
    );
    let defs = SocietyDefs::from_ron(&block).unwrap_or_else(|err| {
        panic!(
            "the guide's first definitions file does not parse, so a host copying it gets an error \
             immediately:\n{err}\n\nblock:\n{block}"
        )
    });

    // And the things it claims are there, are.
    assert!(
        defs.need_id("social").is_some(),
        "the block declares `social`"
    );
    assert!(defs.need_id("rest").is_some());
    assert!(defs.topic_id("kindness").is_some());
    assert!(defs.deed_id("help").is_some());
    assert!(defs.faction_id("millers").is_some());
}

/// **Every trigger form the guide prints must parse.** The guide listed `RumorReach { deed, fraction }`,
/// `NeedBelow { need, below }` and `DialogueNode { conversation, node }` in **brace** form, and RON rejects a
/// struct variant written that way — `Expected opening (`. `NormViolated(deed)` was printed with a field name
/// for a tuple variant, which rejects with `Expected string`. Three of the nine documented forms were therefore
/// a copy-paste error for anyone following the guide.
///
/// The list below is the guide's own, so a future edit to the prose that is not matched here will fail rather
/// than silently mislead.
#[test]
fn every_trigger_form_in_the_guide_parses() {
    let forms = [
        "Tick",
        "Hour(3.0)",
        r#"Interaction("chat")"#,
        "TieLabel(Friend)",
        r#"RumorReach(deed: "steal", fraction: 0.5)"#,
        r#"NormViolated("steal")"#,
        r#"NeedBelow(need: "social", below: 0.3)"#,
        r#"Activity("eat")"#,
        r#"DialogueNode(conversation: "greet", node: "n")"#,
        r#"Custom("thing")"#,
    ];
    for form in forms {
        let source = format!(
            r#"(
                needs: [(id: "social", decay_per_day: 1.0)],
                topics: [(id: "kindness")],
                deeds: [(id: "steal", topic: "kindness", harm: -0.7)],
                conversations: [(conversation: "greet", priority: 0)],
                activities: [(
                    id: "eat",
                    target: Itself,
                    duration_hours: 0.5,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                )],
                interactions: [(id: "chat")],
                triggers: [(
                    id: "t",
                    on: {form},
                    then: "fx.emit(\"x\", #{{}})",
                    once: true,
                )],
            )"#
        );
        SocietyDefs::from_ron(&source).unwrap_or_else(|err| {
            panic!("the guide documents `on: {form}` but it does not parse:\n{err}")
        });
    }
}

/// **The conversation block the guide shows must parse too**, and its `if` condition is not the same syntax as
/// the `on` variant: `ctx.label(actor, target) == "friend"` is Rhai and takes the **lowercase** script name,
/// while `on: TieLabel(Friend)` is RON and takes the capitalised variant. Both appear within twenty lines of the
/// guide, so a host could reasonably confuse them.
#[test]
fn the_guides_conversation_block_parses() {
    let source = r#"(
        needs: [(id: "social", decay_per_day: 1.0)],
        topics: [(id: "kindness")],
        conversations: [
            (conversation: "greet_friend", priority: 2, if: Some("ctx.label(actor, target) == \"friend\"")),
            (conversation: "greet", priority: 0),
        ],
        node_effects: [
            (conversation: "greet", node: "compliment", then: "fx.adjust(actor, target, \"affinity\", 0.1)"),
        ],
    )"#;
    SocietyDefs::from_ron(source).expect("the guide's conversation block parses");

    // And the lowercase script name is what `TieLabel::script_name` returns — the two spellings are deliberate.
    assert_eq!(bevy_society::TieLabel::Friend.script_name(), "friend");
}

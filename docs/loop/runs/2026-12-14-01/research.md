# Research: three of the guide's nine trigger forms did not parse

## Why this iteration

Item 2 was "recover the clamp's mood cost". I looked for something a host hits *before* any of that — the guide
they follow to write their first definitions file — and no iteration had ever compiled it.

## The finding

`docs/guide/authoring.md` lists every `on:` form a trigger can take. Three were printed in **brace** form for
struct variants, and one with a field name for a tuple variant:

| printed | actual | error |
|---|---|---|
| `RumorReach { deed, fraction }` | `RumorReach(deed: "steal", fraction: 0.5)` | `Expected opening (` |
| `NeedBelow { need, below }` | `NeedBelow(need: "social", below: 0.3)` | `Expected opening (` |
| `DialogueNode { conversation, node }` | `DialogueNode(conversation: "greet", node: "n")` | `Expected opening (` |
| `NormViolated(deed)` | `NormViolated("steal")` | `Expected string` |

RON writes a **struct variant with parentheses**, and `NormViolated` is a *tuple* variant taking a bare string.
A host copying the guide got an error on their first run, with a message that names the byte offset and not the
mistake.

## A second trap found while fixing the first

`TieLabel(Friend)` — the label variants are **capitalised** in RON, because they are `Stranger`,
`Acquaintance`, `Friend`, `Rival` as declared. Writing `TieLabel(friend)` fails with a list of the four valid
names. The guide said only `TieLabel(label)`, which is not writable.

And the **lowercase** form is real and correct twenty lines later: `if: Some("ctx.label(actor, target) ==
\"friend\"")` is Rhai and takes `TieLabel::script_name`, which returns `"friend"`. Two spellings, both right, in
one document — which is exactly the kind of thing worth a test rather than a footnote.

## Design

The list becomes a table of all ten forms with what each fires on, and `tests/guide_blocks.rs` asserts **every
row** against `SocietyDefs::from_ron`.

The test extracts the guide's first complete block **from the markdown** rather than restating it: a copy inside
the test would keep passing after the documentation drifted, which is the failure mode being guarded against.

## Sources

Unchanged. `TriggerOn` is `src/defs.rs`; no model is involved.

# Eval

## Change

`docs/guide/authoring.md`'s trigger list corrected to the forms the loader accepts, plus three tests that compile
the guide's own blocks. No crate code changed.

## Before / after

| | before | after |
|---|---|---|
| trigger forms in the guide that parse | **6 of 10** | **10 of 10** |
| guide's opening definitions file | never compiled | **asserted** |
| guide's conversation block | never compiled | **asserted** |
| `TieLabel(friend)` | guide said `TieLabel(label)`, not writable | `TieLabel(Friend)`, with the capitalisation stated |

## The four errors, verbatim

| printed | loader says |
|---|---|
| `RumorReach { deed, fraction }` | `Expected opening (` |
| `NeedBelow { need, below }` | `Expected opening (` |
| `DialogueNode { conversation, node }` | `Expected opening (` |
| `NormViolated(deed)` | `Expected string` |

A host copying any of them got a byte offset and no explanation. RON writes a **struct variant with
parentheses**, and `NormViolated` is a **tuple** variant taking a bare string.

## The second trap

`TieLabel(Friend)` — variants are capitalised. But `ctx.label(actor, target) == "friend"` is **Rhai** and takes
the lowercase `script_name`, twenty lines later in the same document. Both spellings are right, and the test pins
both so neither is "fixed" into the other.

## Why the test reads the markdown

The opening block is extracted from `authoring.md` rather than restated in the test. A copy inside the test would
keep passing after the documentation drifted, and drift is exactly what happened here.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **53** binaries |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |

## Dead-code inventory

Zero.

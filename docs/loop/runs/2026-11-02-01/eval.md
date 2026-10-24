# Eval

## What the harness changed

| | before | after |
|---|---|---|
| A `warn!` deleted from `BehaviorPlugin::finish` | every test still green | `tests/warnings.rs` fails |
| The warning's activity name | not asserted | asserted (`snub`, `follow`) |
| The warning's arithmetic | not asserted | asserted (`1.1 ticks`) |
| A warning fires at all | read by eye once | asserted, level and message |

## The defect it caught

`warn_unscored_fields` counted an activity twice when it had both a `Field` consideration and a
`Gradient` target:

| | message |
|---|---|
| before | `2 definitions rely on social-field inputs … — follow, follow` |
| after | `1 definitions rely on social-field inputs … — follow` |

One offending activity, reported as two. Every invariant test from the previous iteration passed while
that sentence was wrong.

## Captured output

`evidence/captured-warnings.txt` — the two lines, verbatim, from the asserting run.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **19** binaries ok (18 + `warnings`) |
| `cargo test --no-default-features` | 19 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## Dead-code inventory

Unchanged at zero. `tests/common/capture.rs` is test support and carries the module's existing
`#![allow(dead_code)]`, which is what lets a helper exist before every test in every binary uses it.

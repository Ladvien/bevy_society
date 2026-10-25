# Plan: a person-targeted activity must not be cancelled on its first tick

## Why this over the runner-up

Last iteration listed "`ActivityTarget::Home` ignores `Place::capacity`" as the top item. Before building it I
added an `interrupted_by_activity` breakdown to `summary.json` — and the biggest failure was **not** `sleep`
(162) but `compliment` (**1 045**), a *person*-targeted activity. Building a capacity fix would have moved
nothing. The breakdown redirected the iteration before any code was written, which is the runner-up argument.

## Change

One expression in `src/behavior.rs::end_activities`:

```rust
// before
DoingTarget::Person(target) => perception.persons.binary_search(&target).is_err(),
// after
DoingTarget::Person(target) => !perception.persons.contains(&target),
```

`Perception::persons` is ordered by `PersonId`, not by `Entity`; `binary_search` requires the latter. Measured,
the entity ids come out **strictly descending**, so the search found almost nothing.

## Touched

- `src/behavior.rs` — the check, with the measurement in the comment.
- `examples/evidence.rs` — `interrupted_by_activity` in `summary.json` (the diagnostic that found this).
- `tests/village_evening.rs` — a threshold guard on the observable.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## Measured

| | interrupted | share | `compliment` | friends | rivals |
|---|---|---|---|---|---|
| before | 2 237 | 22.8% | 1 045 | 22 | 46 |
| after | **400** | **5.9%** | **11** | **31** | **38** |

## Verification

| check | result |
|---|---|
| revert the one line | restores 2 237 / 1 045 **exactly** |
| `cargo test` both feature sets | **58** binaries |
| clippy / docs / fmt | 0 errors |
| activity mix unchanged in shape | sleep 40%, work 24%, eat 10% |

## The test question, answered honestly

Six attempts were made to reproduce this in a test — the real village fixture, a hand-built village, a ring of
12 movers, a ring of 24, and two churn variants. **All passed under the bug.** The defect needs the specific
`PersonId` order `examples/evidence.rs` produces, which a separate fixture would have to imitate and would then
drift from. The test that ships is therefore a **threshold guard on the observable**, labelled as such, not a
claim to reproduce the defect.

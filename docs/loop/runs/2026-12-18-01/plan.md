# Plan: an honest `Doing` reading

## Why this over the runner-up

Item 1 (message surface) is already complete — `InteractionResolved`, `ActivityStarted` and `ActivityEnded` are
all registered and used 39 times. Item 2's most concrete instance is `Doing` mislabelling a walking person. Items
1 and 2 of the top-5 are escalation-blocked, and item 4 (perf) turned out to be unmeasurable on this machine.

## Change

`impl Doing` gains:

- `is_travelling()` — `arrived_day.is_none()`;
- `label(&SocietyDefs)` — `"travelling:<activity>"` before arrival, the bare name after.

`examples/evidence.rs` calls `label` instead of inlining the same string, which is the point: the convention now
has one definition instead of one per reader.

## Touched

- `src/behavior.rs` — the impl block.
- `examples/evidence.rs` — drop the private workaround.
- `tests/doing_label.rs` — three tests, new.
- `docs/loop/needs-decision/what-doing-reports-while-travelling.md` — what shipped and what is still open.

## Verification

| check | result |
|---|---|
| `series.csv` before vs after | **byte-identical** (`cmp`) |
| `summary.json` before vs after | identical (sorted-JSON diff) |
| new tests bite | the walking test fails when the prefix is deleted |
| `cargo test` both feature sets | **55** binaries |
| clippy / docs | 0 errors each |

## What was deliberately not done

- No schema change (option 2) — that is a host decision and stays open.
- `tests/travel.rs` stays `#[ignore]`: it asserts the *stronger* claim that a walker is being **paid**, which is
  the clock decision, not this one.

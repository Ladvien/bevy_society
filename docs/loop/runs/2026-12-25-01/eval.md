# Eval

## Change

One village definition. `tests/common/village.rs` matches `examples/common/scene.rs`, `examples/evidence.rs` reads
the canonical constant instead of copying it, and `tests/village_layout.rs` fails if any of them drift again.

## The drift, measured

| | tavern | market | workshop | farthest pair |
|---|---|---|---|---|
| canonical (`examples/common/scene.rs`) | (6.5, -5.0) | (-6.5, -5.0) | (6.5, 5.0) | **16.4 m** |
| drifted (`tests/common/village.rs`) | (8.0, -6.0) | (-8.0, -6.0) | (8.0, 8.0) | **21.3 m** |

At `village-evening`'s 64 s/day, 16.4 m needs `travel_timeout_hours` **4.4**; 21.3 m needs **5.7**. The scenario
declares 4.0. A test on the drifted layout was exercising walks the measured village cannot make.

## What changed in behaviour

**Nothing.** `series.csv` is **byte-identical** before and after the harness refactor — the harness was already
reading the canonical numbers, it just kept its own copy of them. The test-side correction changes which village
three tests walk around in, and all three still pass, which is itself evidence the drift was invisible to them.

This iteration's value is that the next drift cannot be silent.

## Why this was worth an iteration

Four consecutive iterations logged "move the builder where `tests/` can reach it" and treated it as a refactor
chore. The real defect was a **divergence that made tests meaningless**, and it was disguised because the three
tests using that builder still passed. A passing test on the wrong village is worse than a failing test on the
right one.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **68** binaries |
| clippy / docs / fmt | 0 errors |
| bite check | re-introducing the drift fails `village_layout` with both coordinates named |

## Coverage

Real. The pinning test fails on exactly the change that caused the problem, and its message tells the next person
which copy to fix and which to leave alone.

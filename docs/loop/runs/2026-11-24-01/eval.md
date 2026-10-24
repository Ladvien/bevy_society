# Eval

## What changed

A third startup check, `warn_walk_outlasts_activity`: if `MIN_WALK_METRES` of walking costs more game-hours
than the **mean** activity duration, a host is told at startup. It names the number, the arithmetic, the open
decision, and three ways out.

No behaviour changed. The society is bit-identical; what changed is that a host can now hear about a defect it
previously had to find in a plot.

## The measurement that motivated it

Same seed, same 24 people, same 16.4 game-days, only `seconds_per_day` varied:

| `seconds_per_day` | travelling | need mean |
|---|---|---|
| **64** | **58.9%** | 0.545 |
| 144 | 50.4% | 0.603 |
| 240 | 46.8% | 0.621 |
| **480** | **43.9%** | 0.624 |

Fifteen points of a person's life, from a pacing knob. `evidence/coupling.png` plots both panels.

## Why the warning and not the fix

The coupling is `needs-decision/travel-time-in-two-clocks.md`, open, and its recommendation is a breaking
change to `Tuning`. Not mine to take. The **silence** is not a decision — the crate already argues that for
its other two checks.

And the curve, which was the log's #1, measured flat: three shapes within one point of baseline.

## Proof it works

| Run | Result |
|---|---|
| `village-evening --seconds-per-day 64` | **warns**: "2.68 game-hours, longer than the mean activity (1.62h)" |
| `village-evening --seconds-per-day 240` | **silent** — 0 matches |

## Gates

| Command | Result |
|---|---|
| `cargo test` | **39** binaries |
| `cargo test --no-default-features` | **39** binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | 0 errors |

## Dead-code inventory

Zero. The warning is called from `finish`, and both its branches are asserted by tests.

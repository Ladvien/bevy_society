# Plan: measure the scale curve, and fix the tool that made it unmeasurable

## Why this over the runner-up

Item 1 is a schema change and non-urgent. Item 2 has stood for nine iterations, and last iteration's note was
already the right instruction: *stop waiting for a quiet machine; re-measure with median-of-N and a reported
spread*. Following my own instruction beat writing a tenth note.

## Change

1. **`examples/benchmark` takes `--reps`** and prints `ms_per_tick_min`, `ms_per_tick_median`, `ms_per_tick_max`
   and `spread_pct` instead of a single mean.
2. **`docs/guide/concepts.md`'s scale table replaced** with measured fastest-window figures, the method stated, and
   the old numbers explicitly retracted with the reason they survived.
3. **`tests/benchmark_reporting.rs`** pins the output contract — a future change that drops back to one number
   fails.

## Measured

| persons | fastest window | median of 9 | spread |
|---|---|---|---|
| 100 | **0.74 ms** | 1.08 | 103% |
| 1 000 | **28.4 ms** | 60.4 | **100%** |
| 2 000 | **100.6 ms** | 140.1 | 49% |

Reproducibility of the fastest window: 28.29 / 28.38 / 28.38 / 28.48 at 1 000 (**0.7%**), 100.55 / 100.93 at
2 000 (**0.4%**) — while the medians moved 18-25%.

## Touched

- `examples/benchmark.rs` — repetition and the spread fields.
- `docs/guide/concepts.md` — the scale table and its method.
- `tests/benchmark_reporting.rs` — new.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## Verification

| check | result |
|---|---|
| numbers reproduce | fastest window within 0.7% across four invocations |
| bite check | reverting to one mean fails the test: "the benchmark line has no `ms_per_tick_min`" |
| `cargo test` both feature sets | **69** binaries |

## What this retracts

`13.5 ms` at 1 000 and `245.1 ms` at 5 000 in the guide cannot be reproduced on this hardware — the fastest window
here is 28.4 ms, more than double. They are replaced rather than annotated, because a host sizing a game from that
table was being told the crate is 2x cheaper than it is.

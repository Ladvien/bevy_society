# Eval

## Change

`examples/benchmark --reps N` reports min / median / max / spread. The guide's scale table is replaced with the
measured figures, and its method is stated.

## The finding

| persons | fastest window | median of 9 | spread |
|---|---|---|---|
| 100 | **0.74 ms** | 1.08 | 103% |
| 1 000 | **28.4 ms** | 60.4 | **100%** |
| 2 000 | **100.6 ms** | 140.1 | 49% |

A **100% spread at 1 000 people**. The old tool printed a single mean, so that variance was invisible — and a
mean drawn from it describes the machine's load, not the crate.

## What is reproducible, and why

The **fastest window**: 28.29 / 28.38 / 28.38 / 28.48 at 1 000 (0.7%), 100.55 / 100.93 at 2 000 (0.4%). Across
the same invocations the medians were 57.5 / 60.4 / 71.7 / 57.3.

A fast window is evidence the machine was momentarily quiet; a slow one is evidence it was not. So the minimum is
a bound on what the code costs, and the median is a bound on what the machine costs. Those are different
questions, and the guide was answering the second while presenting it as the first.

## The retraction

| persons | guide said | fastest window measured |
|---|---|---|
| 1 000 | **13.5 ms** | **28.4 ms** |
| 2 000 | 49.9 ms | 100.6 ms |
| 5 000 | 245.1 ms | does not complete inside a sensible deadline here |

The guide was understating the cost by roughly **2x**. A host budgeting from it was being misled, and the crate's
own honesty rule — every number has a source — was satisfied by a figure nobody could reproduce.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **69** binaries |
| clippy / docs / fmt | 0 errors |
| bite check | one mean → "the benchmark line has no `ms_per_tick_min`" |

## Coverage

Real. The test parses the tool's own output and asserts that `spread_pct` is derived from the reported windows
rather than being a printed constant, so it cannot pass on a tool that reports a spread it did not measure.

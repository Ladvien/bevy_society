# Perf

## The change

`Doing::is_travelling` and `Doing::label` — two methods, no new field, no new pass, no added work per tick. The
evidence harness now calls `label` instead of building the same string inline, and its output is **byte-identical**
(`cmp` on `series.csv`, and a sorted-JSON diff on `summary.json`).

## Benchmarks, and why they are not trustworthy today

Measured on the M1 Max with `examples/benchmark --ticks 300 --settle 400`:

| persons | before | after | doc says |
|---|---|---|---|
| 100 | 0.97 ms | 0.97 ms | 0.6 ms |
| 1 000 | 36.81 ms | 36.77 ms | 13.5 ms |
| 5 000 | 463.3 ms | 461.4 ms | 245.1 ms |

**The absolute numbers are not usable.** The machine's load average was **10.2 on 10 cores** throughout, and the
100-person case — which does no perception-grid work at all — reads 1.6x slower than the documented 0.6 ms. The
contention is the difference, not the code.

What *is* usable is the **pair**: before and after were measured back to back on the same loaded machine, and they
agree to within 1% at every size. A change that added per-tick work could not hide behind contention like that,
because both halves of every pair are equally contended.

## What this iteration did not establish

`docs/guide/concepts.md` claims 0.6 / 13.5 / 245.1 ms. Those were measured in November on this same model, and
today's *unmodified* code reads 0.97 / 36.8 / 461. A 2.5x gap at 1 000 is far too large for contention alone to
explain, so either the guide's numbers are stale relative to later work or they were taken with a different
harness configuration. **This is now a standing item**: re-measure the scale curve on an idle machine before
quoting it again. Recording the doubt is better than shipping a number that might be wrong by 2.5x.

## Cost of the change

Zero measurable. Two accessor methods on an existing component.

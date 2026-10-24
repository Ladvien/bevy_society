# Perf

No library change shipped: the grid was implemented, measured and reverted. The numbers that matter are
the ones it produced.

## Perception, standalone

| people | ms/tick | share of the benchmark tick |
|---|---|---|
| 1 000 | **1.45** | 9.3% of 15.67 |
| 5 000 | **35.49** | **11.8% of 300.8** |

Both grow as `n²`: four times the people is 24× the time.

## The three implementations

| | brute force | `HashMap` grid | `BTreeMap` grid |
|---|---|---|---|
| 100 | **0.573** | 0.707 | — |
| 1 000 | **15.67** | 15.99 | 16.97 |
| 5 000 | 300.8 | **297.96** | — |

The `HashMap` grid's only win is 1% at 5 000, which is inside this machine's run-to-run spread. It loses
at 100 by 19% and at 1 000 by 2%, and the `BTreeMap` variant is the slowest of the three.

## Why

Brute force yields neighbour lists already sorted by index — both loops walk `0..n`. The grid yields them
in cell order and pays `sort_unstable` over **4 264 430 entries** at 5 000 to restore the order every
per-person loop in the crate depends on. That sort costs about what the distance checks cost, and nine
hash lookups per person add ~10× a distance check each.

SPH runs several neighbourhood queries per simulation step and amortises the structure. This crate runs
**one** per society tick.

## Standing benchmark, unchanged

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |

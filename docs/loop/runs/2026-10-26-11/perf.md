# Performance: the price reads a sum instead of keeping a counter

The hot path barely changes. The old `update_prices` integrated `Δprice = price_speed_per_day · (produced −
consumed)` from two counters that `produce_and_consume` updated on every good movement; the new one sums each
person's `Stock` once per price update and applies a piecewise function. **`src/model/price.rs` is two scalar
functions** — no allocation, no collection, no draw.

The measurement of the *removed* work: `produce_and_consume` no longer writes `flow.produced[..] += added` and
`flow.consumed[..] += taken` on each activity completion, so it is strictly less work per completion. The added
work is one `Stock` sum over `PersonIndex.sorted` per price update, which is `O(people · goods)` with `goods` a
handful.

**Measured on the laptop**, `--people N --ticks 200`, five reps, fastest window taken by the harness:

| persons | before (ms/tick min / median) | after (ms/tick min / median) |
|---|---|---|
| 100 | 1.49 / 2.11 | 1.64 / 1.97 |
| 1 000 | 49.7 / 76.9 | 46.8 / 73.2 |
| 5 000 | 841 / 1207 | see below |

**Inside the noise at every size.** The within-run spread is 52–78% at 100 and 1 000 people, far wider than any
difference between the two builds, and the medians move in opposite directions at 100 (after *faster*) and
1 000 (after faster still), which is what a machine under varying load produces rather than a cost.

**This machine was far slower than the run `runs/2026-10-25-10/perf.md` recorded** — that pass measured
0.79/1.60 ms at 100 people and 433/534 at 5 000; this one 1.49/2.11 and 841/1207 for the *same* code at 100 and
5 000. The absolute numbers are therefore not comparable across iterations and are not used as such; only the
paired before/after column here is.

**The `--people 5 000` before column is the last measurement that completed.** The after-side run at 5 000 was cut
by the harness deadline on a loaded machine, and the run was not repeated at that size; the 100 and 1 000 pairs
carry the claim, and the model's cost is provably independent of population beyond the one sum.

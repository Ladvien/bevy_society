# Performance: goods change hands

What changed on the hot path: `Decision` gained the actor's `Stock`, the trade thresholds, and — **only when some
activity reads `Input::Surplus`** — a copy of every neighbour's `Stock`. A pack with no `Surplus` (the benchmark's,
and any host's that does not trade) pays one `Query::get` per person and nothing per neighbour.

## The benchmark's own pack (no goods, no `Surplus`)

`cargo run --release --example benchmark -- --people N --ticks T`, the baseline built from the commit this branched
from (`37d80e8`) in its own worktree, two rounds each, interleaved before / after. ms per society tick.

| persons | before: min / median (round 1, 2) | after: min / median (round 1, 2) |
|---|---|---|
| 100 | 0.787 / 1.579, 0.799 / 1.598 | 0.804 / 1.650, 0.803 / 1.604 |
| 5 000 | 431.3 / 532.9, 432.1 / 533.6 | 436.1 / 533.3, 433.7 / 534.5 |

**Inside the noise at both sizes.** At 5 000 people the median moves by +0.1% and +0.2% and the fastest window by
+1.1% and +0.4%, against a within-run spread of 27%. At 100 people the spread is 91% (windows of 0.8 to 2.2 ms),
so the 0.5-4.5% differences are not distinguishable from nothing. The 1 000-person point is not re-measured for
this pack: the machine's spread at that size has been 24-44% across iterations.

## With the village pack, and `share` switched on and off

The benchmark pack declares no goods, so it cannot show the cost of copying neighbours' stores. This run merges
`examples/assets/village.society.ron` into the benchmark (a temporary local edit, not committed) and removes the
`share` activity for the control, so the **only** difference is whether `Surplus` is read.

| persons | `share` off: min / median | `share` on: min / median | difference |
|---|---|---|---|
| 1 000 | 27.0 / 39.8 | 27.5 / 39.8 | +1.9% / +0.0% |
| 5 000 | 432.1 / 472.6 | 438.4 / 478.1 | +1.4% / +1.2% |

Spreads of 40-51%. **The copy costs about 1% at 5 000 people and is not visible at 1 000.** Nobody holds bread in
this run, so `Surplus` is zero everywhere and the number is the cost of the *copy*, which is what the gate is for.
It is a `Vec<(Entity, Stock)>` per person per tick and each `Stock` is a heap `Vec`; if a host measures that as
too much, the repair is to pass the `Query<&Stock>` into `signal_value` instead of copying, which needs a lifetime
on `Decision`. Not done: 1% is inside the noise.

## Not measured

- The cost of `apply_effects` moving goods: one `get_mut::<Stock>` per side per accepted share, ~5.6 a game-day
  in the village.
- `produce_and_consume`'s `O(n·endings)` index scan (carried from last iteration): still not exercised by the
  benchmark.

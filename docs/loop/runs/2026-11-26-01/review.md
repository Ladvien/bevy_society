# Review

## Pass 1 — Behavior: is the society the same?

**GREAT**, and this is the pass that matters most for a perf change, because the only thing that could go
wrong is a behavioural difference.

| check | result |
|---|---|
| `village-evening`, 67 199 series rows | **67 199 identical** |
| neighbour lists vs all-pairs, n = 1, 2, 40, 200, 500, radii 8 and 4 | **0 mismatched** |
| `tests/determinism.rs` | 2/2 |

The grid reproduces the all-pairs answer exactly — contents and ascending index order — so no decision,
no tie-break and no RNG draw moves. `tests/perception.rs` pins that identity permanently, including the
degenerate one- and two-person grids where the cell bounds are trivial.

## Pass 2 — Zoom: read the plot closely

**GREAT**, and the plot is a correction, not a celebration. `evidence/grid.png`:

- Left: the two cost curves are **almost on top of each other** across three decades. That is the honest
  picture: the change is 3–7%, not a rescaling.
- Middle: the measured breakdown of a 5 000-person tick — perception 36 ms (all pairs) or 29 ms (grid), and
  **226 ms of everything else**. The crate's own documentation said perception was the quadratic term; this
  bar is the refutation.
- Right: neighbours per person rising 0.4 → 36 → 182, and score calls rising with it. That is what scales.

No benchmark was slowed. The 100-person case improved most (40%) because it was dominated by call overhead.

## Pass 3 — Fresh eyes: the narrative

**GREAT.** One sentence:

> A person now looks only at the nine grid cells around itself instead of at everyone, and gets exactly the
> same neighbours in exactly the same order — so the society is unchanged while the search gets cheaper.
> But the search was only 12% of the cost: what actually grows is scoring every one of 182 neighbours
> against every activity that targets a person, and that is a density problem, not an indexing one.

## What I got wrong on the way

Two guesses before measuring: that a per-person `all_doing.clone()` was the N², then that a lookup map would
fix it. Both moved the number ~5%; both were reasoned from the code rather than measured. The direct timers
settled it in one run. Recorded because the same mistake is available next iteration.

## Verdict

Three GREAT. Shipped.

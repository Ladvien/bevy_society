# Plan: build the spatial index, measure it, revert it

## Why this over the runner-up

Runner-up was retiring untellable rumors. Lost on one sentence: **last iteration measured that the book
costs memory, not ticks**, so the index is the remaining performance item — and unlike the book, its cost
is real and `O(n²)` (35.5 ms of a 300.8 ms tick at 5 000 people).

## Design

`perceive` keeps its interface (`Perception { persons, pos, near, contact }` unchanged) and replaces the
double loop with a uniform grid:

- cells as wide as `perception_radius`, so a person's neighbours are in the nine cells around them —
  the recipe in Ihmsen et al. §2;
- a `HashMap<(i32, i32), Vec<u32>>` in a `Local`, cleared and refilled each tick so the allocation is
  reused;
- neighbour lists restored to **ascending index order**, because every per-person loop in the crate
  assumes it and `Person`-target scoring must be deterministic.

A `BTreeMap` variant was tried second, on the theory that ordered cells would avoid the sort.

## What happened

Both were slower or no faster. Measured at three populations:

| | brute force | `HashMap` grid | `BTreeMap` grid |
|---|---|---|---|
| 100 | **0.573 ms** | 0.707 ms | — |
| 1 000 | 15.67 ms | 15.99 ms | **16.97 ms** |
| 5 000 | 300.8 ms | **297.96 ms** | — |

**Reverted.** The reason is the sort: brute force yields sorted lists for free, the grid scrambles them
and pays `sort_unstable` over 4.26 M entries to restore the order, which costs about what the distance
checks cost. SPH amortises that over several solver passes a tick; this crate makes **one** perception
query per society tick.

## What ships

- `tests/perception.rs` — a reference brute force and a comparison against it, **32 016 neighbour
  entries across 180 people**, so a future index is checked rather than trusted.
- `src/core.rs` — the measured cost documented on `perceive`, and the log's claim corrected from 9.2%
  to **11.8% of a tick at 5 000**.

## New `Tuning` fields

None.

## Tests

One new test file, which passes on the reverted code and would have passed on the grid — it is the check
any future index has to satisfy, not a check on this one.

`cargo test` skips it above 180 people because the comparison is itself `O(n²)` by construction; 180 was
chosen so the test runs in about 40 s and still exercises every awkward case (many people at cell
boundaries, clusters, and lone figures).

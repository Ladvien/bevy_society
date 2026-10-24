# Research: the spatial index that was built, measured, and did not pay

## The log's #2 claim, and what it was worth

> **A spatial index for perception** — every `Person`-targeted activity scores once per neighbour, and
> this iteration's 9.2% is that cost.

So I measured perception itself rather than the scoring on top of it. `perceive` in `src/core.rs` is a
brute-force double loop:

```rust
for (i, pi) in pos.iter().enumerate() {
    for (j, pj) in pos.iter().enumerate() {
```

Standalone, over the benchmark's ring layout:

| people | loop, ms/tick | pairs | edges found |
|---|---|---|---|
| 1 000 | **1.45** | 999 000 | 169 000 |
| 5 000 | **35.49** | 24 995 000 | 4 264 430 |

35.5 ms of a 300.8 ms tick is **11.8%** at 5 000, and the cost grows as `n²`. That is a real target, and
a better one than the 9.2% figure — which was the *scoring* cost, not perception's.

## The source

Ihmsen, Orthmann, Solenthaler, Kolb & Teschner, *SPH Fluids in Computer Graphics* (corpus doc
`eth-cgl-sim_anim-Sol14a`), Sec. 2 "Neighborhood Search":

> While hierarchical data structures such as kd-trees are used in multi-resolution scenarios with a
> variable kernel support […] there seems to be consensus to employ uniform grids for standard SPH with a
> fixed kernel support. **Grids are built in `O(n)` and particles are accessed in `O(1)`**, while
> hierarchical data structures are typically built in `O(n log n)` and accessed in `O(log n)`.

> Space is subdivided into cubic cells and each particle is associated to one cell in the construction
> stage. To find all relevant neighbors, the cell of a particle and all adjacent cells are queried.

That is exactly the shape of the problem: a fixed query radius (`perception_radius`), so a grid whose
cells are that radius wide, and the nine cells around an occupant.

## What was built, and what it measured

The implementation followed the paper: bucket every person by cell, then for each person visit the nine
cells around them and distance-test the occupants. Neighbour lists were restored to ascending index
order afterwards, because every per-person loop in the crate assumes it.

| | brute force | uniform grid (`HashMap`) |
|---|---|---|
| 100 people | **0.573 ms** | 0.707 ms |
| 1 000 people | 15.67 ms | 15.99 ms |
| 5 000 people | 300.8 ms | **297.96 ms** (1%) |

**It did not pay.** A `BTreeMap` variant was worse still: **16.97 ms** at 1 000, against 15.67 for brute
force.

## Why, measured rather than guessed

Brute force produces each neighbour list in **ascending index order for free** — `i` and `j` both walk
`0..n`. The grid visits cells, so it scrambles that order and then pays `sort_unstable` over **4 264 430
neighbour entries** to put it back. That sort costs about what the 25 M distance checks cost, and the hash
lookups (9 per person, ~10× a distance check) add to it.

So the index is the right structure for a *dense* query where the sort is amortised over many queries per
tick — which is why SPH, with several solver passes a step, benefits. This crate does **one** perception
query per society tick, so there is nothing to amortise against.

## What ships

The grid is **reverted**, and the reference brute force lives on in `tests/perception.rs`, which computes
both and compares **32 016 neighbour entries across 180 people** entry for entry. The next index is
checked against that reference rather than trusted.

The cost is now documented where a host will find it, and the log's claim is corrected: perception is
**11.8% of the tick at 5 000**, not 9.2%.

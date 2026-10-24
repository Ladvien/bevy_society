# Plan: perception on a uniform grid, and the truth about the scale curve

## Why this over the runner-up

Runner-up was `rumor-crosses-a-cluster` (the log's #4, a `pending` scenario at 0.333 reach against the 40% it
asks for). Measured first: it is **0.333 at 64 s/day and 0.250 at 240 s/day**, i.e. 2–3 of 12 people, at both
clock settings. Two groups 40 m apart never merge, so the observable is asking for a crossing that needs a
10.7 game-hour walk — a **spec** question, not a crate defect. Not mine to re-declare; the decision file
keeps it.

This iteration does what is mine: the cost a host pays.

## Design

`perceive` (`src/core.rs`) becomes a uniform grid:

- **Cell size = `perception_radius`**, the direct reading of Ericson §7.1.1 for points.
- The grid spans the population's own bounding box, so an empty world costs nothing and no map is allocated.
- Each cell list is filled by one walk of `PersonIndex::sorted`, so it is **ascending by index**.
- A person reads the nine cells around its own, merges, sorts, and tests distance — reproducing the
  all-pairs order exactly.

`decide_activities` builds `close_doing` from the person's **close ties** instead of cloning the whole
population's activity list per person.

## Measured

| persons | all pairs | grid |
|---|---|---|
| 100 | 0.99 ms | **0.59 ms** |
| 1 000 | 14.55 ms | **13.50 ms** |
| 2 000 | 53.96 ms | **49.91 ms** |
| 5 000 | 254.54 ms | **245.13 ms** |

**3–7%.** The instrumented breakdown at 5 000 people: `perceive` 36.0 → 28.9 ms of a ~320 ms fixed step.
Perception was never the curve; **scoring 182 neighbours per `Person`-targeted activity** is.

## Identity

| check | result |
|---|---|
| `village-evening`, 67 199 series rows | **67 199 identical** |
| `tests/perception.rs`, neighbour lists vs all-pairs | **0 mismatched** at n = 1, 2, 40, 200, 500 across two radii |
| `tests/determinism.rs` | 2/2 |

## Touched

- `src/core.rs` — `perceive` and its doc.
- `src/behavior.rs` — `close_doing` filtered to close ties; doc correcting the N² claim.
- `tests/perception.rs` (new).
- `docs/guide/concepts.md`, `examples/benchmark.rs` — the false "perception is the cost" claim replaced with
  the measured breakdown.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `tests/perception.rs::the_grid_finds_exactly_the_all_pairs_neighbours` | the grid's answer is the all-pairs answer, exactly — contents **and** order — at five populations and two radii, including the degenerate n=1 and n=2 grids |

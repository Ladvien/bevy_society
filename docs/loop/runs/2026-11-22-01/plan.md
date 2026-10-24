# Plan: a Place tie-break of nearest, not a coin flip

## Why this over the runner-up

Runner-up was implementing `needs-decision/travel-cost-is-not-scored.md`'s crate-level travel consideration.
Lost on one sentence: **that needs an operator's decision on the definitions schema, while this needs
none** — the decision already knows the distance, it just did not consult it when a `Place` tied.

## Design

In `decide_activities`, the target pick after an activity is chosen:

- `Place` and `Point` targets → **nearest**, via a new `place_distance` helper over the existing
  `transforms` and `settings.plane`;
- `Person` and `Itself` → the existing uniform pull.

Ties on distance fall back to `min_by`, which keeps the first of equal candidates, and the seeded draw is
untouched for `Person`, so determinism holds.

**Why not nearest for `Person`:** a villager that always talks to the physically closest person is a
villager with one friend. Variety in *whom* you talk to is the point; variety in *which tavern* is not.

## What it fixes, measured

| | before | after |
|---|---|---|
| 6 000 decisions, 1 m vs 9 m tavern | 3 020 / 2 979 | **5 999 / 0** |
| `village-evening` travelling | 62.5% | **58.9%** |
| need mean | 0.526 | **0.545** |

Holds at seed 42: 58.8% / 0.550.

## Touched

- `src/behavior.rs` — `place_distance`, the tie-break match.
- `tests/nearest.rs` (new) — 6 000 decisions against a 1 m and a 9 m tavern.
- `CHANGELOG.md`, `docs/loop/log.md`, `docs/research/needs-and-utility.md`.

## New `Tuning` fields

None. No rate changed; this is which of two equal-scoring destinations is taken.

## Tests

| Test | What it proves |
|---|---|
| `a_person_walks_to_the_nearer_of_two_places` | **written red first**: 3 020 / 2 979 before, 5 999 / 0 after |
| `determinism` | unchanged, 2/2 — the seeded stream still decides everything it decided before |

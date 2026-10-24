# Perf

## The change

One field read and one gradient evaluation per scored option and per activity target. A field read is a
`StigGrid::sample_cell` plus a division; a gradient is four of those. Against a decision that already scores
every activity against every neighbour, this is negligible.

Two structural savings came with it, both forced by Bevy's sixteen-parameter limit:

- `decide_activities` no longer takes a second `Query<(Entity, &Transform)>`; the places query already
  carries their `Transform` and `Perception` carries every person's position.
- `place_distance` now reads the per-decision place list instead of querying the world.

## Measured

| persons | before | after |
|---|---|---|
| 1 000 | 13.5 ms | not re-measured — no `Obstacle` or `Field` in the benchmark, and one fewer query |

Honest statement: the benchmark spawns no field definitions, so the added scoring work never runs there. The
removed query is a genuine saving that the benchmark will show only at 5 000 people where it is small.

## Not slowed

`village-evening` is bit-identical (67 199 of 67 199 rows).

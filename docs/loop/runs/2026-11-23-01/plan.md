# Plan: make `Distance` work for a `Place` target

## Why this over the runner-up

Runner-up was adding `Distance` to the pack's `eat`, `relax` and `work`. Lost on one sentence: **the pack
could not use it** — `Distance` reported `1.0` for every place, so the line does not price travel, it
disables the activity (measured: `food` 0.402 → **0.034**, at zero 93%). Fixing the capability first is the
prerequisite, and it needs no decision.

## Design

- `PlaceView` gains `position`, filled from the place's `Transform` and the society plane. The `places`
  query already ran; it needed `&Transform` added.
- `Decision` gains `place_positions: Vec<(Entity, Vec2)>`, built from `place_views` once per decision.
- `Signal::Distance` finds a `Place` target in that list. `Point` and `Itself` keep the `None` → 1.0 path,
  with a comment saying why each is genuinely unknown rather than merely unimplemented.

## Measured

| | before | after |
|---|---|---|
| both taverns in one run, 1 m vs 7 m | **0 / 0** | **5 999 / 0** |
| `eat` with `Distance`, `food` mean | 0.034 (93% at zero) | **0.176** (0% at zero) |

**Behaviour with the shipped pack is unchanged**: all 67 199 series rows of `village-evening` are identical,
because nothing in the pack asks for `Distance` on a `Place`. A capability fix, not a behaviour change.

## What it hands to the decision file

With the signal correct, adding `Distance` to `eat` still leaves `food` below baseline (0.176 against 0.402)
because the pack's curve is `Linear(-1.0, 1.0)` — a tavern 6.5 m away scores **0.19**. The shape of that
curve is `needs-decision/travel-cost-is-not-scored.md`'s question; this iteration supplies the capability.

## Touched

- `src/behavior.rs` — `PlaceView::position`, `Decision::place_positions`, the `Signal::Distance` arm.
- `tests/distance.rs` (new).
- `docs/loop/needs-decision/travel-cost-is-not-scored.md` — the blocker recorded, now resolved.
- `CHANGELOG.md`, `docs/loop/log.md`, `docs/research/needs-and-utility.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `a_place_one_metre_away_scores_better_than_one_across_the_village` | **written red first at 0 / 0**: a `Place` activity asking `Distance` must be choosable at all, and must prefer the near place — **5 999 / 0** |

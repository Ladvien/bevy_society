# Plan: a full place is a price

## Why this over the runner-up

Runner-up was "raise the rate of friendly interaction" (`good-is-outweighed-by-bad.md` option 3). It is
authored content that changes the activity mix a host may have tuned, and it needs a three-axis
measurement this iteration does not have room for on top of the measurement below. The capacity item is
**carried by five consecutive iterations**, it is a library defect rather than a pack choice, and it has a
citation that fixes the *shape* rather than just the direction.

## Design

### 1. Score, don't filter

`src/behavior.rs::targets_for` stops dropping `used >= capacity` places. `Place(tag)` and `Bound(name)`
both offer their place as a candidate, and a new `crowding` term multiplies the activity's score.

Concretely, `score_option` gains the place's occupancy for `DoingTarget::Place`:

```
s_crowd = if capacity == 0 { 0.0 }                       // a place that seats nobody seats nobody
          else { exp(-kappa * max(0, (used + 1) / capacity - 1)) }
```

- `used + 1` because the person being decided is not in `used` yet; it is asking for a seat, so it is the
  (used+1)-th occupant that matters.
- `max(0, ..)` keeps it exactly 1.0 while there is room — **no behaviour change at any place that is not
  full**, which is what makes the change measurable against a baseline.
- `exp(..)` never reaches 0 for finite occupancy, so the activity survives at a price.
- `capacity == 0` is an explicit zero: a host that means "nobody" keeps meaning it. This preserves the
  `tests/bound_places.rs::a_capacity_zero_bound_place_admits_nobody` contract without special-casing `Bound`.

### 2. New tuning field

```rust
/// How fast a place's desirability falls once it is over its declared `capacity`.
///
/// **Design default, citing the shape and not the number.** Talamali et al. 2019
/// (DOI `10.1007/s11721-019-00176-9`) model congestion on a foraged path as an exponential in the number
/// of users over the number the path fits, `T_C = T_0 (exp[κ · n / N_crit] − 1)`, and measure that the
/// collection curve is linear below `N_crit` and decays nonlinearly above it. `Place::capacity` is
/// `N_crit`; κ is not transferable (the paper fits 2.321 to Kilobot collisions, which the crate does not
/// simulate), so this is a design default chosen by measurement.
pub place_crowding_penalty: f32,
```

Default 1.0: the first body over capacity costs `exp(-1) = 0.368` of the score, the second
`exp(-2) = 0.135`. That is steep enough to redirect and shallow enough to keep the activity alive.

### 3. Keep the startup warning

`warn_unreachable_places` already tells a host its home seats fewer people than it houses. It stays; the
change makes the *behaviour* degrade gracefully instead of the activity vanishing, which is what the
warning never said.

## Touched modules

- `src/behavior.rs` — `targets_for`, `score_option`, `PlaceView` (already carries `used`/`capacity`).
- `src/core.rs` — one `Tuning` field, its doc, and the default.
- `src/trace.rs` — no change: the crowded score appears in the trace as the activity's score.
- `tests/place_crowding.rs` — new.
- `tests/bound_places.rs` — the capacity-0 test keeps passing unchanged (that is the point of the explicit
  zero).
- `docs/guide/getting-started.md`, `README.md`, `CHANGELOG.md`, `docs/research/needs-and-utility.md`.

## Tests

1. **A full place still gets a candidate.** Capacity 1, two people, one already there: the second person's
   decision still includes the activity (as a score, not as an absence). This is the direct regression
   test for the defect.
2. **The multiplier is 1.0 below capacity and monotone decreasing above it.** Pure arithmetic on the
   helper.
3. **Two places, one crowded: the decision prefers the empty one.** End-to-end, with a fixed seed, over
   enough ticks that the choice is not a tie.
4. **Capacity 0 admits nobody** — the existing contract, re-asserted against the new path.

## Measurement

`examples/evidence.rs --scenario village-evening --seed S --people N` for N in **24, 36, 48** and seeds
**7, 42, 11** — the same nine runs the `village-grows` scenario records, before and after:

- rival pairs and their share of directed pairs,
- `work` and `craft` share of person-ticks,
- interrupted endings and the `sleep` share,
- friends,
- and the `doing` column, so "an activity vanished" is visible rather than inferred.

Before: this same binary, with `place_crowding_penalty` unread (i.e. today's filter). After: the change.
Same seeds, same populations, same day length.

## Risk

The change affects **every** village at every population, because `used` enters every `Place` score. It is
1.0 below capacity, so a village with room is byte-identical — which is the guard, and `24 people / 30
seats` is exactly such a village, so the "before" run at 24 should be **unchanged** if the implementation
is right. If the 24-person run moves, the change is wrong.

# Eval

## Change

`warn_unreachable_places` now also warns when a home holds more people than its arrival disc can seat.

## What it says on the project's own village

```
'well' is home to 8 people, but an arrival disc of radius 1.5 seats about 4. They all go to bed at the same
hour, so the queue behind the disc times out and their sleeps are interrupted. Give them more room: raise
`interaction_radius` to about 2.0 or spread the residents over more homes.
```

`'tavern'` gets the same line. Those are the two crowded homes in that village.

## The measurement behind it

Bodies already standing in the goal disc when a `sleep` times out:

| in the disc | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|
| timed-out sleeps | 1 | 1 | 8 | 40 | **55** | 36 | 4 | 2 | 3 |

Median **6** — exactly the declared `capacity` of those homes. The disc seats about `(1.5/0.7)² ≈ 4.6`, so the six
who get in are already over-packed and the rest queue until `travel_timeout_hours` expires.

## What was deliberately not shipped

Three more fix levers, all measured, all rejected:

| lever | interrupted | rivals |
|---|---|---|
| baseline | 400 | 38 |
| crowd allowance 0.35 | 349 | **59** |
| crowd allowance 0.70 | 290 | **51** |
| physical gate `r + d·√n` | 316 | **61** |

Every one reduces interruptions 21-28% and raises rivalries 13-23%. The cause is **time**: an earlier arrival means
the activity starts earlier and spends its full duration standing inside `contact_radius` instead of walking. Same
contact radius, more exposure ticks.

That is a genuine trade the **host** should make, not a defect the crate should remove silently — so the crate
reports it instead.

## A note on the arithmetic's honesty

Hex packing says 1.5 m seats 4.6 bodies, but ORCA does not pack around a point; everyone aims at the centre and
queues. So the seat count is a **lower bound on the crowding**, and the warning says "about" rather than
pretending to simulate. Claiming a precise packing figure would have been the easy, wrong thing.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **62** binaries |
| clippy / docs / fmt | 0 errors |
| bite check | disabling the condition **fails** the test |

## Coverage

Real. The test captures the log sink, asserts the warning fires with the place name, resident count and seat count,
asserts it stays quiet for a home that fits, and fails with an empty capture when the condition is disabled.

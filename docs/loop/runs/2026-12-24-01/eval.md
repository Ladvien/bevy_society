# Eval

## Change

`Tuning::crowded_arrival_growth` (0.7 by default) widens a `Place`'s arrival boundary by one body diameter per
`sqrt(occupants)`, so the queue behind a packed disc counts as having reached the place.

## Before / after, three seeds

| seed | interrupted | `sleep` | friends | rivals |
|---|---|---|---|---|
| 7 | 401 → **329** (−18%) | 169 → **129** (−24%) | 15 → 18 | 28 → 42 |
| 42 | 417 → **369** (−12%) | 176 → **139** (−21%) | 19 → 14 | 25 → 35 |
| 11 | 438 → **383** (−13%) | 187 → **148** (−21%) | 14 → 9 | 30 → 40 |

**Consistent on every seed**: interruptions down 12-18%, `sleep` failures down 21-24%. Rivalries up by 10-14.

## Why this is shippable now and was not two iterations ago

The same fix, measured before the user's `bounded-ties` merge, took rivalries **38 → 61**. Under
`Tuning::affinity_ceiling` the same change takes them **28 → 42**, and against a baseline that has already halved
friendships (31 → 15). The clamp bounds the side effect.

That is worth stating plainly: my earlier blocker was correct at the time and is now obsolete, and I would have
shipped a warning-only iteration for a third time if I had trusted my own log instead of re-measuring.

## The cost, named

Rivalries rise on all three seeds. The mechanism is **time**: arriving earlier means the activity starts earlier
and spends its full duration standing inside `contact_radius` instead of walking. More exposure ticks, more tie
drift. `affinity_ceiling` caps how far that drift can go.

## Reversibility

`crowded_arrival_growth = 0.0` reproduces the shipped baseline **exactly** — 401 / 169 / 15 / 28 — and a test
asserts the guard case (an empty disc still refuses a walker at 2.2 m with the growth at its default). A host can
turn the behaviour off with one number.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **66** binaries |
| clippy / docs / fmt | 0 errors |
| determinism | `series.csv` identical across two runs |
| bite check | crowded test fails when the gate is removed |

## Coverage

Real, after five attempts. The first four end-to-end fixtures produced identical output with and without the
feature — they never crowded the disc enough. The shipped test asserts the gate directly, with a stationary walker
that has no `Destination`, which is what a person jammed behind a packed disc actually is.

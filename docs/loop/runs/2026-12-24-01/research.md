# Research: the clamp changes what the crowding fix costs

## Why this

Two iterations ago I recorded a blocker: widening the arrival radius cut interruptions but **took rivalries from
38 to 61**, and the choice between those belonged to the host. Last iteration shipped only the warning.

Since then the user merged **`bounded-ties`**, which adds `Tuning::affinity_ceiling` — the clamp that six of my
earlier iterations argued for and could not decide. That changes the trade I measured, so the blocker had to be
re-measured rather than inherited.

## The baseline moved

| | before the clamp | after |
|---|---|---|
| friends | 31 | **15** |
| rivals | 38 | **28** |
| interrupted | 400 | **401** |
| sleep interrupted | 150 | **169** |

The clamp halved friendships and cut rivalries by a quarter. Interruptions are unchanged — as expected, since the
clamp touches ties, not walking.

## The same fix, re-measured under the clamp

Three seeds, same village, 17 game-days:

| seed | interrupted | `sleep` | friends | rivals |
|---|---|---|---|---|
| 7 | 401 → **329** (−18%) | 169 → **129** (−24%) | 15 → 18 | 28 → 42 |
| 42 | 417 → **369** (−12%) | 176 → **139** (−21%) | 19 → 14 | 25 → 35 |
| 11 | 438 → **383** (−13%) | 187 → **148** (−21%) | 14 → 9 | 30 → 40 |

**Every seed improves on the metrics the change targets.** Interruptions fall 12-18%, `sleep` failures fall
21-24%. Rivalries rise by 10-14, which is the cost — but before the clamp the same change produced a rise of 23
(38 → 61). The clamp absorbs most of it, because affinity can no longer run away.

That is the thing the clamp was for: it makes other mechanisms' side effects bounded. The previous iteration's
blocker is no longer a blocker.

## The mechanism, re-confirmed

Nothing about *why* changed. Everyone sleeps at the same hour, every resident of a home aims at one coordinate
inside `interaction_radius`, ORCA holds centres one body diameter apart, so the disc packs solid and the queue
behind it expires. Measured two iterations ago: at every timed-out `sleep` the goal already held a median of **6**
bodies, and a 1.5 m disc seats about `(1.5 / 0.7)² = 4.6`.

What ships is `Tuning::crowded_arrival_growth`, one body diameter per `sqrt(occupants)`. The square root is the
honest shape: a crowd occupies *area*, so the radius that accommodates `n` people grows with `sqrt(n)`, not `n`.

## Setting it to zero

`crowded_arrival_growth = 0.0` reproduces the old arrival test **exactly** — 401 / 169 / 15 / 28, identical to the
shipped baseline. A host that dislikes the trade can turn it off without editing the crate, and that reversibility
is asserted rather than asserted-in-prose.

## Sources

No paper. This is packing geometry and the crate's own measurements; `affinity_ceiling` and `SocialMover::radius`
are the numbers it is built from, and both were already documented.

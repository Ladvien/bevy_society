# Research: two clocks, and the crate's silence about them

## What the iteration set out to do, and what it found instead

The log's #1 was `need-cycle`'s rate balance, and the previous iteration's blocker note was known to be
wrong. Re-measuring against the *starter pack* (rather than the test-local definition that produced the
wrong note) gave:

| need | falls below 0.35 | recovers above 0.7 | range |
|---|---|---|---|
| `social` | no | no | [0.364, 1.000] |
| `rest` | no | no | [0.495, 1.000] |
| `food` | **yes** | **yes** | [0.000, 1.000] |
| `fun` | **yes** | **yes** | [0.000, 1.000] |

So the scenario's observable **is** met by two of the four needs. The note's claim of a back-to-back
sleep loop was an artefact of a definition with no `Hours` gate; the starter pack's `sleep` is gated
`Hours(22, 6)` and ranked 1, and one person sleeps 10 times in 10.1 days.

## The defect that actually starves needs, measured

In the village, completion rates split cleanly by target kind:

| activity | target | started | completed | interrupted |
|---|---|---|---|---|
| `chat` + `gossip` | `Person` | 152 | **152** | **0** |
| `eat` | `Place` | 108 | 23 | **78** |
| `relax` | `Place` | 23 | 8 | **12** |
| `sleep` | `Home` | 108 | 56 | **44** |

Every `Person`-targeted activity completes; every `Place`- or `Home`-targeted one is mostly
interrupted. And **94 090 of 100 800 person-ticks (93%)** were spent travelling.

## Why, in ticks

The crate has two clocks at a walk:

- `SocialMover::speed` is metres per **real** second applied at the fixed step, so 1.4 m/s at
  `1/64 s` is `0.0219 m/tick`;
- `Tuning::travel_timeout_hours` is **game** hours, so at `seconds_per_day = 4` one tick is 5.6
  game-minutes and `h` hours buys `h · 10.7` ticks.

Measured directly:

| walk | ticks taken |
|---|---|
| 8.2 m | **308** |
| 13 m | **527** |

With the timeout at 40 game-hours (427 ticks) the 13 m corner *cannot* be reached — it times out about
100 ticks short. That is `needs-decision/travel-time-in-two-clocks.md`, open since 2026-10-25, and this
iteration does not decide it.

## What is not a decision: the silence

The crate knows the speed, the timeout, the day length and the tick. It can therefore compute how much
walking a timeout buys, and it says nothing. A host sets a short day, gets a village where nobody can
cross the square, and has no signal.

That is the same shape as the two warnings `BehaviorPlugin::finish` already carries — a definition
shorter than two ticks, and a definition that relies on an unscored field input. Both exist because the
crate could see a silent failure and chose to name it. This is the third.

## Literature

| Claim | Source |
|---|---|
| A simulator should name a configuration it can prove inconsistent rather than let it fail silently | the crate's own precedent: `warn_short_activities`, `warn_unscored_fields`; and the crate's rule "Misconfiguration gets a loud `error!`/`warn!`, never a panic" |
| Walking speed is metres per second, not per game-hour | the crate's `SocialMover::speed` doc and default 1.4 m/s, which is the ordinary human walking speed |

No new paper. This is an honesty fix about a number the crate already computes.

## Not fixed, and why

The 40 → 55 hour timeout in the evidence harness is a *host-side* setting, measured rather than guessed
(527 ticks needed, 587 allowed). It improves `fun`'s starvation from 55% to 35% of ticks and makes
`rest` visibly better, but the plot still hugs zero: travel was real and is not dominant. The next
iteration's item is the `Place`/`Person` completion asymmetry, which this iteration measured but did not
touch.

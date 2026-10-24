# Review

## Pass 1 — Behavior: does the warning fire when the thing it names is happening?

**GREAT.** The two cases it must separate are measured, not asserted:

| `seconds_per_day` | 10 m walk | mean activity | warning | measured travelling |
|---|---|---|---|---|
| 64 | **2.68 h** | 1.62 h | **fires** | 58.9% |
| 240 | 0.71 h | 1.62 h | silent | 46.8% |

End to end from the real harness: `--seconds-per-day 64` prints
*"a 10 m walk costs 2.68 game-hours, longer than the mean activity (1.62h)"*; `--seconds-per-day 240` prints
nothing. The threshold is not fitted to a toy — it fires on the project's **own** village day.

## Pass 2 — Zoom: read the plot closely

**GREAT.** `evidence/coupling.png`, left panel: four travelling-share traces over 16.4 game-days, same seed,
same people. The 64 s/day line (red) rides visibly above the 480 s/day line (blue) for the first ~600 days
and the gap persists; no oscillation at a tick boundary, no clipping at 0 or 100, no dead agents. Right
panel: travelling falls monotonically 58.9 → 50.4 → 46.8 → 43.9 as the knob turns, and the need mean rises
0.545 → 0.624 — the two move in opposite directions, which is the finding: **the knob trades walking for
needs**, and no host would guess that from *"seconds of real time that make one game-day"*.

The needs curve flattens past 240 (0.621 → 0.624), so the damage is concentrated in the **short-day** end —
exactly where the warning fires.

## Pass 3 — Fresh eyes: the narrative

**GREAT.** One sentence to a host:

> Your walking speed is in real seconds and your activity durations are in game hours, so the number that
> says how fast you watch a day also decides how much of it people spend walking — at 64 s/day it is 59% of
> their lives, and they reach the tavern after the meal is over.

And the crate now says that at startup, with the arithmetic, instead of leaving it to be discovered in a plot
two iterations later — which is what happened here.

## Honest limit

This **reports** the coupling; it does not reduce it. The 15-point lever is still turned by `seconds_per_day`
until `travel-time-in-two-clocks.md` is decided. Recorded as such in the warning text and the decision file.

## Verdict

Three GREAT. Shipped.

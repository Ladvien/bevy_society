# Evaluation: a label is a latitude

Instrument: `examples/evidence.rs` at `village-evening`'s own settings — 24 people, seed as stated,
`seconds_per_day 64.0`, `start_hour 15.0`, `travel_timeout_hours 4.0`, 67 200 ticks (16.4 game-days).
Before each run the harness was given `--label-hysteresis`, and `summary.json` gained `label_changes`
(every `TieLabelChanged`, both directions). Cause analysis is a probe that reads each tie's affinity,
familiarity and trust **in the tick a label flipped**.

## Baseline: what a host is being sent

| seed | label changes | per game-day | one per | pairs changing > 10 times |
|---|---|---|---|---|
| 7 | 2 867 | 174.8 | **8.2 game-minutes** | 85 of 276 |
| 42 | 3 021 | 184.1 | 7.8 game-minutes | 104 of 276 |

Of seed 7's 2 867: **46%** had a one-tick affinity step above 0.005 (an event), **52%** were within
0.01 of the bar they crossed (noise), and the transitions are not alike:

| transition | count | median margin from its bar | what moved |
|---|---|---|---|
| `Stranger -> Acquaintance` | 828 | **0.0027** (familiarity) | 0.0002 — decay |
| `Acquaintance -> Friend` | 523 | 0.043 (affinity) | 0.013 — an exchange |
| `Friend -> Acquaintance` | 504 | 0.019 (affinity) | 0.013 — an exchange |
| `Acquaintance -> Rival` | 407 | 0.016 (affinity) | mixed |

`evidence/plots/causes.png`.

## The sweep, and the finding that changed the design

Three widths, three seeds, through the real harness (`evidence/plots/sweep.png`, `where.png`):

| band | seed 7 | seed 42 | seed 11 |
|---|---|---|---|
| **0.00** — changes / friends / rivals | 2 291 / 18 / 42 | 2 379 / 14 / 35 | 2 598 / 9 / 40 |
| 0.02, band on **every** bar | 1 486 / **20** / **48** | 1 499 / **16** / **42** | 1 623 / **10** / **49** |
| 0.04, band on every bar | 1 353 / 20 / 50 | 1 335 / 17 / 44 | 1 396 / 10 / 54 |
| **0.02, band on the stranger bar only** | **1 905 / 18 / 42** | **1 911 / 14 / 35** | **2 026 / 9 / 40** |

**The band on every bar is a behaviour change, not a bug fix.** It removes 35% of the messages and
raises rivals 42 → 48 and friends 18 → 20 at seed 7 — because it holds a friendship after the event
that ended it, which is a lie in the other direction. The measured cause says why: at the friend and
rival bars the flip *is* an event (a 0.013 one-tick step), and the label should change.

Applied to the stranger bar only it is inert where changes are real and decisive where they are not:

| seed 7, band 0.02 | no band | stranger-bar band |
|---|---|---|
| `Stranger -> Acquaintance` | 828 | **552** |
| `Acquaintance -> Stranger` | 96 | **1** |
| `Friend`/`Rival` label messages | 2 631 | **2 631** (identical) |
| friends / rivals | 18 / 42 | **18 / 42** |
| changes within 0.01 of the bar | 52% | **29%** |

552 is exactly 276 ordered pairs × 2 directions: **every pair becomes an acquaintance once, ever.**
The one demotion is at tick 6 016 of 6 400 in the test fixture — a pair that genuinely went unseen long
enough to decay past the band, which is the label telling the truth.

## What is not good enough

- **Churn is reduced by 17%, not removed.** 2 867 → 2 315 on the same instrument. The remaining friend
  and rival flips are real and should stay; what is left of the low end is arithmetic (552 is the number
  of pairs), not a defect. If a host wants fewer friend messages, that is a different question — whether
  a `Friend` should be held once made — and the measurement above says the answer there is *no*.
- **A wider stranger band buys nothing.** 0.04 and 0.08 on the stranger bar change `Stranger ->
  Acquaintance` by zero (552 at every width, because promotion is at the same bar) and leave
  `Acquaintance -> Stranger` at or below one. The default is 0.02 and width has no further effect; that
  is the one number in this change that is a **design default with a plateau around it**, which is the
  best a design default can be.
- **The friend count still rises over a long run** in every configuration (18 at 16 days, 12 by the
  `tmp_sweep` instrument's 78 days at a different tick length). That is the previous iteration's
  tension, unchanged and not this change's business.

## Determinism

Two runs at seed 42 with the band produce the same label sequence, asserted in
`tests/label_hysteresis.rs::the_band_is_deterministic`. The change adds no draw and no iteration order.

# Evaluation: a friendship is held through a shallow dip

Instrument: `examples/evidence.rs` at `village-evening`'s own settings — 24 people, `seconds_per_day 64.0`,
`start_hour 15.0`, `travel_timeout_hours 4.0`, `tick_secs 1/64`, 67 200 ticks (16.4 game-days). New flags
`--friend-band`; cause analysis is a probe that reads each tie's three `Friend` gates in the tick a label
flipped, plus the time since the pair's last exchange and last insult.

## What was wrong, and where it came from

| seed 7, 16.4 game-days | |
|---|---|
| friendships made / lost | **523 / 504** |
| median held | **0.36 game-days** (p10 0.02, p90 1.95) |
| the shelf a friend lives in | **0.05 wide** (`friend_affinity` 0.30, ceiling 0.35) |
| a median tie's affinity swing | **0.072** — wider than the shelf |
| positive tie-samples inside the shelf | **7%** (a bucket in the histogram, not a tail) |
| endings caused by the affinity gate | **461 of 504 (91%)**; trust 43, familiarity 0 |
| affinity at the moment of ending | clusters on **0.25** (102) and **0.30** (119) |

0.25 and 0.30 are exactly one `-0.05` below the ceiling and the bar, and `-0.05` is
`Tuning::rejection_affinity_penalty` — paid by anyone whose approach is refused, at a **38%** refusal rate.
**One refused approach, from a full shelf, ends a friendship.**

By cause (seed 7 / seed 42, 504 / 507 endings): **99 / 126** within six game-hours of a witnessed insult,
**349 / 329** within six game-hours of an exchange, and **56 / 52 with no interaction behind them at all**.
The insult-to-ending median is **0.4 game-hours** — prompt and explainable.

## The change, measured

`Tuning::friend_band` = 0.05: a `Friend` is kept while its affinity has dipped no further than 0.05 below
`friend_affinity`. Through the harness, seed 7, same ticks:

| | before (`friend_band 0.00`) | after (0.05) |
|---|---|---|
| friends / rivals | 18 / **42** | 22 / **42** |
| `became_rival` | 272 | **272** |
| label messages | 1 953 | **1 546** (−21%) |
| rumor reach | 0.542 | **0.542** |
| interrupted endings | 329 | **329** |

Probe, seeds 7 and 42:

| | seed 7 | seed 42 |
|---|---|---|
| endings | 504 → **242** | 507 → **251** |
| median held | **0.36 → 1.01** game-days | 0.38 → 0.90 |
| p90 held | 1.95 → **4.50** | 1.89 → 4.71 |
| ended by insult | 99 → **63** | 126 → **90** |
| ended by an exchange | 349 → **136** | 329 → 129 |
| ended by nothing | 56 → **43** | 52 → 32 |

Sweep at seed 7: `friend_band` 0.00 / 0.05 / 0.08 gives messages **1 953 / 1 546 / 1 467**, friends
**18 / 22 / 23**, rivals **42 / 42 / 42**. Seed 11: 2 080 → 1 710, friends 9 → 11, rivals 40 → 40.

`evidence/plots/friend-band.png`.

## What is not good enough — stated plainly

1. **The drift endings only fall 56 → 43.** The band holds the *shallow* dips, and a drift ending that
   arrives with a deeper drop still ends the friendship. Fixing those needs to know whether an *exchange*
   happened, which no field on `Tie` records: `last_contact_day` is stamped on mere proximity, not
   conversation. That is a save-format change → `needs-decision/which-tie-field-says-someone-spoke.md`.
2. **Refusal-driven endings are only partly absorbed.** `-0.05` against a band of 0.05 means a refusal
   from a full shelf still lands exactly on the bar. A wider band holds more and creates friends (0.08 →
   23); the choice of 0.05 is where the friend count moves least.
3. **Friendships are still short in absolute terms.** 1.01 game-days at the median is better than 0.36 and
   nothing like a relationship measured in months. The cause is upstream of the label: the refusal penalty
   is a large step against a tight ceiling, and that is `needs-decision/a-deed-costs-what-the-crowd-saw.md`
   territory.
4. **A plan that did not survive contact.** The plan proposed a grace window on `last_contact_day` to
   distinguish "quiet" friendships. Measured, `last_contact_day` moves on proximity, so it cannot tell
   "nothing happened" from "they stood in the same square", and the arm was dropped rather than shipped on
   a guess.

## Determinism

Asserted in `tests/label_hysteresis.rs::the_band_is_deterministic`. No draw, no iteration order, no new
state: the band reads two existing values.

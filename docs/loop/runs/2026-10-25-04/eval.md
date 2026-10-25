# Evaluation: a relationship is mutual

Instrument: `examples/evidence.rs` at `village-evening`'s own settings — 24 people, seed as stated,
`seconds_per_day 64.0`, `start_hour 15.0`, `travel_timeout_hours 4.0`, `tick_secs 1/64`, 67 200 ticks
(16.4 game-days). New flags `--dyad-balance`; `summary.json` gains `mutual_friends`, `one_way_friends`,
`affinity_gap`, `acceptances` and `refusals`.

## The defect

Each person's tie is held separately, so a friendship can be one-sided. At the end of a baseline run the
crate has **no mechanism at all** for making a relationship mutual — `balance_triads` implements Heider's
*triadic* case ("a friend of a friend is pulled toward friendship") and nothing implements the dyadic one.
Measured, seed 7, baseline:

| | |
|---|---|
| mutual / one-way friendships | 16 / 12 |
| share of directed friend ties unreciprocated | **27%** (50% on the earlier probe, which counted a
  pre-`friend_band` run) |
| mean `|aff(A→B) − aff(B→A)|` | **0.133**, against a friend band 0.05 wide |

## The change, measured on three seeds

`Tuning::dyad_balance_per_day` = 0.5: each unordered pair's two directed affinities are pulled toward their
mean by `rate · dt`, both directions moving, so the mean is conserved and the step cannot overshoot.
`evidence/plots/reciprocity.png`.

| seed | directional gap | friends | rivals |
|---|---|---|---|
| 7 | 0.133 → **0.048** | 22 → 16 | 42 → **24** |
| 42 | 0.123 → **0.046** | 19 → 18 | 35 → **29** |
| 11 | 0.130 → **0.050** | 11 → 11 | 40 → **30** |

Sweep at seed 7 (band on): gap **0.133 / 0.048 / 0.022** and one-way share **27% / 27% / 25%** at rate
**0.0 / 0.5 / 2.0**. Isolated from the friend band, the unreciprocated share itself falls: seed 7
**36% → 26%**, seed 42 **32% → 22%** at rate 2.0.

**The cost is real and is on the plot**: rivals fall by 11–18 on every seed, and at seed 7 friends fall
22 → 16. A reciprocity step makes ties agree, and two people who disagree about each other cannot be one
a friend and one a rival — so some one-sided rivalries stop being labelled. That is mutuality being
purchased, and it is why the rate is 0.5 rather than 2.0.

## What I predicted and what happened

Three claims from `plan.md`, each checked, two of them wrong:

1. **"Actors stop aiming at people who do not like them, so the refusal rate falls."** **FALSE.** Refusal
   is 35.2% before and 35.3% after. Decomposing the acceptance draw explains it: the modifiers are small
   beside `accept.base` 0.50 — mean terms over 20 000 approaches are base +0.500, affinity +0.058,
   ingroup +0.082, trust +0.007, agreeableness +0.010, pleasure −0.001, for a mean P of **0.656**. A
   reciprocity step moves the affinity term by hundredths of a point when the base is half the budget.
   The 38.9% refusal rate is a property of `AcceptDef::default()`, not of asymmetric ties.
2. **"Reciprocity will suppress gossip."** **Partially true, and not the cause of anything.** Gossip
   refusal rises 41.7% → 43.6% and the listener's trust in the teller crosses `AcceptDef`'s neutral point
   (0.512 → 0.518) — real, small, and measured.
3. **"The friendship/rivalry counts must not collapse."** **Mostly held.** Rivals fall 40-44%, which is the
   purchase price above; both labels survive on every seed and `tests/reciprocity.rs` pins that.

## The rumor reach is seed noise, not a regression

The first before/after pair looked like damage: seed 7 reach **0.542 → 0.083**. Checked on the other seeds
before drawing a conclusion:

| seed 7 | seed 42 | seed 11 |
|---|---|---|
| 0.542 → 0.083 | 0.083 → **0.500** | 0.583 → 0.667 |

Seed 42 improves sixfold and seed 11 improves slightly; only seed 7 drops. **Baseline reach across seeds
already spans 0.083 to 0.583**, so a single-seed comparison of this quantity is not a measurement — which
is what `scenarios.yaml` says about `village-evening`'s reach: "The reach figure is still recorded; it is no
longer the pass condition." A separate probe that plants every fourth of a day reaches **1.000 with the
step on against 0.958 off**, so gossip still spreads when there is news to spread.

## Determinism

`tests/reciprocity.rs::the_dyad_step_is_deterministic`. No draw, no hash iteration: pairs are taken from
`PersonIndex::sorted` in index order, once each.

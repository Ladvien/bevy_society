# Review: a relationship is mutual

Three passes, each with a written critique, before the verdict. Evidence is under `evidence/`.

## Pass 1 — Behaviour: does the change do what the paper and the scenario say?

**GREAT, for the claim that survives measurement.** Heider's balance theory says "in a dyadic relation,
balance basically means reciprocity: feeling positive about a person who doesn't reciprocate this feeling
is unbalanced" (via Schweighofer & Schweitzer, corpus `10.18564/jasss.5323`). The crate implemented the
triadic half and not the dyadic half; `balance_dyads` is that half.

| seed | directional gap | unreciprocated share (band off) | rivals |
|---|---|---|---|
| 7 | 0.133 → **0.048** | 36% → **26%** | 42 → 24 |
| 42 | 0.123 → **0.046** | 32% → **22%** | 35 → 29 |
| 11 | 0.130 → **0.050** | — | 40 → 30 |

The gap closes on **every** seed, monotonically in the rate (0.133 / 0.048 / 0.022 at 0.0 / 0.5 / 2.0).

## Pass 2 — Zoom: what did the change actually do, as against what I said it would?

**GREAT for the measurement, and it overturned two of my own predictions.** This is the pass that earned
the iteration.

- **"The refusal rate will fall."** It does not: 35.2% → 35.3%. Zooming into the acceptance draw shows
  why. Over 20 000 approaches the mean terms are `base +0.500`, `ingroup +0.082`, `affinity +0.058`,
  `agreeableness +0.010`, `trust +0.007`, `pleasure −0.001`, mean P **0.656**. The base is half the budget;
  reciprocity moves the affinity term by hundredths. The refusal rate is a property of
  `AcceptDef::default()`, and the plan's test 4 would have failed the change had I written it as a pass
  condition.
- **"The rumor reach fell."** The first before/after pair read 0.542 → 0.083 and looked like damage to a
  scenario this change did not target. Seed 42 goes **0.083 → 0.500** and seed 11 **0.583 → 0.667**.
  Baseline reach already spans 0.083-0.583 across seeds, so one seed is not a measurement of it — which
  `scenarios.yaml` states. The probe that plants more aggressively reaches 1.000 on, 0.958 off.
- **"A dyad step flattens the village into indifference."** Partly. Rivals fall 40-44% because two people
  who disagree about each other cannot be one friend and one rival. That is the price, it is on the plot,
  and it is why the rate is 0.5 and not 2.0.
- **A one-way friendship is mostly a *trust* asymmetry, not an affinity one.** Zooming into the six one-way
  pairs at rate 2.0: five differ only on `trust` (0.478, 0.438, 0.373, ... against the 0.50 bar) while
  affinity agrees to within 0.01. The dyad step converges affinity and cannot touch the thing that actually
  discriminates. Recorded as the next item rather than papered over.

## Pass 3 — Fresh eyes: a different seed, and is it explainable?

**GREAT.** Three seeds, and the one number that is the point of the change is right on all of them. The
narrative needs no new concept for a host: *two people now come to feel the same way about each other;
where they still disagree, one can be a friend and the other not.* `tests/reciprocity.rs` pins both halves
of that, plus determinism and the exact `0.0` off-switch.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**, and it is why the changelog claims reciprocity and not a refusal-rate
improvement. Pass 3 **GREAT**.

## What a stranger would still complain about, correctly

- The refusal rate is untouched, and that is the number a host feels when its villagers decline each other
  all day. It is `accept.base` 0.50 and it is now the top remaining item, with the decomposition above as
  its evidence.
- Friendships and rivalries both get rarer. A host that liked the old count has one `Tuning` field to set
  to `0.0`.

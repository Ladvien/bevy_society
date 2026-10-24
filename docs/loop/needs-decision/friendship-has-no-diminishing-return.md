# Should forty-five conversations make you friends?

## The problem, measured

`village-evening`, seed 7, 24 people, 17 game-days:

| tick | accepted interactions | friend-ties | max affinity |
|---|---|---|---|
| 9 599 | 1 352 | **25** | **0.999** |
| 19 199 | 2 586 | 40 | 1.000 |
| 28 799 | 3 947 | 65 | 1.000 |
| 38 399 | 5 313 | 80 | 0.999 |
| 47 999 | 6 577 | 104 | 1.000 |
| 57 599 | 7 844 | 113 | 1.000 |
| 67 199 | 9 347 | **115** | 1.000 |

Friendships **grow linearly and never saturate**: 15, 25, 15, 24, 9, 2 new friend-ties per 9 600 ticks. The
village is still accreting friends when the run ends, and affinity is pinned at **1.000** — the clamp, not an
equilibrium.

## What it is not

Three hypotheses were tested by sweeping the rate and reading the friend count:

| knob | friends |
|---|---|
| `mere_exposure_per_day` 0.05 (shipped) | **22** |
| 0.02 | 15 |
| 0.01 | 15 |
| 0.005 | 16 |
| `balance_rate_per_day` 0.5 (shipped) | **22** |
| 0.05 | 25 |

**Nearly flat.** Exposure and structural balance are second-order; neither sets the friend count. (Exposure's
own equilibrium is `0.05 / (0.05 + 0.01) = 0.833`, comfortably above the `Friend` threshold of `0.4`, so
proximity is *sufficient* for friendship — but it is not what decides how many friendships a village has.)

## What it is

**The number of exchanges, with no diminishing return.** Measured on a single pair:

| | value |
|---|---|
| interactions before the tie reads `Friend` | **45** |
| affinity after 57 interactions | **0.73** |

`TieLabel::classify` asks for affinity `≥ 0.4` and familiarity `≥ 0.3`. Familiarity from exposure is
`saturating` (it uses `1 - f`), but **affinity from interaction is not**: each accepted exchange adds its
`TieDelta::affinity` (0.1 for a compliment, less for chat and gossip), relaxation removes 0.01 per game-day,
and nothing makes the *n*-th conversation worth less than the first.

So friendship is a monotone function of contact volume. Triple the contact and you triple the friends — which
is exactly what happened twice:

| mechanism | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| a `help` activity (2026-12-03) | 81 | **0** |
| a contact-triggered exchange at 3 h (2026-12-08) | **267** | **0** |
| …at 12 h | 182 | 3 |
| …at 48 h | 72 | 12 |

267 friend pairs in 24 people is **97% of all possible pairs**. Two independent mechanisms, one cause.

## Why it is a decision

The fix is a model change to how affinity accumulates, and three shapes disagree about what friendship is:

### 1. Saturate interaction affinity

`affinity ← affinity + gain · (1 − |affinity|)`, the same form exposure already uses. Then the *n*-th
conversation is worth less, and a friend is someone you have spent real time with rather than someone you
have met often.

- **For:** one line, reuses a form already in the crate, and it targets exactly the missing term.
- **For:** it would have made both failed mechanisms safe, because neither would have pushed past a ceiling.
- **Against:** changes how every tie forms, so it is a model change for every host.

### 2. Raise the `Friend` threshold and make familiarity the gate

Keep affinity linear but require familiarity `≥ 0.8`, so a friend is someone known for a long time.

- **For:** no new formula; the threshold is already a documented constant.
- **Against:** familiarity saturates too (`1 − f`), so the gate would eventually open for everyone in contact
  — the same failure, delayed.

### 3. Cap the number of friends per person

A Dunbar-style cap already exists (`max_ties: 150`, `trim`) but it trims by *familiarity*, not by label, and
150 is far above a village of 24.

- **For:** bounded by construction.
- **Against:** it bounds the *count* without changing what a friendship means; the village would have 150
  friends instead of 267.

## Recommendation

**Option 1.** It is the only one that changes the shape rather than the number, it is the form the crate
already uses for exposure, and it is the term whose absence explains both failed mechanisms rather than one.

## What ships now

The `mere_exposure_per_day` doc corrected: it **cited Zajonc 1968 as if the paper supplied the rate**, and
Zajonc supplies only the mechanism. The doc now states the equilibrium (`0.833` against a `0.4` threshold), the
three sweeps that show exposure is *not* what sets the friend count, and the measured 45-interaction result.

No rate was changed. The crate is at friends 22 / rivals 33.

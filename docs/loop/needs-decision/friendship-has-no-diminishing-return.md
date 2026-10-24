# Should forty-five conversations make you friends?

> **The sixth shape works, and costs mood. Measured 2026-12-13 — see "The clamp that bounds it" at the end:
> affinity clamped to `[-0.35, 0.35]` takes friends **22 → 15** and rivals **33 → 18** (the first shape in six
> to bound the network without inverting it), at a cost of mood **-0.054 → -0.078**.**
>
> **Five mechanisms were tested before that and none worked — see "Five failures, one cause" at
> the end, which is the most useful thing in this file.**
>
> **Option 1 was tested on 2026-12-10 and does not work.** Saturating the gain — the recommendation below —
> leaves the clique intact: with a contact exchange at 3 h, friends are **260** under saturation against
> **267** without it, and rivals are **0** in both. Saturation only slows the *approach* to affinity 1; it does
> not lower the ceiling, and 45 exchanges cross `0.4` on either curve. The recommendation is retracted; see
> "Why saturation fails" at the end.

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

## Why saturation fails, measured

`affinity ← affinity + gain · (1 − |affinity|)` was the recommendation. Built and measured on
`village-evening`:

| configuration | friends | rivals |
|---|---|---|
| shipped (no saturation, no contact talk) | **22** | **33** |
| saturation alone | 22 | 38 |
| shipped + contact talk at 3 h | **267** | **0** |
| **saturation + contact talk at 3 h** | **260** | **0** |

Saturation alone is nearly neutral (22 → 22), which looked promising. It does not survive contact: **260 against
267** is the same clique.

The reason is arithmetic. Saturation makes affinity approach 1 asymptotically rather than linearly, but both
curves pass `0.4`, and both pass it well within 45 exchanges. A saturating gain changes *when* the threshold is
crossed, not *whether*. To stop a clique the model needs something that keeps affinity **below** the threshold
for most pairs — a lower asymptote, a threshold above the asymptote, or a gate that contact cannot satisfy.

## Options that remain

1. **A lower asymptote for interaction affinity.** `affinity ← affinity + gain · (K − |affinity|)` with
   `K < 0.4`, so no number of exchanges with one person is enough on its own; the threshold then needs a second
   input (trust, respect, or a shared faction). Untested.
2. **A higher threshold, gated on something contact cannot supply.** `Friend` at affinity `0.4` *and* trust
   `0.6`, say. Trust moves far more slowly than affinity, so it would bound friendship without touching the
   affinity curve.
3. **Accept the clique and bound the count.** `trim` already exists at `max_ties = 150`; a village of 24 would
   need a far lower cap, which changes what Dunbar's number means here.

**Option 2 is the one to try next**, because it uses a quantity the crate already models and already moves
slowly. Saturation was the cheapest fix and it is now known not to work, which is worth having measured rather
than assumed.

## Five failures, one cause

Every mechanism tried against "everyone becomes friends" has failed, and each failed for the same reason.

| mechanism | measured | tried |
|---|---|---|
| a `help` activity, so positive deeds exist | rivals **44 → 0** | 2026-12-03 |
| a contact-triggered exchange at 3 h | friends **22 → 267** | 2026-12-08 |
| saturating **affinity**, the recommended fix | **260** with contact | 2026-12-10 |
| a **trust** gate on the `Friend` label | **267** — no effect | 2026-12-11 |
| saturating **all three** accumulators (affinity, trust, respect) | 23 alone, **268** with contact | 2026-12-11 |

### Why the trust gate did nothing

Measured distributions at the end of a run:

| | affinity | trust | familiarity |
|---|---|---|---|
| friend-ties (115) | **0.792** | **0.480** | 0.907 |
| other ties (53) | −0.223 | 0.068 | 0.908 |

Trust *does* separate them, sevenfold. But `chat` pays `trust: 0.05` per accepted exchange to both parties, so
contact talk at an 8-a-day rate adds `+0.4/day` and trust saturates exactly as affinity does. **A gate on trust
is a gate on a quantity the same mechanism saturates.**

### Why saturation cannot work

Saturating bounds a variable at **1.0** — above every threshold in `classify` (0.4, 0.3, 0.2, 0.1). It changes
how fast the bound is approached, not whether the threshold is passed. Measured twice: affinity alone (260) and
all three (268).

### The structural conclusion

**No tie variable can be below a threshold for a pair that has had contact.** Affinity, trust, respect and
familiarity all accumulate, and every exchange supplies all of them, so any level-based gate is passed by
contact volume alone.

What remains is not a level but a **shape on the input**:

1. **Bound the accumulator below the threshold.** e.g. affinity asymptote `K = 0.35` with `Friend` at 0.4, so no
   amount of contact with one person is sufficient and friendship needs a second input. Untested.
2. **Gate on `max_ties`,** lowering the cap from 150 to something a village of 24 could respect. Bounds the
   count rather than the meaning.
3. **Do not add contact exchanges.** Restate the two scenarios that wanted them, and accept that the decision
   model's meetings happen by targeting a person, not by proximity.

**Option 1 is the only one left that changes the shape**, and it is untested. Two iterations have now been spent
testing shapes that bounded at the wrong place; this one bounds *below* the threshold, which is the property
none of the five failures had.

## The clamp that bounds it, 2026-12-13

The previous iteration put the label thresholds on `Tuning`, which made the question measurable without touching
code. Two sweeps found where the lever actually is.

### The bar is not the lever

Raising `friend_affinity` barely moves the count, because affinity reaches **1.0** for anyone in contact — any
bar below 1.0 is eventually passed:

| `friend_affinity` | friends | rivals |
|---|---|---|
| 0.4 (shipped) | **22** | 33 |
| 0.55 | 18 | 33 |
| 0.70 | 18 | 33 |
| 0.80 | 16 | 33 |

### And it is not a runaway either

A four-times-longer run shows the friend count **saturating** — it is not an ever-growing clique:

| game-day | friend-ties |
|---|---|
| 8.2 | 77 |
| 16.4 | 115 |
| 24.6 | **122** (peak) |
| 32.8 | 121 |
| 41.0 | 120 |
| 57.4 | 109 |

So the problem is a **saturating system with a high asymptote** — 122 of 276 pairs, 44% of the village — not an
unbounded one. The asymptote is where affinity settles, and affinity settles at **1.0**.

### The shape that works

Clamp affinity to `[-K, K]` with `K = 0.35`, **below** the bar, so contact alone can never make a friend and a
second input is needed. Measured:

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | **22** | **33** | -0.054 |
| clamp `±0.35`, no second input | **0** | 18 | -0.078 |
| clamp `±0.35` + trust `≥ 0.55` + bar 0.25 | **15** | **18** | -0.078 |
| clamp `±0.35` + trust `≥ 0.55` + bar 0.30 | **12** | **18** | -0.078 |

**Both tie counts fall** — friends by a third, rivals by nearly half — for the first time in six attempts. It
also holds at three seeds (4–6 friends at the strictest setting).

### The cost, attributed

Mood falls **-0.054 → -0.078**, and the two bar settings above give *identical* mood, so the cost is entirely
the **clamp**, not the friend count. The chain is visible: affinity gated activities (`chat` at
`Logistic(8.0, 0.4)`, `compliment` at `Logistic(8.0, 0.7)`) see a lower affinity, so fewer contacts happen, so
fewer `Liking` exchanges, so lower mood.

### The one thing not to repeat

Scaling by the asymptote — `delta · (1 − |a|/K)` — is **asymmetric**: a negative delta steps past `-K` to the
outer clamp while a positive one decays smoothly. Measured, rivals rose **33 → 48** instead of falling. A plain
**clamp** to `[-K, K]` bounds both directions and is the version above.

## Recommendation, revised

**Clamp ±0.35 with a trust second input, and accept the mood cost** — or fix the mood cost first by raising
`mere_exposure_per_day` back up, since exposure is a separate path to affinity that the clamp now also bounds.
Untested: whether a higher exposure rate recovers mood without restoring the clique.

## The clamp's real cost is not mood — it is the opinion model, 2026-12-15

Last iteration attributed the clamp's cost to mood (`-0.054 → -0.078`) via fewer affinity-gated contacts, and
recommended recovering it by raising `mere_exposure_per_day`. That recommendation was tested and is **wrong in
kind**: the clamp's largest effect is on a different model entirely.

### The mechanism, measured

`compliment` gates on `Affinity` at `Logistic(8.0, 0.7)`. At the clamp ceiling of `0.35` that curve scores
**0.057**, against **0.673** at the unclamped mean of `0.79` — compliments become **12× less likely**. That is
why mood fell, and it is a curve/gate mismatch, not a missing rate.

Moving the gate down to `Logistic(16.0, 0.25)` to match the ceiling **fixes the mood**:

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | 22 | 33 | **-0.054** |
| clamp 0.35, gate unchanged | 0 | 10 | -0.078 |
| clamp 0.35 + gate at 0.25 | 0 | 10 | **-0.045** |
| clamp 0.35 + gate 0.25 + `friend_affinity` 0.30 + trust ≥ 0.5 | **16** | **10** | **-0.045** |
| …with `friend_affinity` 0.25 | 24 | 10 | -0.045 |

So the trade can be made **good on both axes**: fewer friends than the shipped 22, a third of the rivals (33 →
10), and mood *better* than baseline (-0.045 against -0.054).

### But it breaks the opinion model

`tests/opinion_convergence.rs` fails:

| | before | with the clamp |
|---|---|---|
| opinion spread, 12 000 ticks | 0.5106 → **0.24** (53% fall) | 0.5106 → **0.3813** (25% fall) |

The scenario asserts a fall of at least a third. Deffuant influence is **weighted by tie strength**, so halving
the affinity ceiling roughly halves how fast opinions converge. That is a coupling across three models — ties to
opinions — that none of the six earlier attempts touched, because none of them changed the affinity *scale*.

### What that means

The clamp is not one number. Changing the affinity scale **re-times the opinion model**, and 12 000 ticks was
chosen because unclamped convergence crosses a third around 9 000. Any clamp needs either a longer
`ticks:` in `opinion-converges-but-not-entirely`, or a compensating increase in `influence_mu`.

**This is the thing to decide**, and it is bigger than the earlier trade: bounding the tie network means
re-tuning the opinion scenario, because influence and liking share one variable.

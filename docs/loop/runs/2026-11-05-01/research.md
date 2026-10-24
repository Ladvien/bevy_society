# Research: an activity's odds must not scale with its number of targets

## The problem, measured

Last iteration recorded need starvation as the top item and could not say why. This iteration found
the cause, and it is not the rates.

**Starvation scales with the number of neighbours.** Everyone on one plaza (zero travel), every rate
fixed, only the population changing:

| people | `food` need at 0 | `fun` need at 0 |
|---|---|---|
| 1 | 0.0% | 5.9% |
| 4 | 4.5% | 5.4% |
| 8 | 9.6% | 28.2% |
| 16 | 11.4% | **43.4%** |

A need starving more as more people are around, with every definition unchanged, is not a tuning
problem. It is a bug in how an option is counted.

**The share of starts.** Starter pack, 8 people on one plaza, 1 500 ticks:

| activity | starts | share | target kind |
|---|---|---|---|
| gossip | 582 | 41.0% | `Person` |
| snub | 556 | 39.2% | `Person` |
| work | 78 | 5.5% | `Place` |
| sleep | 48 | 3.4% | `Home` |
| compliment | 41 | 2.9% | `Person` |
| eat | 38 | 2.7% | `Place` |
| relax | 38 | 2.7% | `Place` |
| chat | 28 | 2.0% | `Person` |
| wander | 10 | 0.7% | `Wander` |

`snub` and `gossip` — both satisfying **no need at all** — took 80% of every decision, while `eat`
and `relax`, the only activities that feed `food` and `fun`, took 2.7% each.

## Cause

`targets_for` (`behavior.rs:908`) gives a `Person` activity **one candidate per person in
perception**, and a `Place` activity one per matching place. Selection then drew over every
candidate:

```rust
let total: f32 = viable.iter().map(|c| c.weight * c.score).sum();
let mut roll = rng.0.unit() as f32 * total;
for candidate in &viable { roll -= candidate.weight * candidate.score; ... }
```

So an activity's selection mass was `weight · score · (number of eligible targets)`. `gossip` with
eight neighbours held 8/9 of the mass of the social options; `relax` held 1/9 — and `snub`, which
needs only one grudging neighbour, held nearly all of the rest.

## What the source says

Dill's chapter says the draw is over **options**, and an option is an action:

> "Add up the total weight of all valid options (i.e., those with a weight greater than 0)."
> "Rank uses absolute utility to divide the options into categories, and we only select options from
> the best category. Weight is used to evaluate options within the context of their category."
> — *Dual-Utility Reasoning*, Game AI Pro 2 ch. 3 (corpus doc
> `gameaipro2-ch03-dual-utility-reasoning`, and `GameAIPro2_Chapter03_Dual-Utility_Reasoning`)

An action with eight eligible targets is **one option**, not eight. Nothing in the algorithm makes an
action more desirable for having more places it could be done, and treating it that way is a
selection bug, not a modelling choice: it means a person surrounded by neighbours is *less* likely to
eat.

Dill also gives the remedy for an over-favoured option, and it is the check this iteration used: an
option that should not be selected is marked with weight zero. That is what `tier_cutoff` already
does for low scorers; it cannot help when the bias is in the *count*.

## What believable output looks like

With the bias removed, the same layout should satisfy needs at rates set by the definitions rather
than by the crowd. Concretely: `fun` decays 0.8/day and `relax` pays 0.4/h for 1.5 h = 0.6 per use,
so one use a day plus the 17–23 window should hold it well above zero, and the share of starts should
follow the considerations, not the neighbour count.

Measured after the fix, 24 people, four places, zero travel: `fun` at 0 on **4.8%** of ticks (was
64.7%) and `food` on **1.9%** (was 39.3%).

## Not this iteration's problem, and now cleanly separated

The village with real distances still starves, because the two-clocks mismatch
(`needs-decision/travel-time-in-two-clocks.md`) means walks rarely finish. Same 24 people, same
definitions, four places at the origin instead of metres apart:

| | need0 | need1 | need2 | need3 |
|---|---|---|---|---|
| village, places metres apart | 0.105 | 0.173 | 0.070 | 0.049 |
| village, places at the origin | **0.869** | **0.810** | **0.554** | **0.526** |

Choice is fixed; travel remains, and that is the operator's decision to make.

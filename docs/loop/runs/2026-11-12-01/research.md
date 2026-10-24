# Research: a need whose source cannot meet its decay

## The measurement this starts from

The previous iteration made travel work, and every need recovered except one:

| need | at zero, after the walk fix |
|---|---|
| social | 2.0% |
| rest | 0.8% |
| food | 12.8% |
| **fun** | **35.3%** |

`fun` was the log's #1 and looked like the obvious next target.

## What the arithmetic says

Every need decays per game-day and every activity supplies `satisfies · duration_hours` per use:

| need | decay/day | best source | pays per use | uses/day to break even |
|---|---|---|---|---|
| social | 1.00 | `chat` | 0.36 | 2.78 |
| rest | 0.70 | `sleep` | 1.20 | 0.58 |
| food | 0.90 | `eat` | 0.90 | 1.00 |
| **fun** | **0.80** | **`relax`** | **0.60** | **1.33** |

So `fun` needs **1.33** uses a day. Measured supply, 24 people over 32.8 game-days: **0.10** per person
per day. A **92% shortfall**.

Three candidate causes were tested rather than assumed:

| hypothesis | test | result |
|---|---|---|
| `satisfies` does not pay | one 1.5 h `relax`, fun sampled at arrival and at end | **+0.6000**, exactly `0.4 × 1.5`. The payout is correct |
| the 17–23 window is the binding constraint | remove the `Hours` gate | relaxes 79 → 167, `fun` mean 0.053 → 0.092. A constraint, **not** the binding one |
| capacity | well holds 6, window holds 4 batches of 6 | not binding |

What is binding is **choice**: `relax` competes with `chat`, `gossip` and `snub` for the same evening, and
it targets a `Place` so each use costs a walk plus 16 ticks of sitting, against a `chat`'s 3 ticks with
the other person walking to *you*.

## The fix that was tried and reverted

Sizing `relax` to its need — `satisfies 0.4 → 0.55`, so one evening session pays 0.825 and covers the
day's 0.8 — was implemented, tested and measured:

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **18.8%** |
| `fun` mean | 0.061 | **0.051** |
| `social` mean | 0.124 | 0.112 |
| `food` mean | 0.101 | 0.085 |
| **mean of all four** | **0.127** | **0.118** |

`fun`'s zero fraction halves, which is the thing the log named. But its *mean* falls and every other
need falls with it: the village is **over-subscribed**, 24 people against four places and one evening,
so raising one supply crowds out another. The net is negative.

That is a zero-sum reallocation dressed as a fix, so it is **reverted**.

## What ships instead: name the imbalance

`BehaviorPlugin::finish` gains `warn_unbalanced_needs`, beside the three warnings already there. It warns
when a need's best source needs more than `MAX_USES_PER_DAY = 4` uses a day to break even.

Four, not one, because the crate cannot see a source's *cost*: `social` needs 2.78 chats a day and is
perfectly served, since a chat is three ticks and targets a person. The check catches only what no
activity shape makes plausible.

## Literature

| Claim | Source |
|---|---|
| Well-being depends on needs being *satisfied*, and need **frustration** is a distinct state, not merely a low level | Vansteenkiste & Ryan 2013, DOI 10.1037/a0032359: SDT "argues that satisfaction of the basic psychological needs … fosters immediate well being … whereas need frustration evokes ill-being" |
| A need left unmet does not simply wait; it drives substitute and maladaptive behaviour | same |

That is the reason this is worth naming rather than leaving to a plot: a society where one need is
permanently frustrated is not a society with a low value, it is a different kind of society. The crate
should say so at startup.

## Not done

The real fix is not a rate. It is either a second source for `fun` (a `Person`-targeted leisure activity,
so the cost of a use drops to a `chat`'s), or fewer people competing for the same evening. Both are
definitions-file work with a real design question behind them — what should a village's evening *be* —
and neither is a number I can justify from a paper.

# Research: a person walking to bed is reported asleep

## How this was found

The log's #1 was `need-cycle`'s observable — a design question. Before writing it up I re-ran the village
and counted the activity mix, which is not something the loop had done:

| activity | share of person-ticks |
|---|---|
| **sleep** | **56.8%** |
| eat | 14.9% |
| work | 14.8% |
| relax, snub, compliment, gossip, play, chat, wander | 9.2% between them |

A village asleep 57% of the time is the brief's *"everyone doing the same thing"*. One 8-hour sleep a day
is 33%, and **350 sleeps over 24 people over 17 days** at 8 hours is **28.6%**. So the number could not be
right, and finding out why took four wrong hypotheses.

## The four that were disproved

| hypothesis | measurement that killed it |
|---|---|
| a `Place` activity overruns its duration | `end_activities` reports **`elapsed_h = 8.004`** on every sleep that ends, against a declared 8.0 |
| `satisfies` pays twice per tick | a 2-hour activity at 0.1/h paid **0.2010** — exact. (The first test read 1.000 because the need clamps at 1.0 and the rate was 1.0/h: the *test* was wrong.) |
| `best_telling`'s tie-break starves a fresh rumor | three tests written for it **passed with no code change** |
| a probe measuring overlapping sleeps | fixed with a queue; the `[0.00, 26.84]` range was the probe, and the real range is 8.00–28.00 |

## The one that held

A villager tracked tick by tick while its `Doing` said `sleep`:

```
   1: rest 0.087
   2: rest 0.087
  ...
  12: rest 0.085
```

`rest` is **falling at exactly the decay rate** — 0.7/day is 0.00017 a tick, and the observed change is
−0.00018 a tick. `sleep` pays 0.15 per game-hour, which is 0.00088 a tick, so arrival would make `rest`
*rise*. It falls. **The person is labelled `sleep` and is being paid nothing.**

`arrive_and_satisfy` pays only while `Doing::arrived_day` is `Some`. So the activity is named but not
begun, and the person is walking.

## The mechanism

`Doing` carries an activity and an `arrived_day`. Its own doc says:

> While `arrived_day` is `None` the person is **travelling**; the day it is set, the activity's needs and
> duration start counting.

But `Doing.activity` names the **target** from the moment it is chosen. Every reader — the HUD, the
evidence harness, a bark's placement, a script — sees a person sleeping while it walks.

The time scales make it common rather than rare. `travel_timeout_hours` is **20** at 64 s/day, which is
**3 414 ticks**; an 8-hour sleep is **1 365**. A person can walk toward bed for **two and a half times the
length of the sleep**, labelled asleep throughout. That is the 56.8%: `evidence/reported-asleep.png` shows
the reported-asleep count peaking at **22–23 of 24** every night, far above the red line of 8 that one
8-hour sleep a day allows.

## Why it is a decision

`Doing.activity` is the host-facing answer to "what is this person doing?", and the crate's own systems
read it: `activity_barks` places a bark at `Doing.target` and picks its line by activity, and
`report_to_triggers` reports it. Changing it changes what every consumer reads.
`needs-decision/what-doing-reports-while-travelling.md` states three options and recommends a reserved
`travelling` activity with the target kept in its own field, so no consumer has to remember a second
thing.

## Literature

No new model. The split between an action's *start* and its *run* is the vocabulary Game AI Pro 3 ch. 13
already gives the crate, and `docs/research/needs-and-utility.md` describes arrival as the point the
duration and payout begin. This iteration is about the crate reporting the first as if it were the second.

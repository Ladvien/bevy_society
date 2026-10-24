# Research: why the village's needs sit low, and which of them is actually wrong

The log's second item was "mean need level: `rest` decays 0.7/day while sleep restores at most
1.2 per night, so the village lives near 0.15". That is a hypothesis from arithmetic, not a
measurement. This iteration measured it per need, with the harness built last iteration.

## What the measurement said

Per need over 4200 ticks of `village-evening` at seed 7, before any change:

| need | mean | fraction of ticks below 0.35 |
|---|---|---|
| social | 0.083 | 0.96 |
| rest | 0.195 | 0.94 |
| food | 0.049 | 0.96 |
| fun | 0.042 | 0.95 |

All four are low, and all four are low *most of the time*, which is not what a need cycle
should look like — a cycling need spends its time spread across the range. The cause had to
be structural rather than a rate.

## The three findings

### 1. Social was the only need served solely by a one-shot effect

`social` decays at 1.0/day. Its only sources were interaction effects (`chat` +0.2,
`gossip` +0.15, one time each) — while `eat` paid food at **1.2 per game-hour** and `sleep`
paid rest at 0.15 per game-hour. `chat` and `gossip` are exactly the activities that serve
social, and both declared `satisfies: []`.

Measured cadence: social rose **420** times against **20 365** falls over the run — one rise
per ~80 ticks per person, where the decay needs one per ~51 ticks to break even. A need that
is 1.57× short of break-even decays to the floor and stays there.

**This one was wrong**, and fixing it took social from 0.105 to 0.225.

### 2. A duration is game-time and a walk is real-time

`ActivitySpec::duration_hours` is game hours; `SocialMover::speed` is metres per **real**
second and steering runs every `1/64 s`, so a walker covers `1.4 / 64 ≈ 0.022 m` per tick.

At `seconds_per_day = 4.0` one tick is 0.094 game-hours:

```
sleep       8.0 game-hours =  85 ticks      3.5 m walk = 160 ticks
work        4.0            =  43
relax       1.5            =  16
eat         0.75           =   8
chat        0.3            =   3
```

**Every activity is shorter than the walk to it.** Nobody can arrive before their activity
ends, so `satisfies` never runs at all for the short ones. This is the same clock mismatch
recorded in `docs/loop/needs-decision/travel-time-in-two-clocks.md` last iteration, seen from
its second victim.

The scenario works around it: villagers start within `interaction_radius` of their faction's
place, so arrival is immediate and the duration is what limits the activity. The crate is not
changed, because which clock walking lives in is the open decision.

### 3. `food` and `fun` being low is *correct*

`eat`'s consideration is `Need(food) Logistic(-10, 0.35)`. Evaluated:

| food level | consideration |
|---|---|
| 1.0 | 0.0015 |
| 0.8 | 0.011 |
| 0.6 | 0.076 |
| 0.5 | 0.182 |
| 0.35 | 0.500 |
| 0.2 | 0.818 |

A villager eats when hungry and not before, which is what that curve is for. A mean of 0.056
is the shape that curve produces, not a leak. Changing it would be tuning a correct logistic
to flatter a summary statistic.

## Verdict

One genuine defect (`satisfies` missing on the two social activities), one scenario
work-around for an escalated clock decision, and two needs confirmed as behaving correctly
while looking alarming in aggregate.

The corpus was not searched this iteration: no model was being changed, and the two findings
are about wiring rather than about a paper's formula.

# What should a village's evening be?

## The problem, measured

`fun` is the one need that starves. After the walk fix made every other need recover, in a 24-person
village over 4 200 ticks:

| need | at zero |
|---|---|
| social | 2.0% |
| rest | 0.8% |
| food | 12.8% |
| **fun** | **35.3%** |

`fun` decays 0.8/day. Its only source is `relax`, which pays `0.4 per game-hour × 1.5 h = 0.6` per use,
so a person needs **1.33** uses a day. Measured supply: **0.10**.

Raising the rate was tried: `satisfies 0.4 → 0.55` gives 0.825 per use and 0.97 uses a day.

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **18.8%** |
| `fun` mean | 0.061 | **0.051** |
| mean of all four needs | **0.127** | **0.118** |

**Reverted.** The zero fraction halves and the total gets worse: `relax` now wins the evening against
`chat`, `gossip` and `eat`.

## Why it is a decision and not a bug fix

The village is over-subscribed — 24 people, four places, one six-hour window — so *any* reallocation of
the evening helps one need by taking from another. Which one should win is a design choice about what a
society's day is for, and the crate has no opinion it can derive from a paper.

## Options

### 1. Give `fun` a second source that is cheap to reach

A `Person`-targeted leisure activity — telling stories, playing — so a use costs three ticks like a
`chat` rather than a walk plus sixteen.

- **For:** fixes the *cost* rather than the rate, which is the real difference between `fun` and
  `social`. `social` needs 2.78 uses a day and is served because each use is cheap.
- **Against:** a definitions-file addition, so every host that ships its own leisure activity has a
  duplicate; and it changes what a village evening looks like.

### 2. Let `relax` satisfy `fun` while travelling

- **For:** no new activity.
- **Against:** `satisfies` pays on arrival by design — an activity that pays while walking makes travel
  free, which is the defect the previous iteration fixed in the other direction.

### 3. Accept it, and let the warning say so

- **For:** a society with one chronically unmet need is a real kind of society, and SDT distinguishes
  frustration from low satisfaction.
- **Against:** 35% of ticks at zero reads as broken, not as characterisation.

## Recommendation

**Option 1**, in a later iteration, with the new activity's cost checked against `chat`'s: the test is
`decay / (satisfies · duration)` and the activity should reach one use a day at `Person`-target pricing.

Deferred because it is a definitions-file addition a host would have to know about, and because the
number it should carry is the design question this file exists to ask.

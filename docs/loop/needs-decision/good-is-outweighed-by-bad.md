# Good is outweighed by bad, and the paper says it should prevail by numbers

> **Measured 2026-12-28. Three levers tested, all rejected. The diagnosis is the deliverable.**

## The state

`village-evening`, 24 people, 17 game-days: **18 friend pairs against 42 rival pairs** — 6.5% of all 276 pairs
friendly, 15% hostile. Rivalries do reconcile (a quarter end), but the village settles hostile.

## What the cited literature actually says

The crate already cites **Baumeister, Bratslavsky, Finkenauer & Vohs 2001** (DOI `10.1037/1089-2680.5.4.323`),
*Bad Is Stronger Than Good*, in `snub`'s consideration. The review establishes two things, and the crate has
implemented the first while contradicting the second:

1. "bad impressions and bad stereotypes are quicker to form and **more resistant to disconfirmation** than good
   ones" — bad beats good **per event**;
2. "**good may prevail over bad by superior force of numbers**: Many good events can overcome the psychological
   effects of a single bad one."

So a believable village should have **more** friendly ties than hostile ones, each of them individually more
fragile. The crate has the opposite: fewer friendly ties *and* symmetric decay.

## Why bad wins here, quantified

The asymmetry is not in the decay rates — it is in the **event weights**, and it is large.

| event | affinity effect | reach |
|---|---|---|
| `compliment` | **+0.10** | one person |
| `insult` (a deed, harm `-0.4`) | **-0.08** per witness (`witness_gain 0.2 x -0.4`) | **every witness** |

A village has roughly six people within perception. One insult therefore moves about **-0.48 of total affinity**
across the crowd, against **+0.10** for a compliment — bad outweighs good about **5:1 per event**.

And the *numbers* do not compensate: `compliment` occupies **11.3%** of person-ticks against `snub`'s **3.8%**, a
ratio of about **3:1**. Good is outnumbered 3:1 while being outweighed 5:1 per event, so friendship cannot
accumulate. That is the whole mechanism.

## The three levers, all measured, all rejected

### 1. Asymmetric decay (negative affinity fades half as fast)

The obvious reading — grudges persist — makes the village **worse**:

| | rivals | friends | ratio |
|---|---|---|---|
| symmetric (shipped) | 42 | 18 | 1.45 |
| grudge fades 2x slower | **82** | 11 | 1.85 |

With structural balance disabled it is still 64, so balance is not the driver: persisting grudges simply persist,
and friendship decays at the same rate as before, so nothing offsets them. **The paper's asymmetry applied
without the paper's numbers is a regression.**

### 2. Higher `affinity_relax_per_day` (measured last iteration)

0.01 -> 0.03 takes rivals 42 -> 10 **and friends 18 -> 5**. Relaxation cannot tell a grudge from a fondness.

### 3. Stronger `compliment` (+0.10 -> +0.20)

Inconsistent: seed 7 gives 31 rivals / 14 friends, seed 42 gives 44 / 7. Doubling one delta perturbs the
deterministic trajectory more than it changes the dynamics, so the result is not a trend.

## Options

### 1. Leave it, and document the village as tense.

- **For:** the mechanism is coherent, it saturates, and a closed village competing for one well being hostile is
  defensible.
- **Against:** it contradicts the paper the crate cites, and 2.3 rivals per friend is a strong flavour a host will
  not expect.

### 2. Cut `witness_gain` so that bad stops winning per event.

- **For:** one number, and it targets the actual asymmetry — a deed's cost is currently multiplied by the crowd.
- **Against:** `witness_gain` also drives reputation and rumor formation; lowering it weakens two other models.
  Needs a three-axis measurement before it can be chosen.

### 3. Raise the **rate** of friendly interaction, not its weight.

- **For:** matches the paper's own account of how good prevails — "superior force of numbers" — rather than
  making each compliment stronger. A second affinity-positive activity, or a shorter `compliment` duration, would
  raise the 3:1 ratio toward the 5:1 the weights demand.
- **Against:** new authored content, and it changes the activity mix a host may have tuned.

## Recommendation

**Option 3, measured on all three axes (ties, needs, interruptions) before shipping.** It is the only option that
follows the paper's own mechanism rather than fighting it, and it does not weaken reputation or rumor formation
the way option 2 would. Option 1 is honest but leaves a documented contradiction.

**Do not repeat:** asymmetric decay. It was measured and it doubles rivalries.

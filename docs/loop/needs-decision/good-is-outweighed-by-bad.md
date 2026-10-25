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

The asymmetry is not in the decay rates — it is in the **event count**, and the totals are measured. On
`village-evening`, seed 7, 24 people over 17 game-days, summing every affinity write:

| event | affinity effect | events | total |
|---|---|---|---|
| accepted `compliment` | **+0.10** each | **1 604** | **+160.4** |
| `insult` witnessed (a deed, harm `-0.4`) | **-0.0305** mean per witness | **7 920** | **-241.2** |

Bad is ahead **1.5:1** in total, and good is outnumbered **4.9:1** by events: one compliment weighs about 3.3x a
witness's reaction, but there are 4.9x as many reactions. Good loses on numbers, not on weight, and friendship
cannot accumulate. That is the whole mechanism.

An earlier version of this section claimed `-0.08` per witness (`witness_gain 0.2 x -0.4`), `-0.48` in total and
"5:1 per event". `witness_gain` moves **trust**; affinity moves by `0.15 x approval`, which is where `-0.0305`
comes from. The ledger reproduces with `cargo run --release --example evidence -- --scenario village-evening
--seed 7 --out <dir>` (`accepted_compliments` in `summary.json`).

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

- **Withdrawn.** This option rested on the arithmetic corrected above: `witness_gain` moves **trust**, not
  affinity, so cutting it would not touch the 1.5:1 affinity gap. It would weaken reputation and rumor formation
  for nothing. The affinity lever is `0.15 * approval` in `src/reputation.rs`, which has not been measured as a
  lever.

### 3. Raise the **rate** of friendly interaction, not its weight.

- **For:** matches the paper's own account of how good prevails — "superior force of numbers" — rather than
  making each compliment stronger. A second affinity-positive activity, or a shorter `compliment` duration, would
  raise the compliment count; it takes **1.5x** as many (about 2 400 instead of 1 604) to level the totals.
- **Against:** new authored content, and it changes the activity mix a host may have tuned.

## Recommendation

**Option 3, measured on all three axes (ties, needs, interruptions) before shipping.** It is the only option that
follows the paper's own mechanism rather than fighting it, and it does not weaken reputation or rumor formation
the way option 2 would. Option 1 is honest but leaves a documented contradiction.

**Do not repeat:** asymmetric decay. It was measured and it doubles rivalries.

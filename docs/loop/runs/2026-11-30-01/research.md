# Research: a village where nobody is ever pleased

## The measurement

`village-evening`, seed 7, 24 people, 17 game-days:

| | value |
|---|---|
| mean pleasure | **-0.150** |
| **max pleasure over every person and tick** | **+0.000** |
| stdev of pleasure | 0.031 |

Nobody is ever pleased, and mood barely moves. `evidence/mood.png` left panel: pleasure begins at exactly
zero, falls, and oscillates around -0.15 for 240 game-days without once returning above the axis.

## Why it cannot rise

`Mood::step` (`src/mind.rs`):

```rust
let pull = (rates.pull_per_day * dt) * intensity;
self.pleasure += (centre[0] - self.pleasure) * pull;
let back = (rates.return_per_day * dt).min(1.0);
self.pleasure += (baseline.pleasure - self.pleasure) * back;
```

`pull_per_day = 4.0`, `return_per_day = 1.0`. The personality baseline for a sampled personality is near
zero — `Personality::sample` takes a mean of three draws, so traits centre on 0.5, and `pad::baseline` is
linear in traits with zero at 0.5. So pleasure is pulled toward a **negative** emotion centre at 4:1 and
returns to **zero** at 1:1. Its ceiling is the baseline, which is zero.

The emotion centre is negative nearly always. Measured over 20 000 ticks with 24 people:

| appraisals written | count |
|---|---|
| Distress | **1 302** |
| Joy | 427 |
| Anger | 339 |

and at the end of the run the **live** entries were 15 `Anger`, 4 `Distress`, and **0 `Joy`** — joy arrives
less often *and* is crowded out, so the steady state has negative affect and no positive affect at all.

## Where the asymmetry lives

- `patient_reaction` gives `Gratitude` for a good deed and `Anger` for a bad one, both at `|harm|`: symmetric.
- `witness_reaction` gives `Admiration` for approval and `Reproach` for disapproval, both at `|approval|`:
  symmetric.
- The **interactions** are not. Until this iteration the starter pack had **no `on_reject` on any
  interaction**, so a refusal's only emotional consequence was the crate's literal `0.3` distress, and an
  acceptance paid nothing emotional at all. Half of all interactions in this village are refused (measured
  51 accepted, 56 refused).

So the 3:1 is a **definitions-file** property, not a model property — which is why the fix belongs in the
pack, and why it is a decision rather than a bug fix.

## Sources

- **Mehrabian 1996**, DOI `10.1080/00049539608259510` — the personality-to-PAD baseline, already cited at
  `src/model/pad.rs:10`. It is **centred on zero**: a neutral personality has neutral mood.
- **Gebhard 2005** (ALMA), DOI `10.1145/1082473.1082478` — the mood update's pull-and-return form, already
  cited at `src/mind.rs:140`.
- **Ortony, Clore & Collins** via **Gratch & Marsella 2004**, DOI `10.1016/j.cogsys.2004.02.002` — the OCC
  appraisal set: a refusal is *distress*, not an injury.

No new paper was needed. The finding is that two cited models compose badly: ALMA's 4:1 pull assumes
emotions short-lived relative to the baseline's pull, and Mehrabian's baseline assumes a neutral personality
is neutral *in mood*, so a pack whose appraisals skew negative has nowhere to go but down.

## What believable output looks like

A village where pleasure is **sometimes positive**: a person whose needs are met, whose greetings are
accepted, and who has just been complimented should be above its baseline, and the pack should say so.

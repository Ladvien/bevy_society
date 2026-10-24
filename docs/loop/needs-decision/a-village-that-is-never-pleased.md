# Should a village ever be pleased?

## The problem, measured

`village-evening`, seed 7, 24 people, 17 game-days:

| | value |
|---|---|
| mean pleasure | **-0.150** |
| **max pleasure, over every person and every tick** | **+0.000** |
| stdev of pleasure | 0.031 |
| mean arousal | +0.030 |
| mean dominance | +0.069 |

Nobody is ever pleased. Pleasure reaches exactly zero and stops, as though it were clamped — and the
standard deviation is a third of the mean, so mood barely moves at all.

## Why

`Mood::step` does two things per tick:

```rust
let pull = (rates.pull_per_day * dt) * intensity;          // toward the emotion centre
self.pleasure += (centre[0] - self.pleasure) * pull;
let back = (rates.return_per_day * dt).min(1.0);            // toward the personality baseline
self.pleasure += (baseline.pleasure - self.pleasure) * back;
```

`pull_per_day = 4.0` against `return_per_day = 1.0`, so the emotion centre dominates by 4:1. And in this
village the centre is negative nearly always. Measured over 20 000 ticks with 24 people:

| appraisals written | count |
|---|---|
| Distress | **1 302** |
| Joy | 427 |
| Anger | 339 |

and at the end of the run the **live** entries were 15 `Anger` and 4 `Distress`, with **zero** `Joy`. The
personality baseline for a sampled personality is near zero (traits centre on 0.5, and `baseline` is
linear in them), so pleasure is pulled down by a factor of four and returns to zero by one. It can only
ever reach the baseline, never exceed it.

## Why it is a decision and not a bug fix

Three defensible answers, and they disagree about what the model *is*:

### 1. The appraisals are unbalanced, and that is the bug

Distress 1 302 against Joy 427 is 3:1. Where the pack has no `on_reject` emotion, a refusal costs distress
and pays nothing; a *witnessed* deed gives `Admiration` for approval and `Reproach` for disapproval
symmetrically, so the asymmetry is in the interactions. The fix is in the **definitions file**: give each
refusal a modest, definition-level emotion and each acceptance a matching one, so the pack's own ratio is
near 1:1. That is data, and a host's pack may want a cruel village.

### 2. The mood baseline is wrong for a neutral personality

`pad::baseline` is Mehrabian's, and it is *centred on zero* — a neutral personality has neutral mood. But a
person whose needs are met and whose ties are friendly is not at neutral mood; the model has no term for "my
life is going well". Adding one (a slow drift toward the mean of need satisfaction, say) would give pleasure
somewhere to go, and it is a model change with a paper to find.

### 3. `pull` and `return` are the wrong ratio

4:1 toward the emotion centre is ALMA's mood update, but ALMA's emotions are shorter-lived than this
crate's. With a 0.1-day half-life they are not, so the ratio may simply be too aggressive here. One number,
no new concept.

## Recommendation

**Option 1 first.** It is data, it is where the 3:1 lives, and it does not change the model for every host.
Measured while investigating: removing the refusal distress entirely moves mean pleasure from **-0.202 to
-0.105** in a tight eight-person probe — so refusals are about half of the negativity *there*, but barely
register in the full village (-0.150 to -0.154), which means the village's own 3:1 has another source and
option 1 needs the full-village numbers before it is tuned.

## What ships now

`Tuning::rejection_distress` names the refusal constant, which was a literal `0.3` and the crate's only
unnamed emotion intensity. A definition's own `on_reject` emotion now **replaces** it rather than stacking
with it, which measured *better* than either alone.

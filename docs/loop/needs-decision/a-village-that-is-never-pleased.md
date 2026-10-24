# Should a village ever be pleased?

> **Largely resolved, in two steps and not by the options below.** 2026-12-01 added the **positivity offset**
> (option 2's "a term for life going well", with a source: Cacioppo et al. 2013's positive *intercept*), which
> fixed the **ceiling** — max pleasure went from exactly `+0.000` to `+0.0155`. 2026-12-02 gave `chat` and
> `gossip` a mutual `Liking` on accept (option 1, the appraisal balance), which fixed most of the **mean**:
> `-0.122 -> -0.066` at three seeds, and the steady state went from **Joy 0 / Liking 0** to **Joy 22 /
> Liking 14**. Mean pleasure is still mildly negative and that is now a deliberate negativity bias, not a
> defect. The original analysis follows, because it is what made both steps findable.

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

## What actually fixed it, 2026-12-02

Option 1, but not by tuning a rate. Measuring where the 3:1 came from gave a structural answer:

| | count |
|---|---|
| refusals | 1 216 |
| distress appraisals | **1 216** |
| acceptances | 1 338 |
| positive appraisals | **443** |

**Every refusal is exactly one appraisal and no acceptance is.** `chat` and `gossip` — 43% of accepted
interactions — paid no emotion to either party, and `compliment` paid only its target. Giving both a mutual
`Liking` moved the ratio to **1.06:1** and mean pleasure from **-0.122 to -0.066** at all three seeds.

**Why the mean stays negative, deliberately.** Cacioppo et al. give the negative channel the steeper *gain*;
an insult pays `Anger 0.5` against a conversation's `Liking 0.2`, refusals are more frequent than acceptances
in a crowd, and so distress outnumbers joy even now. That is the negativity bias the paper describes, and the
remaining headroom is a question about the pack, not the model.

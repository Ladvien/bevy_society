# Review

## Pass 1 — Behavior: does the change do what the literature and the scenario say?

**GREAT.** Cacioppo et al. 2013 give the negative channel the steeper *gain* and the positive channel the
higher *intercept*. Neither is silence, and the pack had silence: 43% of accepted interactions paid no emotion
at all, and the actor of a pleasant exchange gained no affect from it in any interaction.

| claim | measured |
|---|---|
| every refusal is one appraisal | `Distress` **1 216** against **1 216** refusals — exact, `assert_eq!` |
| acceptances were not | **443** positive appraisals from **1 338** acceptances |
| after | **1 491** positive appraisals; ratio **3.58:1 → 1.06:1** |
| mood | **−0.122 → −0.066** at seed 7, and **−0.127 → −0.065** / **−0.110 → −0.070** at 42 and 99 |

## Pass 2 — Zoom: read the plot closely

**GREAT.** `evidence/appraisal-balance.png`:

- **Left:** six traces, three seeds × before/after. Every green (after) trace sits above its red counterpart
  for the whole 240 game-days, and all of them still oscillate below the zero axis — the band moved up, it did
  not invert. No clipping, no drift, no trace pinning at ±1.
- **Middle:** the mean halves at every seed, so this is not one lucky run.
- **Right:** the steady-state emotion bars. **Joy 22 and Liking 14** where both were **zero** before. That
  zero bar was the defect; it is now populated, with Anger unchanged at 14 — the bias survives.

## Pass 3 — Fresh eyes: three seeds, and a skeptical host

**GREAT.** Three seeds rather than one, because the previous iteration's number could have been seed luck; the
effect is consistent to within 0.005 across all three.

Two questions a host would ask, both answered:

1. **Does this erase the negativity bias?** No — `insult` still pays `Anger 0.5` against `chat`'s
   `Liking 0.2`, and Anger is unchanged at 14 people at the end. The ratio moved from 3.6:1 to 1.06:1 by the
   *positive* channel firing, not by the negative one weakening.
2. **Is the mean fixed?** No, and the eval says so. Mean pleasure is −0.066, still negative, because each
   refusal is exactly one distress and refusals are frequent.

The narrative to a host:

> A pleasant conversation now registers emotionally for both people, not just the one being complimented.
> Before, `chat` and `gossip` — the two most common interactions, 43% of accepted exchanges — gave a need bump
> and no feeling at all, while every insult and every refusal left a mark. Your villagers' mood now sits near
> neutral rather than pinned negative, and the negativity bias is intact: an insult still hits 2.5× harder
> than a pleasant word.

## Verdict

Three GREAT. Shipped.

# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does `fun` stop starving, without robbing the others?

Claim: a `Person`-targeted leisure activity lets `fun` be met at a cost a person can pay, so it stops
sitting at zero — and unlike a rate change, it does not simply move the shortage elsewhere.

**Evidence.** `tests/leisure.rs` (written **red first** at `best source for fun is relax, 0.600/use ->
1.33 uses/day`) and the same-seed run:

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **1.3%** |
| `fun` mean | 0.061 | **0.112** |
| mean of all four needs | 0.127 | **0.128** |

**Critique.** The three other needs each lose a little — `social` 0.124 → 0.107, `food` 0.101 → 0.080 —
because `play` competes for the same hours. The total is flat, which is the honest summary: this is not
a free win, it is a reallocation that happens to hit the worst-served need. That is exactly the test last
iteration's rate change *failed*, and the reason this one is different is that the new source does not
consume the evening.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the plot

**Critique.** `evidence/play.png` — and this time I checked it against the numbers instead of trusting
it, which is what last iteration taught. The `fun` panel shows green holding a band above red; the mean
panel shows green above red throughout. Both agree with the table.

The thing to look for and *not* find: no flatline at zero, no saturation at 1.0, no oscillation between
the two needs that would suggest they are stealing from each other tick by tick. `fun` rises and decays
at a period of roughly a game-day, which is what a daily leisure activity should look like.

**Critique of the cost.** `fun`'s mean doubles but `food`'s falls 20%. If `food` were the worse-served
need this would be the wrong trade. It is not — `food` is at zero 15% of ticks against `fun`'s 34% — but
the margin is not enormous, and a village with more people would make it worse. Recorded.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: would a host understand the new activity?

**Critique.** `play` reads in the definitions file with a comment explaining the two decisions that
matter: why `Person` and not `Place`, and where the rate comes from. The narrative shows it as a real
event (`d17 21h33: started play`, `d17 22h07: finished play (completed)`).

**Critique — the honest cost.** The 5 000-person tick cost rises **9.2%**, measured three times. That is
the price of a `Person`-targeted activity and it is stated in `perf.md` with the arithmetic
(`10/9 − 1 = 11.1%` predicted against 9.2% observed) rather than waved at as noise. A host that prefers
the cheaper tick can drop `play` and keep `relax`.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — `fun` at zero 33.8% → 1.3%, mean doubled, total flat |
| Zoom | GREAT — plot and numbers agree; no flatline, no saturation, no tick-by-tick oscillation |
| Fresh eyes | GREAT — the activity explains itself, and its 9.2% cost is measured and stated |

## Top 5 remaining

1. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
   `needs-decision/retiring-untellable-rumors.md`.
2. **A spatial index for perception** — every `Person`-targeted activity scores once per neighbour, and
   this iteration's 9.2% is that cost. It was top-5 in earlier runs and now has a price attached.
3. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`food` at 15% of ticks at zero** — the worst-served need now that `fun` is fixed.

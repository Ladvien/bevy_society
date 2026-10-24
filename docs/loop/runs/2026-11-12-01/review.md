# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the crate now say what it cannot fix?

Claim: a need whose sources cannot meet its decay is named at startup, and the arithmetic is available to
a host.

**Evidence.** `warn_unbalanced_needs` in `finish`, and two accessors, exercised by `tests/need_budget.rs`:

```
social  decay 1.00/day, best source chat   pays 0.36/use -> 2.78 uses/day
rest    decay 0.70/day, best source sleep  pays 1.20/use -> 0.58 uses/day
food    decay 0.90/day, best source eat    pays 0.90/use -> 1.00 uses/day
fun     decay 0.80/day, best source relax  pays 0.60/use -> 1.33 uses/day
```

**Critique — the honest weakness.** The starter pack does **not** trip the warning, because the
threshold is four uses a day and `fun` is at 1.33. So the warning does not fire on the very case that
motivated it. That is deliberate — the crate cannot see that `relax` costs a walk and a window while
`chat` costs three ticks — but a reader is entitled to ask what the warning is *for*. It is for a
definition where a need's source supplies under a quarter of its decay, which is a definition that can
never work. `fun`'s case is a definition that *could* work if its source were cheaper, which is the
decision file, not a warning.

**Verdict: GREAT** for what it does, with that limit stated in the code and in the eval.

## Pass 2 — Zoom: reading the plot

**Critique.** `evidence/relax.png` is the plot of the fix I did **not** ship. It shows green above red:
`fun`'s peaks rise from ~0.10 to ~0.15 and the mean of all four needs sits visibly higher after tick
800. A reader glancing at it would conclude the change worked.

The numbers say otherwise: `fun`'s mean falls 0.061 → 0.051 and the total goes 0.127 → **0.118**. That is
the single most useful thing this iteration learned — **a plot of one need can look like an improvement
while the society gets worse.** `eval.md` prints both.

**Critique.** I nearly shipped it. The zero fraction halved, which is the metric the log's #1 named, and
the plot agreed. What stopped it was computing the mean of all four needs, which is not a metric the
loop had been requiring.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: is the record honest?

**Critique.** A stranger reading this run's directory sees: the measurement that motivated it, three
hypotheses tested and rejected with numbers, the fix, the reason it was reverted, and a decision file
asking the design question the fix was trying to answer. The negative result is kept rather than
deleted, which is the point — the next iteration should not spend a round rediscovering that raising
`relax` does not work.

**Critique.** `research.md` says plainly that the real fix is not a rate. That is a smaller deliverable
than a working `fun` cycle, and calling it a win would be dishonest. It is a diagnostic and a recorded
negative result, and the review says so.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — the arithmetic is named and testable; the threshold does not fire on `fun`, and that is documented |
| Zoom | GREAT — the plot of the reverted fix looks like an improvement while the total got worse |
| Fresh eyes | GREAT — the negative result is recorded with numbers and a decision file |

## Top 5 remaining

1. **A cheap source for `fun`** — `needs-decision/over-subscribed-evening.md` recommends a
   `Person`-targeted leisure activity, so a use costs a `chat`'s three ticks rather than a walk and a
   window.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
3. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`work` at 72%** — the lowest completion of the three fixed activities.

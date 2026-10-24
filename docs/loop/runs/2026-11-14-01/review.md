# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: is the corrected claim right?

Claim: the rumor book grows, its growth costs memory and not ticks, and most of it is untellable.

**Evidence.** `tests/rumor_book.rs`, both tests run:

```
2250 rumors, 2250 alive, 301 still tellable
3000 ticks: 2250 (~105 KiB); 6000: 4865 (~228 KiB)
```

and the per-tick table, sampled five times in a 6 000-tick run, with and without a `RumorReach` trigger.

**Critique.** The test asserts `tellable * 5 < total` — 80% untellable — where the measurement is 87%.
I did not pin 87% exactly, because the figure depends on population and run length and this iteration
exists precisely because a number like that was carried too long without re-measuring. The assertion is
the property; the percentage is a printed observation.

**Verdict: GREAT.**

## Pass 2 — Zoom: what the table actually shows

**Critique.** The ms/tick column *falls* from 5.76 to 2.97 before rising to 4.80 at tick 5 000. That rise
is worth a second look: is it the book finally biting?

It is not — the same rise appears in the no-trigger column, and the run without a trigger has an
identical book. What varies across samples is how many people are mid-decision, which depends on how
many activities happen to end together. The honest statement is "the cost does not grow with the book",
not "the cost is flat", and `perf.md` says the former.

**Critique of the method.** Five samples of 100 ticks is enough to see a 4× effect and not enough to
resolve a 20% one. Since the claim being tested was 4×, that is proportionate; a smaller claim would need
more samples, and the file says so rather than presenting the numbers as precise.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: does a host reading `RumorBook` learn the truth?

**Critique.** The decision file now opens with the correction, keeps the original claim for the record,
and states the urgency change plainly: a 4× blowup justifies a breaking semantic change, 5 MiB does not.
A host reading it can decide for themselves with the measurement in front of them, which is what a
decision file is for.

**Critique — what this iteration does not do.** It ships no behaviour change. On a loop whose goal is
believable societies, correcting a number is a smaller deliverable than fixing one. But eight iterations
of top-5 lists have been steering by it, twice as the #1 item, and the fix it justified is a breaking
change to a host-visible field. Getting that wrong would have been expensive; the correction is what
makes the *next* attempt at it honest.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — book grows, all alive, 87% untellable, ~48 B/rumor, pinned by two tests |
| Zoom | GREAT — the late rise is decision timing, not the book; stated as "does not grow", not "flat" |
| Fresh eyes | GREAT — the decision file now leads with the correction and the change in urgency |

## Top 5 remaining

1. **Retire untellable rumors** — still the right fix for memory, now with a true cost and a decision
   file a host can act on: 5 MiB, not 4×.
2. **A spatial index for perception** — the measured 9.2% from iteration 2026-11-13-01.
3. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`food` at 15% of ticks at zero** — the worst-served need now that `fun` is fixed.

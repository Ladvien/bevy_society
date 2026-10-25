# Research: a number without a spread is not a measurement

## Why this

Item 2 in my log for **nine iterations**: "the guide's scale curve is unverified; the machine has never been idle
enough." Last iteration I wrote the honest version of that note — *stop waiting for a quiet machine; re-measure
with median-of-N and a reported spread* — and this iteration did it.

## The defect was in the tool, not the machine

`examples/benchmark` timed **one window** and printed a mean. On a shared machine that statistic describes the
load, not the code. Measured on one binary, one afternoon:

| persons | fastest window | median of 9 windows | spread |
|---|---|---|---|
| 100 | **0.74 ms** | 1.08 | 103% |
| 1 000 | **28.4 ms** | 60.4 | **100%** |
| 2 000 | **100.6 ms** | 140.1 | 49% |

A **100% spread at 1 000 people**. No amount of waiting for an idle machine fixes that; the figure simply cannot
be derived from one window.

## What is reproducible

The **minimum** is. Four separate invocations at 1 000 people gave 28.29, 28.38, 28.38, 28.48 ms — a spread of
**0.7%** — while the corresponding medians were 57.5, 60.4, 71.7, 57.3. At 2 000: 100.55 and 100.93, **0.4%**.

That is the methodological answer: **report the fastest window**, because a faster window is evidence the machine
was momentarily quiet, and a slow one is evidence it was not. The minimum is a lower bound on what the code costs.
The median is a lower bound on what the machine costs, which is a different question.

## The numbers that were wrong

`docs/guide/concepts.md` said **13.5 ms** at 1 000 and **245.1 ms** at 5 000. The fastest window measurable here at
1 000 is **28.4 ms** — more than twice the claimed figure — and the 5 000 case no longer completes inside a
sensible deadline on this machine at all.

Those numbers survived nine iterations because:
1. the tool printed one mean, so a noisy reading looked like a measurement;
2. each iteration noted the doubt instead of replacing the number;
3. nothing asserted the tool's output shape, so dropping `spread_pct` would have been silent.

All three are fixed.

## Sources

None — this is measurement methodology, not a model. But the principle is standard: a benchmark whose variance
exceeds its effect is not reporting anything, and the fix is to report the variance rather than to average harder.

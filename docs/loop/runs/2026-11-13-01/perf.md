# Perf

One new activity definition takes the starter pack from nine to ten, and the new one targets `Person`,
so it scores once per neighbour — the same shape as `chat` and `gossip`.

| Persons | ms per tick, before | after | delta |
|---|---|---|---|
| 100 | 0.564 | **0.573** | +1.6% |
| 1 000 | 15.02 | **15.67** | +4.3% |
| 5 000 | 275.5 | **300.8** | **+9.2%** |

## The 5 000 figure is real, and it is the price of the fix

Measured three times: 300.786, 300.709, 300.906. Not noise.

Scoring is `O(activities × targets)`. Going from nine activities to ten predicts
`10/9 − 1 = +11.1%` if scoring dominates; the observed **+9.2%** is that, minus the share of the tick
that is perception, gossip and the rest. So the regression is fully explained by the extra
`Person`-targeted activity, and it is the cost of meeting `fun` with something cheap to *reach* rather
than cheap to *score*.

Worth stating plainly rather than buried: **a `Person`-targeted activity is cheap in ticks-per-use and
expensive in score-per-neighbour.** At 5 000 people that is 25 ms a tick. A host that cares more about
the second number than the first can drop `play` from its own definitions file, and `relax` remains.

## What would make it cheaper

Scoring every activity against every neighbour is the same cost that made the candidate-per-person bug
in iteration 2026-11-05-01 worth fixing. A spatial index for perception is the recorded item behind it
(`docs/loop/log.md`, top-5 #5 in earlier runs) and would reduce every `Person` activity's per-neighbour
work at once, not just this one.

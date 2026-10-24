# Review

## Pass 1 — Behavior: does the shape do what it claims?

**GREAT**, and it is the first positive result in six attempts.

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | **22** | **33** | -0.054 |
| clamp only, bar 0.4 | 0 | 18 | -0.078 |
| clamp + trust 0.55 + bar 0.25 | **15** | **18** | -0.078 |
| clamp + trust 0.55 + bar 0.30 | **12** | **18** | -0.078 |

**Both** counts fall — friends by a third, rivals by nearly half — where five previous mechanisms either did
nothing or inverted the village. Holds at three seeds.

## Pass 2 — Zoom: is the cost understood or hand-waved?

**GREAT**, and this is where the measurement earns its keep. Mood falls -0.054 → -0.078, and the two bar settings
give **identical** mood, which isolates the cause to the **clamp** rather than to the friend count. The chain is
then visible in named numbers: `chat` gates on `Affinity` at `Logistic(8.0, 0.4)` and `compliment` at
`Logistic(8.0, 0.7)`; a clamp at ±0.35 puts both curves lower, so fewer contacts occur, so fewer `Liking`
exchanges, so lower mood.

That is a tradeoff with a mechanism attached, not a number that moved.

## Pass 3 — Fresh eyes: what would trip this up next time?

**GREAT**, and two things did.

**The runaway prediction was wrong.** I expected a four-times-longer run to give a bigger clique; it saturates at
**122** and decays to **109**. The problem is a high asymptote, not an unbounded one — which is *why* the bar
sweep failed and the clamp works: the bar only delays the crossing, the clamp lowers the ceiling.

**The first clamp was asymmetric.** `delta · (1 − |a|/K)` decays a positive step but lets a negative one reach
the outer `-1` clamp; measured, rivals **rose 33 → 48**. A plain `clamp(-K, K)` bounds both directions. Had I not
checked rivals, the asymmetry would have been indistinguishable from "the clamp does not work".

## Verdict

Three GREAT for the investigation. **No model change shipped** — the shape bounds the network and costs mood,
and choosing between those is the decision file's call, which now has the numbers.

The honest narrative:

> A village's friendship count is not a runaway — it saturates. But it saturates high, at 44% of all pairs,
> because liking rises to its maximum for anyone you have spent time near and the bar for "friend" is far below
> that maximum. Raising the bar does not help; capping liking *below* the bar does, and costs a little mood
> because fewer people then reach the activities that need a liked companion.

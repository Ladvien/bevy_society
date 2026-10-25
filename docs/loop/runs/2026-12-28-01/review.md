# Review

## Pass 1 — Behavior: did any lever do what it claimed?

**FAIL on all three.** Asymmetric decay made the village worse (42 → 82 rivals); a stronger compliment was
inconsistent across seeds (31/14 against 44/7); a higher relax rate was already rejected last iteration for
emptying friendships.

## Pass 2 — Zoom: the regression is the finding

**GREAT as measurement, FAIL as a fix.** The asymmetric-decay result was my prediction and it came out backwards.
Isolating it — `balance_rate_per_day = 0` still gives 64 rivals — shows structural balance explains only 18 of the
40, so the direct effect is simply that persisting grudges persist while friendship keeps fading at the same rate.

Reading the paper again rather than my summary of it produced the real answer: Baumeister et al. separate
**per-event weight** (bad wins) from **aggregate outcome** (good wins by count). The crate implements the first and
violates the second, and the violation is arithmetic:

- one `insult` = **-0.48** total affinity across ~6 witnesses;
- one `compliment` = **+0.10** to one person;
- ratio of events = **3:1**, ratio of weights = **5:1**.

That is why the village is hostile, and none of it is a decay rate.

## Pass 3 — Fresh eyes: is "nothing shipped" defensible?

**GREAT, and it is the honest answer.** The fix that follows the paper — raise the *rate* of friendly interaction,
not its weight — is new authored content that changes the activity mix a host may have tuned. It needs a
three-axis measurement (ties, needs, interruptions) before it can be chosen, and that is a decision with options,
not a unilateral change to a starter pack.

What a host gets instead: a citation that says the starter pack contradicts its own literature, and the three
numbers that show by how much.

## Verdict

Pass 1 **FAIL**. Pass 2 **GREAT** (measurement), FAIL (fix). Pass 3 **GREAT**.

## Not shipped, deliberately

- The asymmetric decay constant. Measured and rejected; recorded so it is not retried.
- A stronger `compliment`. Inconsistent across seeds.
- Option 3 (more friendly events). The right shape, but authored content plus a three-axis measurement.

## A note on the prediction

I predicted the asymmetry would help and it doubled rivalries. The value of the iteration is that the number
exists now, and the paper was re-read rather than trusted through my own summary — which is where the real
mechanism turned up. That is the third iteration running where re-reading the primary source beat the plan.

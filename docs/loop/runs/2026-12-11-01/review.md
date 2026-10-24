# Review

## Pass 1 — Behavior: did the threshold hypothesis hold?

**FAIL**, twice, and the failures are the finding.

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| trust gate 0.6 | 22 | 33 |
| contact talk + trust gate 0.35 | **267** | **0** |
| saturate all three | 23 | 43 |
| saturate all three + contact talk | **268** | **0** |

The threshold is not the lever either. Neither is trust, and neither is saturation of any set of variables.

## Pass 2 — Zoom: the distributions explain both failures

| | affinity | trust | familiarity |
|---|---|---|---|
| friend-ties | **0.792** | **0.480** | 0.907 |
| other ties | −0.223 | 0.068 | 0.908 |

Two things fall out. **Familiarity does not discriminate at all** (0.907 against 0.908) — yet `classify`
demands `≥ 0.3` of it, which every pair in contact satisfies. And **trust does discriminate sevenfold**, which is
why the gate looked right — but `chat` supplies `trust: 0.05` per exchange, so the gate's quantity is saturated
by the very mechanism it is meant to bound.

## Pass 3 — Fresh eyes: does the explanation cover all five failures?

**GREAT**, and this is what makes the iteration worth more than its reverted code. Five mechanisms have failed —
a `help` activity, a contact exchange, saturating affinity, a trust gate, saturating all three — and one
statement covers every one:

> **No tie variable can sit below a threshold for a pair that has had contact.** All four accumulate, every
> exchange supplies all of them, and saturation bounds at 1.0 which is above every threshold. Any *level*-based
> gate is therefore passed by contact volume.

That is the first account that is not specific to the mechanism that produced it, and it names the one untested
shape: bound **below** the threshold.

The narrative to a host:

> Every relationship number — liking, trust, respect, familiarity — grows without limit as two people interact,
> and the bar for "friend" is a fixed value well below where they all end up. So anyone you have spent enough
> time near is your friend, whatever the time was like. Five attempts to fix that by changing a rate or adding a
> second bar all failed; the fix has to cap the numbers *below* the bar.

## Verdict

Pass 1 **FAIL**. Pass 2 **GREAT** (the distributions explain both negatives). Pass 3 **GREAT** (one statement
covers all five failures).

**Reverted.** The decision file now carries the failure table, the distributions, and the single untested shape.

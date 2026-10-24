# Review

## Pass 1 — Behavior: does the recommended fix do what it claims?

**FAIL**, and that is the result.

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| saturation alone | 22 | 38 |
| contact talk 3 h | **267** | **0** |
| **saturation + contact talk 3 h** | **260** | **0** |

The claim was that a saturating gain stops friendship scaling with contact volume. **260 against 267** is not a
bound; it is the same clique with two fewer pairs.

## Pass 2 — Zoom: where the claim breaks

Saturation alone is neutral — 22 → 22, which is why it looked plausible. The failure appears only when the
mechanism that caused the original problem is present, and that is the configuration that matters.

The arithmetic explains it: a saturating gain makes affinity approach 1 asymptotically instead of linearly, but
both curves cross `0.4`, and both do so within 45 exchanges. It changes **when**, not **whether**.

**This is the third mechanism to fail on one problem** — a `help` activity (rivals 44 → 0), a contact exchange
(friends 22 → 267), and saturation (260). Each was plausible; each was measured; none worked. That pattern is
now the most informative thing in the log.

## Pass 3 — Fresh eyes: what a host would get

Nothing, which is correct. Shipping saturation would have been a model change to every tie in every host, sold
as a fix, that leaves a 24-person village with 260 friend pairs and no disagreements.

The narrative:

> The cheapest fix for "everyone becomes friends" does not work. Making each conversation worth less than the
> last still crosses the friendship threshold — it just takes more conversations. What is needed is something
> that keeps affinity *below* the threshold for most pairs, and the untested candidate is a gate on trust,
> which the crate already models and which already moves far more slowly.

## Verdict

Pass 1 **FAIL**. Pass 2 **GREAT** for the diagnosis. Pass 3 **GREAT** for not shipping.

**Reverted.** The decision file's recommendation is retracted, with the measurement and three remaining options —
recommending the **trust gate** next, because it is the only candidate that uses a quantity contact cannot
satisfy quickly.

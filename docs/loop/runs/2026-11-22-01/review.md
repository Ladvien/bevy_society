# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does a person walk to the nearer place?

Claim: with two places serving the same activity, a person takes the nearer one.

**Evidence.** `tests/nearest.rs`, written **red first**: `6000 decisions between a 1 m tavern and a 9 m one:
near 3020, far 2979` → `near 5999, far 0`.

**Critique.** The test removes `Doing` every tick to force a fresh decision, which is not how a village
works — but it is the right way to isolate *this* decision, and the real run corroborates the direction
(62.5% → 58.9% travelling). A test that only measured the village would have shown a 3.6-point difference
and could not have distinguished "nearest works" from "noise".

**Verdict: GREAT.**

## Pass 2 — Zoom: the plot

**Critique — and this is the pass that keeps the claim honest.** `evidence/nearest.png` shows the two
travelling lines **overlapping heavily**. The need means separate by a visible but modest gap. A reader
looking only at the plot would ask whether anything changed.

That is the truthful picture: **the defect is unambiguous in isolation and modest in this layout**, because
the village's four places are within about 13 m of each other. Reporting "62.5% → 58.9%" as a fix for
travelling would overclaim; the `travel-cost-is-not-scored.md` decision is the larger lever, and its measured
12-point improvement came from a different mechanism.

**Critique of a choice worth stating.** Nearest for `Place` and uniform for `Person` is not arbitrary: a
villager that always talks to the physically closest person has one friend. Variety in whom you talk to is
the point; variety in which tavern is not.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: is the crate more honest?

**Critique.** Before this, `Signal::Distance` existed, was documented as an input, and was **not consulted**
for the one target kind where the destination cannot move. A host reading the docs would assume the decision
prefers a near place; it did not. That is now true, and the new test says so.

**Critique.** The change is small and its village effect is small. The value is that a documented signal now
does something where a reader would expect it to, and the next measurement of travelling is not polluted by
a coin flip.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 3 020/2 979 → 5 999/0; village travelling 62.5% → 58.9%, holds at a second seed |
| Zoom | GREAT — the plot shows a small effect, and the review says so rather than overclaiming |
| Fresh eyes | GREAT — a documented signal now does what its docs imply for `Place` targets |

## Top 5 remaining

1. **Travel cost is not scored** — `needs-decision/travel-cost-is-not-scored.md`. Five activities score
   `Distance` nowhere, and eight are shorter than a crossing walk. The larger lever.
2. **`Doing` while travelling** — the labelling decision; un-ignore `tests/travel.rs` when it lands.
3. **`rumor-crosses-a-cluster` is `pending`** — 0.333 against a 40% observable; a 10.7 h crossing walk.
4. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
5. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip's "23 of 25" is measured under
   settings the spec does not declare.

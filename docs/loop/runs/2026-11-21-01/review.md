# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: is the report honest now?

Claim: the harness no longer calls a walking person asleep, and the resulting number is the truth about
what a villager does with its day.

**Evidence.** `evidence/travelling.png` and the split table. `sleep` alone is **14.7%** — *below* the 33.3%
one 8-hour sleep a day allows — because most villagers never arrive at bed in the hours the `Hours(22, 6)`
gate allows. Travelling is **62.5%**.

**Critique.** The change is a measurement fix, and a measurement fix that makes the number *worse* (14.7%
sleeping, not 36.4%) is worth more than one that makes it look better. But it also means the previous
iteration's "56.8% → 36.4%" summary is now superseded: 36.4% was itself 21.8 points of walking. The log says
so rather than leaving two incompatible claims.

**Verdict: GREAT.**

## Pass 2 — Zoom: the plot

**Critique.** The stacked area chart is the clearest evidence this loop has produced: red (travelling) fills
most of every column for 17 game-days, with green (sleeping), blue (working) and orange (eating) as thin
bands. There is no period where the village is mostly *doing* anything.

**Critique of my own reading.** I nearly wrote this up as "the village is asleep half the day" one iteration
ago. The plot now shows that the red band was always there, hidden inside the `sleep` label.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: what a host sees

**Critique.** A host reading `Doing` still gets `activity: sleep, arrived_day: None`. The decision file for
that stands. But this iteration shows the *consequence* is larger than "a confusing label": 62.5% of a
village's life is travel, and the decision cannot price it because `targets_for` gives a `Place` no distance
term.

**Critique of scope.** The measured fix — `Distance` on three activities — is a 12-point improvement in
travelling for 1.3% of need. It is a real effect and I did not ship it, because it edits the file a host
ships. That is the right call under the crate's own rule, and
`needs-decision/travel-cost-is-not-scored.md` recommends the version that is not a host edit.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — `sleep` 14.7% and travelling 62.5%, both now measured rather than inferred |
| Zoom | GREAT — the clearest plot the loop has made; the red band was hidden inside the `sleep` label |
| Fresh eyes | GREAT — the labelling decision's consequence is 62.5% of a life in transit, and the cause is priced nowhere |

## Top 5 remaining

1. **Travel cost is not scored** — `needs-decision/travel-cost-is-not-scored.md`, recommending a crate-level
   consideration. Measured at 12 travelling points for 1.3% of need.
2. **`Doing` while travelling** — the labelling decision; un-ignore `tests/travel.rs` when it lands.
3. **`rumor-crosses-a-cluster` is `pending`** — 0.333 against a 40% observable; a 10.7 h crossing walk.
4. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
5. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip's "23 of 25" is measured under
   settings the spec does not declare.

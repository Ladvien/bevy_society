# Review

## Pass 1 — Behavior: does the change do what the crate's own vocabulary says?

**GREAT.** `Signal::Distance` is documented as an input for the utility decision, and it was true for a
`Person` target and false for a `Place`. The test is written to the contract, not the implementation: a
`Place` activity asking for `Distance` must be **choosable at all**, and must **prefer the near place**.

| | near | far |
|---|---|---|
| before | **0** | **0** |
| after | **5 999** | **0** |

Written red first at 0 / 0, which is the defect in one line: not "the wrong place", but **no place**.

## Pass 2 — Zoom: read the plot closely

**GREAT, and the plot is where the honest part lives.** `evidence/distance-signal.png`:

- The **red** line (`Distance` before the fix) is flat at zero for the whole run — 93% of person-ticks at
  zero. That is what "disabled" looks like, and it is unmistakable.
- The **grey** line (no `Distance` on `eat`) oscillates 0.3–0.6, a normal eat-sleep cycle.
- The **blue** line (after the fix) starts near the grey line and then **drifts down** to ~0.1 over 270 days.

That drift is not the fix failing; it is the **curve** being too harsh. A tavern 6.5 m away inside an 8 m
perception radius scores `Linear(-1, 1)` = **0.19**, so `eat` is chosen less and less as `food` falls, and
`food` settles low. The signal is right; `Linear(-1.0, 1.0)` over the whole perception radius is the wrong
shape for a destination a person walks to once a day. Recorded in `needs-decision/travel-cost-is-not-scored.md`
rather than tuned here.

No oscillation at a tick boundary, no clipping artefacts, no dead agents beyond the one the red line
represents.

## Pass 3 — Fresh eyes: a different seed

**GREAT.** Same pack variant at seed 42: `food` mean **0.167** against seed 7's **0.176**, at zero **0%** at
both. The direction and the magnitude hold, and the narrative is one sentence to a host developer:

> A place now reports how far away it is, so an activity can price the walk. Before, every place claimed to
> be at the edge of perception — so asking for `Distance` on a place-targeted activity set its score to zero
> and the activity never happened. Nothing warned; the activity just vanished.

## Verdict

Three GREAT. Shipped.

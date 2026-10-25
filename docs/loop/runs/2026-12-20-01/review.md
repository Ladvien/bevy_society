# Review

## Pass 1 — Behavior: does the one-line change do what it claims?

**GREAT.** `compliment` failures go **1 045 → 11**, interruptions **2 237 → 400**, and reverting the single
expression restores both numbers exactly. The change is causally tied to the observable, not correlated with it.

The chain of evidence, each step a measurement:

1. a per-activity breakdown said `compliment` was 47% of failures — not `sleep`, which is what I had assumed;
2. instrumenting **every** exit path at once put all 1 045 on `gone=true`;
3. `binary_search` and `contains` disagreed **on the same call** — "gone" versus present, in a list of 24;
4. raw entity ids printed as `4294967234, 4294967233, …` — **strictly descending**, which is precisely what breaks
   a binary search.

Evidence: `evidence/before-after.png`.

## Pass 2 — Zoom: read the whole picture, including what moved the wrong way

**GREAT**, with one thing that had to be explained rather than waved through. Total endings **fell** 9 808 →
6 768. That looks like the village doing less, so I checked the activity mix instead of assuming:

| | sleep | work | eat | compliment | travel |
|---|---|---|---|---|---|
| before | 39% | 24% | 12% | — | 1.0% |
| after | 40% | 24% | 10% | 7% | 0.9% |

The shape is unchanged and `compliment` becomes **visible** at 7% — it was previously killed before it could
count. Fewer, longer activities is the right direction: the village stopped restarting the same thing.

Nothing saturates, nothing clips, travel stays ~1%, and sleep at 40% is near the 33% one 8-hour sleep a day
implies.

## Pass 3 — Fresh eyes: can a host explain it, and is the test honest?

**GREAT on the change, and deliberately uncomfortable on the test.**

A host reads one sentence: *an activity aimed at a person was being cancelled on the tick it began, because the
"is the target still here" check assumed a vector order that the crate does not guarantee.*

**Six attempts to reproduce this in a test all passed under the bug**: the real village fixture, a hand-built
village, a ring of 12 movers, a ring of 24, and two entity-churn variants. The defect needs the specific
`PersonId` ordering `examples/evidence.rs` produces. Rather than ship a test that reports protection it does not
give, the shipped test is a **threshold guard on the observable**, and its doc comment says so plainly.

That is the same lesson as last iteration: I would rather ship a guard labelled honestly than a reproduction
labelled confidently and wrong. But it is also the second iteration in a row where I could not test what I
shipped, and that is worth naming as a trend rather than a one-off.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT** for the change, **NOT YET** for test coverage — recorded,
not hidden.

## Not shipped, deliberately

- `ActivityTarget::Home` and `Place::capacity` — last iteration's top item, now measurably **not** the biggest
  problem (sleep was 162 failures of 2 237). It may still be worth doing; it is no longer first.
- The guide's scale curve — unverified for a fourth iteration.

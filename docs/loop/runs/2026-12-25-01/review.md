# Review

## Pass 1 — Behavior: is there now one village?

**GREAT.** `tests/common/village.rs` matches `examples/common/scene.rs` on all five numbers, `examples/evidence.rs`
reads the canonical constant rather than copying it, and `tests/village_layout.rs` fails if either drifts. The
harness output is **byte-identical**, so no behaviour changed — which is the correct result for a de-duplication.

## Pass 2 — Zoom: read the geometry, not the summary

**GREAT.** `evidence/layouts.png` draws both villages against the same 14.9 m circle a 4-hour timeout reaches from
the well. In the canonical layout the workshop (~8.1 m) is inside it and the far corner is 16.4 m; in the drifted
layout the workshop sits at 11.3 m and the far corner at 21.3 m. The drifted village is **29% wider**, which at
64 s/day is the difference between needing 4.4 hours and needing 5.7.

That is not a cosmetic difference in a test fixture. Three tests were walking around a village that no measured run
or clip has ever used, and they **passed**, which is exactly why nobody noticed.

## Pass 3 — Fresh eyes: what did the log get wrong?

**GREAT, and this is the iteration's finding.** Item 3 read: *"the harness's village builder should move where
`tests/` can reach it — carried over four iterations, and it is why four fixtures last iteration could not
reproduce the village."*

Every clause of that was wrong:

- the builder **had** been extracted to `tests/common/village.rs`;
- `tests/` **could** reach it;
- last iteration's fixtures failed because they were written against a **different geometry** than the harness
  measures — not because a builder was unreachable.

Four iterations of carrying an item I had described incorrectly. Reading the code instead of the log is what
resolved it in about ten minutes, and the corrected statement is a much worse defect than the one I had recorded.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**.

## Not shipped, deliberately

- Splitting arrival radius from contact radius (item 1). Still a schema change, and non-urgent since the clamp
  made the numeric fix affordable.
- `Place::capacity` enforcement at arrival (item 4). It is now documented as a known gap; enforcing it without an
  overflow destination makes the village worse, which was measured two iterations ago (400 → 440 interruptions).

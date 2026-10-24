# Plan: measure the activity mix, and find why the village is asleep 57% of the time

## Why this over the runner-up

Runner-up was `need-cycle`'s observable, a design question about rates. Lost on one sentence: **counting
the activity mix found a 2× discrepancy in what the village reports doing**, which is a defect rather than
a preference.

## Design

No library change shipped: the finding is a decision, and the fix changes a component a host reads.

What ships:
- `tests/drift.rs` — a `Place` activity pays **exactly** its declared duration, displacement included. It
  passes, and it exists because three of this iteration's hypotheses were that it did not.
- `tests/travel.rs` — asserts the honest behaviour and **fails**; ignored so it does not block later
  iterations, and `--ignored` shows it.
- `needs-decision/what-doing-reports-while-travelling.md` — the question, three options, a recommendation.

## What the measurement found

| activity | share of person-ticks |
|---|---|
| **sleep** | **56.8%** |
| eat | 14.9% |
| work | 14.8% |
| everything social | 9.2% |

One 8-hour sleep a day is 33%, and the run's own 350 sleeps are 28.6%. A villager tracked tick by tick has
`rest` **falling at exactly the decay rate** while its `Doing` says `sleep`: the payout has not begun,
because `arrived_day` is `None`. The person is walking to bed and reported asleep.

## Four hypotheses disproved first

| hypothesis | killed by |
|---|---|
| a `Place` activity overruns its duration | `elapsed_h = 8.004` on every sleep that ends |
| `satisfies` pays twice per tick | a 2-hour activity at 0.1/h paid 0.2010 — exact |
| `best_telling`'s tie-break starves a fresh rumor | three tests passed with no code change |
| a probe measuring overlapping sleeps | fixed with a queue; real range 8.00–28.00 |

Each is recorded in `review.md`, because the wrong answers are why the right one took this long.

## Touched

- `tests/drift.rs` (new), `tests/travel.rs` (new, ignored).
- `docs/loop/needs-decision/what-doing-reports-while-travelling.md` (new).
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `drift::a_two_hour_activity_pays_two_hours` | the payout is `rate · duration`, exactly — 0.2010 against 0.2 |
| `drift::being_pushed_away_does_not_extend_the_payout` | displacement does not extend it — 0.2004 |
| `travel::a_person_walking_to_bed_is_not_reported_as_sleeping` | **ignored, failing**: a person labelled `sleep` with `arrived_day = None` has `rest` falling at the decay rate |

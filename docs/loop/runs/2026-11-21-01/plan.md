# Plan: make the harness tell travelling from doing

## Why this over the runner-up

Runner-up was the `Doing`-while-travelling decision. Lost on one sentence: **that is blocked on the
operator, but the number that makes it look bad is a measurement artefact I can fix today** — and fixing it
reveals something larger than the labelling.

## Design

No library change. `src/` is untouched.

The evidence harness's `doing` column now labels a person `travelling:<activity>` when
`Doing::arrived_day` is `None`, and `<activity>` when it is not. That is the distinction `Doing`'s own doc
already draws; the harness was not making it.

## What it revealed

| label | share |
|---|---|
| `travelling:sleep` | **21.8%** |
| `sleep` | **14.7%** |
| `travelling:eat` | **12.6%** |
| `travelling:work` | **12.4%** |
| `work` | 9.1% |

| | |
|---|---|
| **travelling, total** | **62.5% of all person-ticks** |
| doing an activity | 37.5% |
| `sleep` | **14.7%**, against 33.3% that one 8-hour sleep a day allows |

## The defect that number points at

A crossing walk in the village is **375 ticks**. Eight activities are shorter than that — `snub` 17,
`compliment` 34, `chat` 51, `gossip` 51, `play` 85, `eat` 128, `relax` 256, `wander` 85 — and **five of
them do not score `Distance` at all**.

Measured candidate fix: adding `Distance` to `eat`, `relax` and `compliment` takes travelling from **62.5%
to 50.5%** and the need total from 0.526 to 0.513. Fair, small, and a definitions-file change — a host's
file, so it is `needs-decision/travel-cost-is-not-scored.md`.

## Touched

- `examples/evidence.rs` — the `travelling:` label.
- `tests/travel_cost.rs` (new, ignored) — pins the eight short activities and the five blind to distance.
- `docs/loop/needs-decision/travel-cost-is-not-scored.md` (new).
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `travel_cost::every_activity_is_reachable_in_less_time_than_it_lasts` | **ignored, failing**: eight activities shorter than a crossing walk, five of them blind to distance |
| `travel::a_person_walking_to_bed_is_not_reported_as_sleeping` | still ignored and failing — the labelling decision stands separate |

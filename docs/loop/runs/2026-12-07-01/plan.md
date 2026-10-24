# Plan: pin the cluster's failure and record its cause

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics — escalated. This is the last `pending` scenario, and two
iterations have been spent on `measured` scenarios whose claims nothing checked. `rumor-crosses-a-cluster`'s
claim was checked this time and **fails**, with the cause now established as arrival timing rather than any
of the six layers beneath it.

## Design

1. `tests/cluster_reach.rs` measures **the quantity the observable names** — how many of the second group of
   six ever hold the rumor — not `rumor_reach`, which divides by all twelve.
2. It asserts the current failure (`0 of 6` at both clocks) rather than a pass. Marking the scenario `measured`
   then requires **deleting that line deliberately**, which is the point: the number cannot drift to `measured`
   on the strength of a reading nobody took.
3. `scenarios.yaml` carries the measurement and the cause, and now names its `test:`.
4. `needs-decision/how-can-two-groups-ever-meet.md` records the three options.

## Measured

| | 64 s/day | 240 s/day |
|---|---|---|
| second group reached | **0 of 6** | **0 of 6** |
| person-ticks moved | 359 948 | — |
| bridge chosen | 36 489 | — |
| arrived | 6 720 | — |
| both groups arrived | 434 ticks | — |
| cross-group interactions | **0** | — |

## What was tried and measured

| attempt | result |
|---|---|
| crossroads with `gathering` | reach 0.167 → **0.167** — nearest-place rule sends everyone home |
| market with a tag the hearths lack | chosen **36 489** ticks, arrived **6 720**, still **0 of 6** |
| explicit cross-group `Interact` | **0 of 6** |

## Touched

- `tests/cluster_reach.rs` (new).
- `docs/loop/scenarios.yaml` — the measurement, the cause, and the `test:` line.
- `docs/loop/needs-decision/how-can-two-groups-ever-meet.md` (new).
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

# Plan: make `ActivityStarted`/`ActivityEnded` a balanced pair

## Why this over the runner-up

Runner-up was need starvation (`need3` at 0.0 for 78% of ticks), which is the more visible defect.
Lost on one sentence: **a starved need is a tuning problem, an unpaired message is a broken contract.**
Every host-facing consumer of the pair — barks, a HUD, a log, witness reactions — reads an end as the
close of something it was told about. The crate was emitting 53 ends for 55 decisions in the simplest
possible reproduction, and every downstream system inherits it. Fixed in one place, measured, and it
turned out to be the cheapest of the three symptoms.

## Design

The crate's own component already said the answer. `Doing::started_day` is documented "Game day the
activity started" and is stamped at **decision**, in the same `insert(Doing { .. })` the message
accompanies. `ActivityStarted` was the thing out of step.

- `decide` writes `ActivityStarted` for **every** activity it picks, travelling or not.
- `arrive_and_satisfy` no longer writes one. Arrival keeps its own meaning: it sets `arrived_day`,
  which begins the duration and the `satisfies` payout. The `MessageWriter` parameter is removed, so
  the system cannot write a second start by accident.
- `end_activities` is unchanged: it still ends whatever `Doing` it removes, and now that is always
  paired.

The doc comment on `ActivityStarted` states the contract, because a host reading the type is the one
who needs it: *one start when `Doing` appears, one end when it is removed, including a timeout.*

**Consequence a host will notice:** barks now fire when a person *decides* to do something rather
than when it arrives. That is the better reading of "Back to it." (which is `work`'s bark): a person
says it as it sets off, not an hour later on the doorstep.

## Touched

- `src/behavior.rs` — `decide` writes the start; `arrive_and_satisfy` stops; the type's docs.
- `tests/behavior.rs` — `every_activity_end_has_a_start`.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. No rate changed.

## Tests

| Test | What it proves |
|---|---|
| `every_activity_end_has_a_start` | a person that can *never* arrive gets `starts >= ends`, and at most the in-flight activity is unpaired |

Written **red first**: `0 starts, 149 ends`, then `150 / 149` after the fix. The bound is `>=` rather
than `==` because a run can stop mid-activity, which leaves one start legitimately unpaired — the
first draft asserted equality and was wrong for that reason, not because the fix was wrong.

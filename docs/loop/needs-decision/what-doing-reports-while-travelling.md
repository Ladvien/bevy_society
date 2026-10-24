# What should `Doing` say while a person is still walking?

## The problem, measured

`Doing` carries an activity and an `arrived_day`. Its doc says:

> A person re-decides only when its `Doing` ends. While `arrived_day` is `None` the person is
> **travelling**; the day it is set, the activity's needs and duration start counting.

But `Doing.activity` names the **target** activity from the moment it is chosen. So every reader — a
host's HUD, the evidence harness, a script, `Doing` itself — sees a person **sleeping** while it walks to
bed and is paid nothing.

Measured, `village-evening` at its own settings (64 s/day, 24 people, 17 game-days):

| | measured | expected |
|---|---|---|
| person-ticks reported `sleep` | **56.8%** | 28.6% (350 sleeps × 8 h a day) |

and one villager tracked tick by tick: `rest` at **0.087 and falling at exactly the decay rate** while
its `Doing` said `sleep`. The payout had not begun, because `arrived_day` was `None`.

The cause is a mismatch in time scales. `travel_timeout_hours` is **20** at 64 s/day, which is **3 414
ticks**; an 8-hour sleep is **1 365**. A person can walk toward its bed for two and a half times the
length of the sleep itself, labelled asleep the whole way.

## Why it is a decision and not a bug fix

`Doing.activity` is the host-facing answer to "what is this person doing?" Changing it changes what every
consumer reads, and the crate's own systems read it too — `activity_barks` places a bark at
`Doing.target`, `report_to_triggers` reports the activity, and the evidence harness tallies it.

`tests/travel.rs` asserts the behaviour this file is about, so it **fails today** and will pass when the
decision lands.

## Options

### 1. `Doing` gains a `travelling` reading, and readers check it

Keep `activity` as the target, add nothing, and document that a reader must check `arrived_day.is_none()`
before treating the activity as begun.

- **For:** no schema change, no message change. `Doing::arrived_day` already says it.
- **Against:** every existing reader is wrong until it is fixed, including the harness and the crate's own
  barks. A contract that is easy to read wrongly stays read wrongly.

### 2. A reserved `travelling` activity, with the target kept separately

`Doing.activity` is the reserved idle/travel entry while `arrived_day` is `None`, and a new field carries
the target activity.

- **For:** every reader is right by default; `activity_name` returns something true.
- **Against:** a host that switches on the activity name sees a new value; `Doing` gains a field, so
  existing construction sites change.

### 3. The timeout is capped at the activity's duration

Leave `Doing` alone and make `travel_timeout_hours` no longer than the shortest activity's duration, so a
travel-only entry cannot outlast the activity it names.

- **For:** one number, no schema change.
- **Against:** it does not fix the mislabelling — a person walking for 3 000 ticks is still reported
  asleep — and it makes long walks fail that would otherwise succeed.

## Recommendation

**Option 2.** The question "what is this person doing?" has one honest answer, and it should not depend on
every consumer remembering a second field. It also fixes the crate's own bark and trigger reporting, which
option 1 leaves to each of them.

Deferred because it changes a component a host reads and a field on it.

## What ships now

| item | status |
|---|---|
| `tests/travel.rs` | **failing**, asserting the honest behaviour |
| `tests/drift.rs` | passing, pinning that a `Place` activity pays exactly its duration |
| the measurement | in `docs/loop/runs/2026-11-19-01/` |

Four hypotheses were measured and disproved this iteration before this one: a `Place` activity overrunning
its duration, a doubled `satisfies` payout, a `best_telling` tie-break, and a probe measuring overlapping
sleeps. Each is recorded in the run's `review.md`, because the wrong answers are why the right one took
this long.

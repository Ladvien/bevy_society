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

## Option 1 shipped, in the form that does not depend on every reader remembering, 2026-12-18

The file recommended option 2 (a reserved `travelling` activity plus a new target field). That is still the
better *schema*, and still needs a host's approval because it changes a component a host reads and adds a field.

What shipped is option 1 **made safe**: `Doing` now carries the honest answer itself.

```rust
impl Doing {
    pub fn is_travelling(&self) -> bool { self.arrived_day.is_none() }
    pub fn label(&self, defs: &SocietyDefs) -> String { /* "travelling:<activity>" | "<activity>" */ }
}
```

This is not option 1 as written — option 1 said "document that a reader must check `arrived_day`", which leaves
every consumer to remember. Putting the check on the component means a host calls `doing.label(&defs)` and gets
something true, while a raw `doing.activity` read keeps its existing meaning (the **target**), so no existing
consumer changes behaviour.

**The evidence that this was the right place for it:** the crate's own evidence harness had already invented
`format!("travelling:{base}")` privately. When two independent readers converge on the same workaround, the
workaround belongs on the type. After the change the harness calls `label`, and `village-evening`'
`series.csv` and `summary.json` are **byte-identical** — the helper is a faithful extraction, not a behaviour
change.

**What is still a decision.** Option 2 remains open, because `Doing.activity` still names the target and a host
that switches on the activity *name* (rather than reading `arrived_day`) still sees `sleep` for a walker. The
difference now is that the crate offers a correct alternative next to it.

**Tests.** `tests/doing_label.rs` pins the prefix while walking, its removal on arrival, and that a raw
`activity` read still names the target — three properties, and the walking one fails if the prefix is deleted.
`tests/travel.rs` keeps its `#[ignore]`: it asserts the *stronger* claim that a walker is being **paid**, which
is a different defect (the clock decision), not this one.

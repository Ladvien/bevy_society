# Why do `Field` considerations and `Gradient` targets score nothing?

> **Resolved 2026-11-29: they score now.** The wiring was one of three causes; the other two were in the
> tests that had declared the feature dead. See "What actually went wrong" at the end.

## The state, measured

With the `fields` feature on (the default), all of this is true at once:

- `SocialFieldPlugin` builds `SocialFields`, deposits presence and commotion every tick, evaporates
  and diffuses, and refreshes its peaks. Verified: at a crowded cell `field_value(Crowd, …)` returns
  **1.0** with sample == peak.
- `SocialFields::gradient_target(input, own, pos, toward)` returns a real point four cells along the
  slope. Verified: `Some(Vec2(4.0, -4.2e-7))` on a crowd.
- The decision does **not** call either. `signal_value`'s `Field` arm returns a constant `0.0`, and
  `targets_for` returns no targets for `ActivityTarget::Gradient`.

So a definition that uses them compiles, and the activity is never chosen:

```
activity "follow" with target Gradient(Crowd, true) over 60 ticks: 0 person-ticks
```

## What I tried, and where it stopped

Wiring the two call sites is mechanical — `field_value` and `gradient_target` are public, and the
decision already holds positions and (now) faction memberships. I wired both, threaded
`Option<Res<SocialFields>>` through `decide_activities` → `score_option` → `signal_value` and through
`targets_for`, and wrote two tests. Both failed, and I could not find why in the time this iteration
had:

- The `Field` arm provably runs (traced), with the actor's real position.
- `decision.position` comes from `Perception.pos`, which is correct.
- `field_value` at that same cell, at the same moment, returns 1.0.
- Yet the activity scores 0 and the person idles, every tick, for 60 ticks.

The one ordering fact established: `SocialFields` does not exist after tick 1 (it is created by a
deferred command in `Clock`) and `deposit_fields` runs in `Relate`, *after* `Decide`. So a decision
reads the *previous* tick's grid, and on the first tick reads nothing. That is defensible for a
person choosing from what they can see, but it is not the whole explanation, because by tick 20 the
grid is populated and the score still comes out 0.

**The wiring was reverted rather than shipped broken.** The change is in this iteration's git history
for whoever picks it up.

## What did ship

A startup `warn!` naming every definition that relies on a `Field` consideration or a `Gradient`
target, so a host does not write one and wonder why the activity never happens. It is the same shape
as the `Script` warning that preceded it, and it is pinned by `tests/fields_unscored.rs`, which
asserts the facts the warning keys on (both accessors see the offending activity; the starter pack
triggers neither).

`SocietyDefs::activity_signals()` and `SocietyDefs::has_gradient_activity()` are public for that
check, and `Signal` is now public with them.

## Options for the next attempt

1. **Wire the two call sites** (the attempted approach), and find the scoring bug with a
   single-person minimal case where the field has one cell of value 1.0 and the activity has no other
   consideration. My own attempts always had a crowd, which may be hiding something.
2. **Move `deposit_fields` into `Perceive`** so a decision reads the tick's own grid rather than the
   previous one. That is a scheduling change, and it alters what "what a person can see" means.
3. **Remove `Field` and `Gradient` from the accepted schema** until they work, so a host cannot write
   a definition that silently does nothing. That is a defs-schema change and would break a host that
   already has one.

## Recommendation

**Option 1 with option 2's diagnosis first**: find out why the score is 0 *before* deciding whether
the ordering needs to change. The warning means no host is misled in the meantime.

## What actually went wrong, 2026-11-29

Wired and working. Measured: `field_value(Crowd)` at the chooser reads **0.120**, `gradient_target`
returns `Some(Vec2(-2.0, 0.0))`, and a `Field`-only activity is chosen **6 times in 2 400 ticks** where it
had been chosen **0**.

Three causes, and the two that made the earlier attempt look like a scoring bug were in the probe:

1. **A crowd in one cell has no slope.** `field_value` divides a cell by the channel's **peak**, a global
   maximum, so every cell in the hottest part of the grid reads 1.0. That is a plateau, and
   `gradient_target` correctly returns `None` because there is no direction to walk. Every earlier attempt
   "always had a crowd", as this file suspected; the crowd was also *in one place*, which is the thing that
   hid it. A gradient needs high cells and low ones adjacent: nine people at the origin, chooser 2 m away.
2. **The fallback outlasted the test.** `idle` lasts one game-hour. At `seconds_per_day = 240` with
   `tick_secs = 1/64` a tick is **0.0015625** game-hours, so one fallback holds **640 ticks**. The original
   test ran 60; two of mine ran 400 (0.625 game-hours). The person was still idle every time.
3. **The wiring**, which also needed `Decision::own_factions` — the `OwnFaction` and `OtherFactions` inputs
   are faction-relative and the decision carried no memberships.

**The warning changed meaning.** It now fires only when the `fields` feature is **off**, and only compiles
into that build: with the feature on a `Field` definition is an ordinary definition. With it off the
definitions file rejects `Field` and `Gradient` at parse time, so the warning is a backstop rather than the
contract.

`tests/field_scores.rs` pins the behaviour.

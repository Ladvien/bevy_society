# Research: a documented feature that could never fire

## The gap

With the `fields` feature on — the default — all of this was true at once:

- `SocialFieldPlugin` built its grids, deposited presence, evaporated, diffused, and refreshed peaks.
- `SocialFields::field_value(Crowd, ..)` returned **1.0** at a crowded cell.
- `SocialFields::gradient_target(..)` returned a real point, e.g. `Some(Vec2(-2.0, 0.0))`.

And the decision read **neither**. `signal_value`'s `Field` arm was the constant `0.0`, and
`targets_for` returned `Vec::new()` for `ActivityTarget::Gradient`. So a definition using either was
accepted by the definitions file and could never be chosen. The crate's answer was a startup `warn!`.

A warning is right for a known gap. It is wrong for a *documented feature*: the README lists fields, the
`fields` feature is on by default, and the guide presents the definitions as data a host writes.

## What the previous attempt got wrong

The decision file records that wiring both call sites was tried and the tests still failed, with two facts
established and the cause not found. The facts were:

- The `Field` arm provably ran, with the actor's real position.
- `field_value` at that same cell, at the same moment, returned 1.0.
- Yet the activity scored 0, every tick, for 60 ticks.

Three separate things were wrong, and only one of them was the wiring.

### 1. The test's crowd had no gradient

The first test packed nine people into **one cell** and expected a `Gradient` target. That cannot work:
`field_value` divides a cell by the channel's **peak**, a global maximum, so every cell in the hottest part
of the grid reads 1.0. That is a plateau, `gradient_target` correctly returns `None`, and the option has no
target. A gradient needs high cells *and* low ones adjacent.

Measured with a knot of nine at the origin and the chooser 2 m away: `field_value(Crowd)` = **0.120** at the
chooser, and `gradient_target` = **`Some(Vec2(-1.9999998, 0.0))`**. Both work.

### 2. The fallback outlasted the test

`idle` lasts **one game-hour**. At `seconds_per_day = 240` with `tick_secs = 1/64`, one tick is
**0.0015625** game-hours, so a single fallback holds for **640 ticks**. The original test ran 60, and two of
mine ran 400 — 0.625 game-hours. The person was still idle in every one of them, so of course the activity
was never chosen.

### 3. Only then: the wiring

`Signal::Field` needed `SocialFields`, the actor's cell and its factions; `targets_for` needed the same for
`Gradient`. `Decision` carried no faction memberships, so that had to be added too.

## Source

The field model is `bevy_stigmergy`; the crate's own citation for the mechanism is in
`docs/research/needs-and-utility.md` and `src/fields.rs`. Nothing in the literature was needed here — this
was a crate defect, not a modelling question.

## What believable output looks like

An activity whose only consideration is a populated field is **chosen**, and walks toward the field's
slope. Measured after the fix: `join` chosen **6 times in 2 400 ticks** against a 1-hour fallback, from a
standing start where it had been chosen **0 times in 400**.

# A village begins with empty larders

> **Assumed decision (owner may override), 2026-10-30-27: option (a), `GoodDef::initial`, the starter pack's bread at
> `1.0`.** Measured and researched in `docs/loop/runs/2026-10-30-27/research.md`.

## The state

A person is created with every need at its `initial` level (1.0) and every good at `0.0`. The starter pack's `eat`
consumes bread and its `bake` is a one-hour dawn errand, so a fresh society holds no food for its first 22 game-hours.
A village of 24 at seed 99: **99.8 %** of game-day 1's person-ticks have an empty larder and **38.8 %** of game-day
2's have `food` below 0.05; days 3-10 sit at 1.4-10.7 %. It was filed as "needs hit the floor" and is a start-up
transient, not a steady-state shortage.

## Options

- **(a) `GoodDef::initial`, a starting larder per good** (picked). The schema gains one optional field whose default
  (`0.0`) reproduces every existing pack. A person starts as a household that holds what the pack says one holds, the
  way they start with each need at the pack's level. The starter pack writes `bread` at `1.0`.
- **(b) A warm-up the host runs before it shows anything** (`fast_forward` a day). No schema change, but a host that
  wants to watch a society from its first minute cannot discard its first day, and every host must know to do it.
- **(c) Stagger or randomise the start** (initial needs spread, an endowment draw per person). It would also break the
  lockstep of every first crossing, but it needs a per-person draw in a fixed order and is a model of its own; it does
  not give the first dawn any bread.

## Why (a)

Reversible (set `bread` back to `0.0`), conventional (it mirrors `NeedDef::initial`), touches no id table so a live
`reload_defs` accepts it, and it is the smallest change that makes the first game-day read like the days after it.

## What the owner can override

The value `1.0` is a design default: `Tuning::price_full_store` and `Tuning::trade_reserve` are both `1.0`, so a person
starts with a full store and no surplus to give away; no paper gives a number of loaves. Override by editing `initial`
in `assets/starter.society.ron`; `0.0` is today's behaviour.

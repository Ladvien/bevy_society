# Households that provision themselves: the starter pack's `bake` runs on the clock

> **Assumed decision (owner may override), 2026-10-30-28: ship the capability and the overlay, leave the starter pack
> alone (option (b)).** The default flip is one patch away (`docs/loop/runs/2026-10-30-28/provisioning-default.patch`) and is
> blocked by exactly one protected assertion. Measured and researched in `docs/loop/runs/2026-10-30-28/`.

## The state

The starter pack's `bake` is a one-hour dawn errand (`Hours(6, 7)`, the `craft` skill) drawn against nine other options by
`weight · score`. Nothing in it tells a person how much bread it holds. It is an open loop: stable only if the clock's output
happens to equal what the village eats. On the evidence village scaled to 60 people it makes 0-3 bakes in ten game-days
(the 24-person village: 0.5-2.7 a day), the town's bread falls from 1.0 to 0.05 a head, and at the library's own 240-second day 86 % of
person-ticks sit at `food` below 0.05 by game-day 12 (HEAD, `evidence/before/t7d240`). At 64 seconds a day it is 100 % from
about game-day 17 (pass 27, `runs/2026-10-30-27/eval.md`).

`Input::Stock(good)` (this pass) lets a consideration read the actor's own shelf. The overlay
`examples/assets/provisioning.society.ron` replaces `bake` and `eat` with versions that use it: a hungry household with a
bare shelf bakes in daylight, `eat` is priced (not gated) at `0.05` of its score on an empty shelf. The same town holds
1.4 loaves a head and 23 % at the floor over its last six game-days (`evidence/after/t7d240`; the 64-second-day towns of 60 and 40 end at 1.1-1.3 loaves and 23-38 %).

## Options

- **(a) Replace the starter's `bake` and `eat` now.** The town is fixed for every host by default.
  **Blocked:** it makes `tests/starting_larder.rs::a_starting_larder_removes_the_second_days_famine` fail its first
  assertion, `empty > 0.2` ("the transient this guards against has to be there to cure"). On the overlay a village whose
  larders start empty has a day-2 floor share of **0.057**, not 0.2, because its households bake the first time they are
  hungry. The *second* assertion (`stocked < empty / 2`) still holds (0.000 < 0.029). The loop's rule is that a protected
  check is never removed or loosened by an assumed decision, so the pass cannot take this one.
- **(b) Ship `Input::Stock` and the overlay opt-in; record the flip** (picked). Reversible, touches no protected check, and
  a host that wants a town can adopt it today (`docs/guide/authoring.md`). The default pack keeps its clock-driven `bake`,
  so a run of the default pack is bit-identical to 0.7.0 (`eval.md` compares `series.csv` byte for byte).
- **(c) Keep the dawn window and add the shelf term** (`Hours(6, 7)` plus `Stock`). Passes the pinned assertion (0.322) and
  does not fix the town (floor 70 %, bread 0.24 a head at game-day 10; measured, `eval.md`): the one-hour window is the
  defect, not the missing term.

## To take (a)

Apply `provisioning-default.patch` (the starter pack's `bake` and `eat` become the overlay's; the first assertion of
`a_starting_larder_removes_the_second_days_famine` is deleted, since the transient it guarded is cured by the pack). Then
`CHANGELOG.md` records the changed default and the README's economy sentence says the starter pack provisions itself. The
`rumor_book`, `friendship_bound`, `label_hysteresis`, `need_cycle`, `trade` and `economy` pins pass on the patched pack
(`eval.md`, "The flip").

## What the owner can override

The overlay's numbers are design defaults, measured: the daylight window `Hours(6, 20)`, the hunger centre `0.5`, the shelf
curve `Linear(-1, 1)` (a day's supply a head, `Tuning::price_full_store`), the `0.05` floor on `eat`. No paper gives them;
the structure (a balancing loop on a stock, appetitive before consummatory) is the literature's.

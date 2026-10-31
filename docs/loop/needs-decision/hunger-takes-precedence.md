# Hunger takes precedence: `ActivityDef::urgent`, and where it is switched on

> **Assumed decision (owner may override), 2026-10-30-29: add the optional `urgent` field to the activity schema, switch it on in the
> provisioning overlay only, at `below: 0.25`, rank 1 (option (b)).** Measured and researched in
> `docs/loop/runs/2026-10-30-29/`.

## The state

An activity's `rank` is static. `sleep` is rank 1 and everything else is rank 0, so a person at `food` 0.02 with bread on the
shelf draws `eat` against `compliment`, `work` and `play` by `weight · score`; about half of the person-ticks at the floor on `food`
were a person choosing something else (`research.md`). Maslow's prepotency, the Sims' need buckets and Dill's dual-utility ranks all
say a low need should win the choice outright, and a satisfied one should take no part.

`UrgentDef { need, below, rank }` on `ActivityDef` raises an activity's rank to `rank` while the actor's `need` is under `below`. It is
`#[serde(default)]`, so a pack without it is unchanged and decides bit for bit as before. The decision trace records the tier
(`ScoredOption::rank`).

## Options

- **(a) Put it in the starter pack's `eat` now.** Measured to make the default village worse, not better: the starter's `eat` prices an
  empty shelf rather than reading it, and its `bake` is a one-hour dawn errand outside the bucket, so a hungry person on a bare shelf
  keeps choosing a meal that feeds nothing (floor 19 → 27 % at 0.25 and 52 % at 0.35, bread a head 3.2 → 0.3 → 0.0). Rejected.
- **(b) Add the field; switch it on in the provisioning overlay, whose `eat` reads the shelf and whose `bake` answers a hungry
  bare household** (picked). The bucket then holds only options that can solve the hunger. The overlay is itself opt-in
  (`households-provision-themselves.md`), so a default-pack run is bit-identical to 0.7.0.
- **(c) Infer the bucket from `satisfies`** (a need under its line puts every activity that raises it in the top tier, the Sims'
  way), with a `critical` level on `NeedDef` instead of a field per activity. Smaller to author, but `bake` satisfies nothing and has
  to be in the bucket when the shelf is bare, so it needs a second, explicit field anyway. Rejected: the explicit field says what
  the pack means.

## To take it for every host

Flip the starter pack to the provisioning overlay (`households-provision-themselves.md`, one patch and one deleted assertion); the
`urgent` lines come with it. Until then a host that wants hunger to take precedence copies the overlay's two activities.

## What the owner can override

`below: 0.25` and `rank: 1` are design defaults. Maslow gives an order, not a level; `0.25` was picked from a sweep at 0.15, 0.25
and 0.35 (floor on the village 3.6, 2.7, 1.8 %; spells of hunger 7.4, 6.8, 6.0 game-hours): 0.25 takes the floor from 19 % to under 3 %
with the least intrusion on what people do while merely peckish, and 0.35 (the midpoint of `eat`'s own curve) buys one more point
for a larger share of the day spent eating. A rank of 2 would put hunger over `sleep`: a hungry, tired person at night would then
always eat first.

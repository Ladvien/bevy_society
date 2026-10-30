# 2026-10-30-26 — research: trigger bookkeeping that survives people, edits and reloads

Four open bug issues, all filed by the owner's review pass (2026-10-30): #62 (NeedBelow arming is one
flag for the whole town), #63 (`fast_forward` runs extra fixed steps after a live frame), #64 (a restore
applies once/cooldown state by trigger index), #65 (stale "a save does not carry ..." docs). Bugs come
before every enhancement; #62 is the oldest, #64 shares its save-format change, #63 and #65 are small.

## No new model

Nothing here adds a model, a formula or a default, so there is no paper to cite and no corpus search to
run. The contract being repaired is the one already documented:

- `src/defs.rs` `NeedBelow`: "When a person's need first falls below this level."
- `docs/guide/scripting.md` 157-158: "`NeedBelow` re-arms only once the need is back above `below + 0.05`,
  so a level hovering at the threshold does not fire every tick."
- `src/save.rs` `restore`: "every trigger's once/cooldown bookkeeping are set back to what the save
  recorded" and "nothing is written until everything is validated".

The `+ 0.05` re-arm band is the existing design default of plan D8 (a dead band, the usual comparator
hysteresis); it is unchanged and no source is claimed for it.

## What the contract means, stated as invariants

1. **Arming is per person.** A threshold crossing is a fact about one person's need. The state that
   remembers "this person already fired and has not recovered" is therefore keyed by
   `(need, threshold, person)`. A different person's level cannot arm or disarm it.
2. **One crossing, one event.** An event is `(need, threshold, actor)`. Every `NeedBelow` trigger with that
   need and threshold matches it. Two triggers on the same threshold must not each generate their own
   event (today they do: one arming slot per trigger, so a shared threshold dispatches twice).
3. **Arming state is not indexed by position in the defs.** `reload_defs` and `restore` both accept an
   edited defs file. A key made of `(need index, threshold bits)` does not move when a trigger is inserted,
   removed or reordered. The need table is already refused when it changes (`DefsMismatch`).
4. **Once/cooldown state follows the trigger's authored `id`.** `reload_defs` already does this; `restore`
   must do the same, with an id the defs no longer have dropped under one `warn!`.
5. **`fast_forward` runs exactly the ticks it reports.** One fixed step per loop pass, so the result is the
   society a live app has after the same number of ticks whatever the last frame's delta was.

## Behaviour that counts as believable

A person who gets lonely fires `lonely` once, when it falls below 0.3, and again only after recovering
past 0.35 and falling back. A second person falling below while the first is still below fires for the
second. Nothing fires every tick, and a `persons: [1]` filter sees person 1's own crossing.

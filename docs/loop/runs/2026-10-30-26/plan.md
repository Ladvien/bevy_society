# 2026-10-30-26 — plan

## #62 NeedBelow arming per person (`src/rules.rs`)

- `NeedArmed(Vec<Vec<bool>>)` becomes `NeedDisarmed(BTreeSet<(u16, u32, u32)>)`: `(need index, threshold
  bits, person id)` of every person who fired and has not recovered. Absent = armed, so a new person, a new
  threshold and a reload need no bookkeeping.
- `thresholds()` returns each distinct threshold once, in first-definition order, so a shared threshold
  makes one event per crossing (the event already matches every trigger with that need and threshold).
- Per person and threshold: `level < threshold` and not in the set -> insert, push the event;
  `level > threshold + 0.05` -> remove. Between the two nothing changes (the dead band).
- Each tick the set is pruned to persons that still exist and thresholds the defs still have (a despawned
  person, a reloaded trigger). Skipped when the set is empty.
- No new `Tuning` field; the 0.05 band is unchanged.

## #64 restore remaps trigger state by id (`src/save.rs`, `src/rules.rs`)

- `TriggersSave::ids: Vec<String>` (new, `#[serde(default)]`): the authored id of each trigger at the index
  the `fired` and `cooling` rows use. `capture` writes it. `restore` maps every saved index through it to
  the index the current defs give that id; an id the defs lack is dropped, one `warn!` naming the ids.
  An empty table (a file written before this change) applies indices as before.
- `TriggersSave::need_armed` is removed; `need_disarmed: Vec<(u16, u32, u32)>` replaces it. An older file's
  `need_armed` is ignored on read (no `deny_unknown_fields`): everyone is armed, so a person already below a
  threshold fires once after the restore.
- `VERSION` 4 -> 5, with an `upgrade` step 4 -> 5 that has nothing to rewrite.
- `reload_defs` and `restore` share one index-remap helper (`TriggerRemap`) in `rules.rs`.

## #63 `fast_forward` (`src/fast_forward.rs`)

- Zero `Time<Virtual>`'s delta for the duration (`advance_by(Duration::ZERO)`), so `run_fixed_main_schedule`
  adds nothing but the one timestep per pass; put `Time<Virtual>` and `Time` back afterwards.
- `wanted` is checked against `u64` range and `checked_add`ed to the clock's tick; a count that does not fit
  is `BadDays`.
- A world with no `Time<Fixed>`, `Time<Virtual>` or `Time` is `NoTimeStep` (was a panic).

## #65 docs

Four "does not carry" sentences in `src/core.rs` and `docs/guide/concepts.md`, and the `persons` row of
`docs/guide/persistence.md`, rewritten to what `capture` writes.

## Tests (each fails on `main`)

- `tests/need_below.rs`: one firing per person per crossing over 10 ticks; a second person crossing fires for
  the second; `persons: [1]` fires for person 1 with both low; two triggers sharing a threshold fire once
  each per crossing; re-arming after recovery above `below + 0.05`; a threshold edit through `reload_defs`
  keeps a still-below person quiet.
- `tests/save.rs`: a save restored after a trigger was inserted ahead keeps `welcome` spent and lets `dawn`
  run; a person below a threshold at save time does not fire again after restore.
- `tests/fast_forward.rs`: after a live frame, exactly the ticks asked and the live app's hash; an absurd
  `days` is `BadDays`; a world without `TimePlugin` is `NoTimeStep`.

Evidence: a behaviour-contract fix with no model change, so no before/after plots; the issue's own
reproduction is the before/after (firings per tick). Precedent: passes 18-21.

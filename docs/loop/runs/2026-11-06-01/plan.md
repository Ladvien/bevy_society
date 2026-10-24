# Plan: bound `Knowledge`, and make the harness follow the rumor it planted

## Why this over the runner-up

Runner-up was the rumor-book retirement rule. Lost on one sentence: **that one changes what a host sees
in `RumorBook.alive`, so it is a decision, not a fix** — and it is now filed as
`needs-decision/retiring-untellable-rumors.md` with the measurements to make the call. The `Knowledge`
cap changes no observable behaviour at the defaults (verified: all 4 200 series rows and the whole
narrative byte-identical), matches two existing bounds, and removes real growth.

The harness fix comes with it because the same investigation proved the column every earlier iteration
read was measuring a different rumor.

## Design

**`SocietySettings::knowledge_capacity: usize`, default 64.** Beside `memory_capacity`, because a
capacity is not a rate and so does not belong on `Tuning`.

`Knowledge::insert` takes `(known, capacity, now_days)` and, when the list is over capacity, forgets
the entry with the lowest `belief · 0.995^(hours since heard)` — the crate's own `telling_weight` minus
the valence `Knowledge` cannot see, so it drops what the person would have been least likely to tell.
Ties break on the higher `RumorId`, so the kept set does not depend on insertion order. `capacity == 0`
keeps nothing, mirroring `Memories::record`.

The four call sites thread the capacity through: `plant_rumor` (used by `beliefs.rs`, `reputation.rs`
and `script_apply.rs`) and `gossip`.

**Harness.** `plant()` now returns the `RumorId` it creates (the book's length before the message is
applied, because `plant_rumor` appends), the run loop carries it, and `snapshot_row` reads *that* id.
Before the plant the column is `0.0` rather than another rumor's reach — the first draft spliced
`RumorId(0)` in front of the planted rumor and drew a line that fell to zero at the plant tick, which
the plot caught.

## Touched

- `src/core.rs` — the field and its default.
- `src/beliefs.rs` — capped `insert`, retention rule, `settings` into `apply_planted_rumors`.
- `src/rumor.rs`, `src/reputation.rs`, `src/behavior.rs`, `src/script_apply.rs` — thread the capacity.
- `examples/evidence.rs` — track the planted rumor; document the origin choice.
- `tests/knowledge_bound.rs` (new), `docs/loop/needs-decision/retiring-untellable-rumors.md` (new).
- `CHANGELOG.md`, `docs/loop/log.md`.

## Tests

| Test | What it proves |
|---|---|
| `a_person_does_not_remember_every_rumor_forever` | with `knowledge_capacity = 32` and a book of 325 rumors, nobody holds more than 32 |

Written **red first**: `book=325 knowledge max=287 of capacity 32`.

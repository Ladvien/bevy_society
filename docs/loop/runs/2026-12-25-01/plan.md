# Plan: one village, in one place

## Why this over the runner-up

Item 1 was "split arrival radius from contact radius" — a schema change, and the clamp has made it non-urgent.
Item 3 was mine and had blocked four iterations of test-writing. Reading the code instead of my log showed the
item was **mis-stated**: the builder had already been extracted, and the real defect was that a third copy had
drifted from the canonical one.

## Change

1. `tests/common/village.rs::PLACES` corrected to the canonical coordinates. Farthest pair 21.3 m → **16.4 m**.
2. `examples/evidence.rs::PLACE_POSITIONS` now **reads** `common::scene::PLACES[..].1` instead of duplicating it.
3. `tests/village_layout.rs` — three tests pinning the canonical table, the 16.4 m span, the shared builder's
   agreement, and that the village's own definitions still compile.

## Touched

- `tests/common/village.rs` — five numbers, and a comment saying why they must match.
- `examples/evidence.rs` — the copy replaced by a read of the canonical constant.
- `tests/village_layout.rs` — new.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## Verification

| check | result |
|---|---|
| `series.csv` before vs after the harness refactor | **byte-identical** |
| existing village tests after the layout correction | `determinism` and `friendship_bound` still pass |
| drift re-introduced deliberately | `village_layout` **fails**, naming `(8, -6)` against `(6.5, -5)` |
| `cargo test` both feature sets | **68** binaries |

## What this buys

Last iteration's four failed fixture attempts were blamed on the builder being unreachable from `tests/`. It was
reachable. The fixtures failed because they were written against a layout that did not match the harness being
measured — and the drift was invisible, because three tests were passing on the wrong village. That is now a test
failure with a message that names both coordinates.

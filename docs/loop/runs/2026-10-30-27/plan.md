# 2026-10-30-27 — plan: a person is created holding their starting larder

## Change

`GoodDef::initial` (units of the good every new person holds), the way `NeedDef::initial` is the level every new
person's need starts at. Default `0.0` (`#[serde(default)]`), so every pack that does not write the field is
byte-identical to HEAD. The starter pack sets `bread` to `1.0`.

## Touched

- `src/defs.rs`
  - `GoodDef { id, initial: f32 }` (`#[serde(default)]`); doc names the three crate numbers it matches.
  - `SocietyDefs::good_initial: Vec<f32>` (`pub(crate)`, parallel to `goods`). `add_good` validates `initial` is finite
    and not negative (`DefsError`, naming the good). `goods` stays `Vec<String>` so no caller moves.
  - `size_person_state`: a new `Stock` is `initial` per good, a short `Stock` grows with `initial` for the goods it
    lacks (the same rule needs follow), both clamped to `Tuning::stock_ceiling` (takes `Res<SocietySettings>`).
    A host-supplied `Stock` of the right length is untouched.
- `src/save.rs`: an empty saved `stock` restores as a zero-filled `Stock` when the defs have goods, instead of
  removing the component. Without it the first tick's `size_person_state` would hand a version-1 file's people the
  starting larder, and `SocietySave::upgrade` documents that they hold nothing. Nothing else in the file format moves;
  `VERSION` stays 5.
- `assets/starter.society.ron`: `(id: "bread", initial: 1.0)`, with the comment that holds the measurement.
- `examples/evidence.rs`: `floor0..floor3` columns in `series.csv`, the number of villagers whose need *k* is below
  0.05 at that tick (the metric of this pass); no behaviour change.
- `tools/plot_larder.py`: the per-game-day table and the plot.
- Docs: `docs/guide/authoring.md` (goods), `docs/research/economics.md` (the note and DOIs), `CHANGELOG.md`,
  `docs/loop/needs-decision/a-village-begins-with-empty-larders.md` (schema change: options and the pick), README
  current state if it names the starting stock.

## New `Tuning` fields

None. The larder is authored data, like a need's `initial`; the ceiling it clamps to is the existing
`stock_ceiling`. No rate is introduced.

## Tests

- `tests/starting_larder.rs` (new): a pack with `initial: 1.5` makes every spawned person hold 1.5 at tick 1; an
  `initial` above `stock_ceiling` is clamped to it; a person spawned with a short `Stock` grows the missing good by
  `initial` and keeps the loaves they had; a host-supplied full-length `Stock` is left alone; a negative or NaN
  `initial` is a `DefsError` naming the good; the field is optional (`initial` omitted = 0.0).
- The mechanism, paired: two villages from the same seed whose packs differ only in `bread`'s `initial` (0 vs 1),
  two game-days. With 0 the day-2 floor share is above 20 % (the transient this pass found); with 1 it is under half
  of that. This fails on `48e2260`'s starter pack (initial absent = zero) and passes after.
- `tests/save_upgrade.rs` (existing) keeps its meaning: a version-1 file's people hold nothing after restore.
- Determinism and save/reload: `tests/determinism.rs`, `tests/save.rs` (existing), and the full suite in both
  feature sets.

## Gate and evidence

- `cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo
  doc --no-deps --all-features`; `cargo test --release` and `--no-default-features`; `cargo build --no-default-features`.
- Protected pins most at risk (a changed economy moves the whole village): `rumor_book`, `friendship_bound`,
  `label_hysteresis`, `need_cycle`, `trade`, `economy`. If one fails on its own threshold the larder ships as
  the pack's default only if it can without loosening a pin; otherwise the field ships and the pack keeps `0.0`
  (as pass 24 shipped the friend brake), logged.
- Evidence at identical seeds: village-evening seeds 7, 42, 11 and 99, 17 game-days; town of 60 at seeds 7 and 99, 30
  game-days; before = HEAD, after = this change. `series.csv` stays ignored; `tools/plot_larder.py` writes the
  figure and `floor.md`.
- Perf: `examples/benchmark.rs` at 100 / 1 000 / 5 000 before and after (the change adds one `Res` to a system that
  runs on every tick, and a `Vec` clone only on the tick a person is created).

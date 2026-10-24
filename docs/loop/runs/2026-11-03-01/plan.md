# Plan: make the documentation a host can actually run

## Why this over the runner-up

Runner-up was the `Field`/`Gradient` wiring (candidate #1/#2), still the biggest behavioural gap. Lost
on one sentence: **a host cannot judge a `Field` bug in a society they could not start.** Both of the
crate's "here is how you begin" documents produce a broken or panicking app, and the fix is one
commit that makes every later iteration's evidence reachable by a stranger. The wiring is now #2 and
has a sharper repro than before (see `research.md`).

## Design: one source of truth, compiled

The failure mode was drift: three copies of one snippet (`README`, guide, example), one of which was
right. Copies that are checked by eye drift; copies that are compiled cannot.

- **`src/lib.rs`** gains the quick start as a **running doctest** (no `no_run` for the final
  assertion): 600 updates, then `assert!(clock.tick > 0)`. `cargo test --doc` therefore *executes* the
  snippet a host is told to copy, and a missing import or changed API fails the build.
- **`README.md`** carries the same text. `tests/readme.rs` extracts the doctest body from `src/lib.rs`
  (stripping `//!`), extracts the README's `rust,no_run` block, and asserts they are **byte-equal**
  after trimming. Editing one without the other fails.
- **`docs/guide/getting-started.md`** shows the `SocietyPlugins::build().disable::<DialoguePlugin>()`
  form and states plainly that the plugin **panics in `finish`** without a speech-bubbles camera. A
  test asserts the guide keeps both.

The `TimeUpdateStrategy` line is the load-bearing one and carries a comment saying why, because it is
the one line a host will delete as noise if nothing explains it.

## The harness changes

The brief's gates need things the harness did not have:

- `--seed <n>` and `--people <n>` overrides. The fresh-eyes pass requires "a different seed and
  population" and the harness had neither. `one_place_with` and `village_with` take a count; a
  population past 24 wraps the three factions instead of indexing out of bounds (which it did, and
  panicked — found by running it, fixed by running it).
- `clock_label` now prints `d3 20h43` rather than `20h43`. Without the day, a multi-day narrative reads
  as though the clock ran backwards at midnight — which is exactly how I first read it.

## Touched

- `src/lib.rs` — the quick start as a doctest.
- `README.md` — the same block, byte-identical.
- `docs/guide/getting-started.md` — the dialogue-plugin warning and the disable.
- `examples/evidence.rs` — `--seed`, `--people`, day-stamped clock labels, faction wrap.
- `tests/readme.rs` (new) — 3 tests pinning the three documents.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. This is a documentation and harness iteration; no rate changed.

## Tests

| Test | What it proves |
|---|---|
| `the_readme_quick_start_is_the_crate_doctest` | the README and `src/lib.rs` cannot drift |
| `the_quick_start_pins_the_fixed_step` | the one line that makes a headless app advance is present |
| `the_getting_started_guide_does_not_hand_a_host_a_panic` | the guide warns about the dialogue panic and shows the disable |
| `cargo test --doc` | the snippet **compiles and runs**, and asserts `tick > 0` |

Verification that they have teeth: deleting the `TimeUpdateStrategy` line from the README fails
`the_readme_quick_start_is_the_crate_doctest` — done, observed, restored.

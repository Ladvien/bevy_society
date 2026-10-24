# Plan: build the evidence harness, then fix what it shows

## Why a harness, before anything else

`docs/loop/log.md` has carried a "top 5" for two iterations, and every claim in it was
asserted from end-state numbers rather than seen. The loop's own step 6 makes plots and a
narrative a mandatory gate, and neither existed:

```
$ ls docs/loop/runs/*/evidence/
nf.log            # a stray cargo log, not evidence
```

So this iteration builds the gate first and lets it choose the work.

## Design

`examples/evidence.rs`, one scenario per invocation:

- builds the scenario (starter pack, or the village's merged definitions),
- runs it headlessly at one society tick per update, with `TimeUpdateStrategy` pinned,
- writes three artefacts:
  - `narrative.txt` — the tracked villager's exchanges and activity transitions in prose,
    with the game clock on each line,
  - `series.csv` — one row per tick: mean need levels, mean PAD, opinion variance on topic
    0, rumor reach, friend and rival counts, and a census of who is doing what,
  - `summary.json` — the end state, for a test or a log entry to assert against.

The scenarios are the ones `docs/loop/scenarios.yaml` already declares. The harness
implements the four it can build from the crate's own definitions and exits with a clear
message for the rest, rather than pretending.

No model changes. No new `Tuning` fields. The harness reads components and messages and
writes files; it is evidence, not behaviour.

## What it found, and therefore what was built

Two defects, both measured from the first run's artefacts:

1. **Run length.** 900 ticks is 4.1 game-days, and affinity needs about ten to reach the
   0.4 a Friend requires. Fixed by lengthening `village-evening` to 4200 ticks.
2. **Spawn layout and travel timeout.** A walker covers `speed / 64` metres per tick while
   the timeout is game hours, so at a 14 m layout every journey timed out. Fixed in the
   scenario: a 3.5 m grid and a 40-hour timeout. The coupling itself is escalated to
   `docs/loop/needs-decision/travel-time-in-two-clocks.md`.

## Touched

- `examples/evidence.rs` (new), `Cargo.toml` (one `[[example]]`),
- `examples/common/scene.rs` and `examples/capture.rs` (grid, places, settle steps, camera),
- `docs/loop/needs-decision/travel-time-in-two-clocks.md` (new),
- `docs/DEMOS.md`, `docs/loop/log.md`, and this run's directory.

## Not touched

`src/` behaviour. The two defects were the scenario's, and the model was left alone.

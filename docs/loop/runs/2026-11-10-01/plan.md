# Plan: one clip per feature

## Why this over the runner-up

Runner-up was the `Place`/`Person` completion asymmetry, which is the more important model question.
Lost on one sentence: **the user asked for the GIFs, and the instruction is explicit that each feature
needs one.** Doing it also found three real defects in the demo path, including a `RumorId(0)` bug that
made the hero clip's headline number read 0.000.

## Design

A `Scene` enum in `examples/capture.rs`, parsed from `--scene`, which chooses:

- **which overlays are added** — `ShowTies`, `RumorMarkers`, and one of `draw_mood_discs`,
  `draw_opinion_bars`, `draw_need_rings`;
- **the camera** — `orbit` for the hero, `close_camera` framed on the crowd for the overlay scenes,
  `ties_camera` for the graph;
- **the HUD's third line**, a `SceneLabel`, saying what the colours mean.

A new `examples/common/scenes.rs` holds the three overlay systems and the two cameras. Every system is
a read-only `Gizmos` view: a demo must not change the society it is demonstrating.

## Touched

- `examples/common/scenes.rs` (new) — mood discs, opinion columns, need halos, the two cameras,
  `MOOD_SPAN` and `OPINION_SPAN`.
- `examples/common/visuals.rs` — `MARK_HEIGHT` → chest height for ties and rings; `SceneLabel`; a dark
  plate behind the HUD so white text reads over grass and over night sky; `update_hud` reads the
  **newest** rumor.
- `examples/capture.rs` — `--scene`, the scene's camera and label, `stats()` reads the newest rumor.
- `docs/media/*.gif` — six clips. `docs/DEMOS.md`, `README.md`.

## New `Tuning` fields

None. This is demo code and documentation.

## Tests

The crate's tests are unchanged and all 27 binaries pass in both feature sets. The clips are verified
by **looking at them**: `docs/DEMOS.md` now records which frames were checked and what each must show,
including the note that a GIF must be **coalesced** before a single frame is extracted — `-layers
optimize` stores partial rectangles, and extracting one uncoalesced frame shows garbage that is the
extractor's fault, not the GIF's.

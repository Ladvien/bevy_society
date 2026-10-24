# Plan: rebuild the six clips, and give the crate a way to say "solid"

## Why this over the runner-up

The runner-up was continuing the loop's own list (the clock decision, `wander`'s ceiling, the chat/gossip
schema). All are escalated decisions. The demos were **not** — and they are the first thing a host sees. A
crate whose README shows six clips that all look the same, with people inside buildings, is misrepresenting
itself regardless of what the model does.

## Design

### Crate: `Obstacle`

```rust
#[derive(Component, Reflect)]
pub struct Obstacle { pub radius: f32 }
```

- Attach to anything round and solid. `steer_movers` converts each into ORCA's `(b, approach)` wall
  half-plane: `b` from the mover toward the centre, `approach = gap / dt` (0 when already overlapping, which
  lets it slide or back off).
- `push_out_of_obstacles` runs after steering and resolves contact, which `bevy_orca` explicitly leaves to
  the caller. Deterministic: pushed along the centre-to-mover line, with a fixed `+X` fallback at distance 0.
- Only obstacles within `WALL_RANGE` (2 m) constrain a step.

### Demo scene

- Places are **1.2 m stalls** (`PLACE_HALF = 0.6`), not 3 m cubes, and carry an `Obstacle` of `1.25 m` —
  the corner diagonal `0.6·√2` plus a body. Villagers gather on a **1.3 m** ring: outside the stall and
  inside `interaction_radius`, two constraints that used to be contradictory.
- Each faction's people live at **their own** place. All twenty-four shared one, so every sleep was a
  cross-village walk that timed out (measured: 25 interrupted sleeps against 4 completed).
- `seconds_per_day = 32`, not 8. A 3 m walk then costs 1.6 game-hours instead of 6.4 — watchable and
  finishable.
- Constant daylight (`SunClock(false)`); the clock is in the HUD. The last third of every old clip was dark.
- Ground plane 80 m across, so its edge never enters shot.

### Overlays

- **Indices corrected**: `NEED_CLIP = 2` (`food`, was 1 = `rest`), `OpinionBar(1)` (`honesty`, was 0 =
  `kindness`).
- Rumor halos read the **newest alive** rumor, not `RumorId(0)`. The correct function already existed and was
  never called.
- Opinion bars: radial on the ground, colour = sign, length = magnitude.
- Mood: diverging ramp, green → amber → red, so a mildly negative village is not uniformly red.
- Faction colours spread across the hue circle (orange / green / violet); teal and blue were 0.19 apart and
  indistinguishable after quantization.
- HUD: two lines, 13 px, bottom-left; the scene name appears only in the burned-in caption.
- Camera computed per frame from the crowd's mean position and spread.

### Encode

`tools/gif.py` in Pillow. `ffmpeg` and `magick` are not installed here and `tools/gif.sh` failed after the
frames were rendered, twice. One 256-colour palette per clip, built from the first frame.

## Tests

| Test | What it proves |
|---|---|
| `tests/obstacles.rs::a_walker_does_not_cross_a_solid_block` | with an `Obstacle` the walker stops at **2.35 m**; the control without one reaches **0.01 m** — through the block |
| `tests/sleep.rs` | an 8-hour sleep at home is not cut by the 4-hour travel timeout; and in a village most nights complete |

## Verified visually

Every clip inspected at frames 0, 120 and 239. See `eval.md`.

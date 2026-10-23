# Demos

`docs/media/hero.gif` is not hand-made. It is a recording of `examples/capture`, and this page is the
recipe, so a later capture can be compared with it.

## Capture

`examples/capture` builds the same village as `examples/village` (one scene, in
`examples/common/scene.rs`), runs it headlessly, and writes one PNG per frame.

```
cargo +1.98.1 run --release --example capture -- \
    --font /System/Library/Fonts/Supplemental/Arial.ttf \
    --out frames/hero \
    --frames 480
```

What it does:

- Settings: `seconds_per_day 64.0`, `tick_secs 0.1`, `start_hour 15.0`, so the clip covers about six
  game-hours — one afternoon into one evening.
- Six hundred unrecorded steps first, so ties have formed before the first captured frame.
- `warm_up(3)`, then 480 recorded frames at 30 fps (16 s, 15:30 → 21:30).
- At recorded frame 30 it writes the same `PlantRumor` the village example's `R` key writes: Bram stole
  bread, told by a weaver.
- The camera orbits from 20° to 50° around the origin at radius 24, height 16, at 720×405.
- It prints one JSON line and exits non-zero if fewer PNGs exist than frames requested:

```json
{"frames": 480, "rumor_reach": 0.0, "friends": 0, "rivals": 0, "barks": 0}
```

The clip is accepted when `rumor_reach ≥ 0.5`, `barks ≥ 6`, `friends ≥ 3` and `rivals ≥ 1`. If it is
not, tune **only** `examples/assets/village.society.ron` (the gossip, compliment and snub weights) and
the capture's `bark_chance`. Never change a crate default to make a demo look better: the defaults are
the model.

Simulated time is pinned with `TimeUpdateStrategy::ManualDuration(1/30 s)`, so a frame is exactly one
thirtieth of a second of game time whatever the host's clock does. Two captures on different machines
therefore record the same run.

## Encode

`tools/gif.sh` is the other half. It needs ImageMagick (`magick`) and the fonts named explicitly — a
default font path exists on exactly one machine and fails after the whole encode has been spent.

```
FONT=/System/Library/Fonts/Supplemental/Arial.ttf \
BOLD=/System/Library/Fonts/Supplemental/Arial\ Bold.ttf \
WIDTH=640 FPS=60 STRIDE=2 \
    bash tools/gif.sh frames/hero docs/media/hero.gif \
    "bevy_society: one rumor, one village, one evening"
```

- Two-pass palette (`palettegen=max_colors=128`, `paletteuse=dither=bayer:bayer_scale=3`) so colours
  do not shift between frames.
- `STRIDE=2` keeps every other frame and `FPS=60` restores real-time speed, which halves the file for no
  loss of information.
- The caption is burned in with ImageMagick because this ffmpeg build has no `drawtext` filter.

Fonts per host:

| Host | Regular | Bold |
|---|---|---|
| macOS | `/System/Library/Fonts/Supplemental/Arial.ttf` | `…/Arial Bold.ttf` |
| Linux | `/usr/share/fonts/liberation/LiberationSans-Regular.ttf` | `…/LiberationSans-Bold.ttf` |
| Windows | `C:\Windows\Fonts\arial.ttf` | `C:\Windows\Fonts\arialbd.ttf` |

## Comparing two captures

Compare the **printed JSON**, not the PNGs. Pixels differ across GPUs; the numbers do not. Two runs of
the same commit with the same seed must print the same JSON, and if they do not, the crate has lost
determinism and that is a bug, not a rendering difference.

## Visual check

Before committing a new GIF, look at frames 0, 160, 320 and 479 and at the GIF itself. Each must show:

- faction colours that tell villagers apart at a glance;
- legible balloons, with no clipped text;
- the HUD (`Day n · hh:mm`, the rumor's reach, friendships and rivalries);
- rumor rings appearing only after frame 30 and spreading;
- tie lines;
- a first frame that is not blank.

Any failure is a scene bug: fix `examples/common/scene.rs` or `visuals.rs` and capture again. Do not
paper over it in the encode.

# Demos

`docs/media/*.gif` are not hand-made. Each is a recording of `examples/capture` with a different
`--scene`, and this page is the recipe, so a later capture can be compared with it.

## One clip per feature

| file | `--scene` | what it shows |
|---|---|---|
| `hero.gif` | `village` | every overlay: rumor rings, tie lines, HUD |
| `mood.gif` | `mood` | PAD rings — colour is pleasure, radius is arousal |
| `opinions.gif` | `opinions` | one opinion column per villager |
| `needs.gif` | `needs` | food halos — full and green is fed, small and red is hungry |
| `ties.gif` | `ties` | the tie graph alone |
| `factions.gif` | `factions` | faction colours, no lines |

All six are the same village, same seed and same run: `{"frames": 240, "rumor_reach": 0.880,
"friends": 36, "rivals": 45, "barks": 2810}`. Only the overlay and the camera differ, which is the
point — a feature clip is a different *view* of one simulation, not a different simulation.

### Overlay scales are normalised to the measured range, not to [-1, 1]

The first mood clip was a field of identical dull-olive rings: PAD in this village spans only about
±0.4, so a full [-1, 1] colour scale maps every villager to the same colour. The scale is normalised
to `MOOD_SPAN = 0.45`, which is measured (4 200 ticks: pleasure [-0.387, 0.382], arousal [-0.361,
0.447], dominance [-0.413, 0.513]), and the opinion columns are normalised the same way. A clip whose
overlay cannot distinguish anyone is not a clip of that feature.

### Overlays draw above the mesh, not at ground level

`MARK_HEIGHT` is 0.06 m. Rings and tie lines drawn there are hidden by the villager capsules and by
the ground plane's depth: measured, the ties clip showed two short segments where the graph has
hundreds of edges. Everything now draws at chest or head height.

## Capture

`examples/capture` builds the same village as `examples/village` (one scene, in
`examples/common/scene.rs`), runs it headlessly, and writes one PNG per frame.

```
cargo +1.98.1 run --release --example capture -- \
    --font /System/Library/Fonts/Supplemental/Arial.ttf \
    --out frames/hero \
    --frames 240 \
    --scene village
```

What it does:

- Settings: `seconds_per_day 64.0`, `tick_secs 0.1`, `start_hour 15.0`, so the clip covers about six
  game-hours — one afternoon into one evening.
- Six hundred unrecorded steps first, so ties have formed before the first captured frame.
- `warm_up(3)`, then 240 recorded frames at 30 fps (8 s). The encode uses `STRIDE=2 FPS=60`
  (see below), which keeps real-time speed at half the file.
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

### Current result: all four thresholds met

```
{"frames": 480, "rumor_reach": 0.360, "friends": 80, "rivals": 37, "barks": 3648}
```

`rivals 0` was the last one outstanding, and it took three fixes to reach: `snub` was gated on low
affinity so the village never snubbed anyone it knew, script considerations could not see their
target, and `witness_reaction` signed approval by the witness's opinion of the *norm* rather than of
the *act* — so a kindness-valuing witness rewarded an insult. With those fixed the village forms
both friendships and rivalries, which is the picture the clip was always meant to show.

The earlier clip ran 900 ticks — 4.1 game-days — and formed no friendships at all. Ties were never
broken: `TieLabel::classify` needs `affinity ≥ 0.4`, affinity moves at `mere_exposure_per_day = 0.05`
per day against `affinity_relax_per_day = 0.01`, and four days is not enough. The same village run for
4200 ticks (17 game-days) forms 125+ friendships and the rumor reaches half of it. The capture now
settles 3800 frames before recording, and the village spawns on a 3.5 m grid so its people can actually
reach each other — at `1.4 / 64` metres per tick a 14 m walk was 640 ticks and every activity ended
interrupted.

`rivals 0` remains. No rivalry forms because `snub`'s consideration
(`Affinity Logistic(-8, 0.3)`) needs a negative affinity, and nothing in this village creates one:
gossip about Bram pushes *his* regard down, but he is also someone people compliment. A rivalry
scenario needs a deliberate snub, which is the next iteration's candidate.

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
- The caption is burned in with ImageMagick because this ffmpeg build has no `drawtext` filter. It sits
  at the bottom centre with a thin black stroke: the scene's own HUD owns the top-left, and a caption
  there covered it.

`ffmpeg` and `magick` are not installed on every host; they are on `big`, so the encode runs there:

```
rsync -az frames/hero/ big:/tmp/bevy_society_hero/
scp tools/gif.sh big:/tmp/bevy_society_gif.sh
ssh big 'bash /tmp/bevy_society_gif.sh'   # with FONT/BOLD/WIDTH/FPS/STRIDE exported
scp big:/tmp/bevy_society_hero.gif docs/media/hero.gif
```

Fonts per host:

| Host | Regular | Bold |
|---|---|---|
| macOS | `/System/Library/Fonts/Supplemental/Arial.ttf` | `…/Arial Bold.ttf` |
| Linux (big) | `/usr/share/fonts/liberation/LiberationSans-Regular.ttf` | `…/LiberationSans-Bold.ttf` |
| Windows | `C:\Windows\Fonts\arial.ttf` | `C:\Windows\Fonts\arialbd.ttf` |

## Comparing two captures

Compare the **printed JSON**, not the PNGs. Pixels differ across GPUs; the numbers do not. Two runs of
the same commit with the same seed must print the same JSON, and if they do not, the crate has lost
determinism and that is a bug, not a rendering difference.

## Visual check

**Coalesce before you look.** `tools/gif.sh` writes with `-layers optimize`, which stores later frames
as partial rectangles (`640x311+0+49`). Extracting one frame with `magick file.gif[N]` yields a
corrupted image — that is the extractor, not the GIF. Use `magick file.gif -coalesce out_%03d.png`.

Before committing a new GIF, look at frames 0, 120 and the last one **after coalescing**. Each must show:

- faction colours that tell villagers apart at a glance;
- legible balloons, with no clipped text;
- the HUD (`Day n · hh:mm`, the rumor's reach, friendships and rivalries);
- rumor rings appearing only after frame 30 and spreading;
- tie lines;
- a first frame that is not blank;
- the scene's own overlay legible as the thing it names — for `mood`, rings that differ in colour *and*
  size between villagers; for `opinions`, columns that differ in height; for `ties`, a graph rather
  than a handful of segments.

Last checked 2026-11-10, all six clips, frames 0 / 120 / 239: HUD plate legible over grass and over
night sky, captions correct, every overlay distinguishing at least two villagers, no blank first
frame. The `factions` clip deliberately carries no tie lines, so its three colours are the only thing
on screen.

Any failure is a scene bug: fix `examples/common/scene.rs` or `visuals.rs` and capture again. Do not
paper over it in the encode.

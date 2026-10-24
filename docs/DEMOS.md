# Demos

`docs/media/*.gif` are not hand-made. Each is a recording of `examples/capture` with a different
`--scene`, and this page is the recipe, so a later capture can be compared with it.

## One clip per feature

| file | `--scene` | what it shows |
|---|---|---|
| `village.gif` | `village` | the whole thing: rumor halos, tie lines, counters |
| `mood.gif` | `mood` | mood halos — green pleased, amber neutral, red displeased; size is arousal |
| `opinions.gif` | `opinions` | one opinion bar per villager on **honesty**: green high, red low |
| `needs.gif` | `needs` | **hunger** halos — big green is fed, small red is hungry |
| `ties.gif` | `ties` | the tie graph alone: green friendship, red rivalry |
| `factions.gif` | `factions` | the three faction colours, no lines |

All six are the same village, same seed and the same run: `{"frames": 240, "rumor_reach": 0.520,
"friends": 16, "rivals": 5, "barks": 959}`. Only what is drawn differs, which is the point — a feature
clip is a different *view* of one simulation, not a different simulation.

### The clips were rebuilt in 2026-11-28, and every one of them had been lying

A visual review of the previous set found six separate faults. They are recorded because each has a
shape a later change could reintroduce.

| fault | what it looked like | cause |
|---|---|---|
| villagers inside the buildings | half a body emerging from a cream cube | places were 3 m cubes and villagers spawned **0.6 m** from their centres |
| everyone bunched, never moved | the same clusters in every frame | a place's half-width was **1.5 m** and `interaction_radius` is **1.5 m** — being "arrived" required standing *inside* the block, so nobody ever walked |
| the same again | no visible walking | `seconds_per_day = 8` made a **3 m walk cost 6.4 game-hours** against a 0.3-hour chat |
| `needs` showed one colour | 24 identical red halos, first frame to last | the need's index was **1**, which is `rest`; `food` is 2 |
| `opinions` showed one colour | all-green bars | topic index **0** is `kindness`; `honesty` is 1, and bars were drawn inside the capsule |
| `village` showed no rumor | counters said 10/24, no halos | the ring read `RumorId(0)`, the first rumor the run made, not the planted one |
| `factions` showed two colours | millers had walked out of shot | fixed camera, and `play` targets `Wander(6.0)` |

Two of these changed the crate rather than the demo. **`Obstacle`** (`src/steering.rs`) is new: the
component that says a thing is solid, fed to ORCA as a hard wall half-plane and backed by a resolver that
pushes a walker out of it. Before it, `steer_movers` passed `&[]` for walls and the crate had no way to
express a building at all. `examples/common/scene.rs` also gives each faction its own `Home`; all
twenty-four shared one, which made every sleep a cross-village walk that timed out — measured, 25
interrupted sleeps against 4 completed.

### Overlay scales are normalised to the measured range, and the sign is the colour

This village's PAD spans about ±0.4 and its opinions cluster within ±0.2 of neutral. A full [-1, 1] scale
maps every villager to the same colour, so mood is normalised to `MOOD_SPAN = 0.45` (measured over 4 200
ticks) and opinion to `OPINION_SPAN = 0.5`.

Opinion bars are drawn **radially on the ground**, with the colour carrying the sign and the length the
magnitude. A bar drawn from inside the capsule at waist height was occluded by the body it belonged to:
11 046 negative bars were drawn per run and the clip rendered as though every villager were positive.

### Overlays draw above the mesh, not at ground level

`MARK_HEIGHT` is 0.06 m. Rings and tie lines drawn there are hidden by the villager capsules and by the
ground plane's depth: measured, the ties clip showed two short segments where the graph has hundreds of
edges. Halos and ties therefore draw at chest or head height. Opinion bars are the exception and are
drawn on the ground, because a bar needs its *length* in the camera's plane.

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

`tools/gif.py` is the other half, and it needs only **Pillow** — no ffmpeg, no ImageMagick. Both were
absent on the machine that last rebuilt the clips, and `tools/gif.sh` failed *after* the frames had been
rendered, twice. Pillow is already present wherever the evidence plots are made.

```
python3 tools/gif.py frames/village docs/media/village.gif \
    "a rumor spreads · ties form · relationships change"
```

Optional positional arguments after the caption: `width` (720), `stride` (1), `fps` (30).

- **One palette for the whole clip**, `256` colours, built by adaptively quantizing the first frame. Two
  wrong versions preceded it: a vertical montage of eight sample frames produced a palette dominated by
  grass and turned every faction grey-green, and 128 colours compressed the amber/teal/blue faction
  capsules into one another. A demo whose subject is colour needs the format's full depth.
- The caption is burned into a 30 px strip below the frame. It is the *only* place the scene's name
  appears: the in-scene HUD carried it as well, which cost a third line of the plate and covered the
  village.
- A fixed palette and fixed dither mean two clips a week apart are comparable; a palette change between
  runs would look like a visual difference no code change caused.

### Framing

The camera is computed per frame from **where the crowd actually is** (`scenes::crowd`): the mean of the
living villagers' positions and their maximum spread, with `reach = spread * 2.7` clamped to
`[14.5, 27.0]` metres of height. A fixed frame lost a faction as soon as it wandered — `play` targets
`Wander(6.0)` — and measured, the `factions` clip showed two colours while claiming three. The ground
plane is 80 m across so its edge never enters shot.

Three earlier framings, all wrong, in the order tried: 7.2 m out at 6.2 m up (two nearest stalls filled
the frame, bottom edge cut people off); 11 m out at 8 m up (the far faction left the top, a third of the
frame was empty grass); 24 m out (everything visible but every villager 8 px tall).

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

**Look at frames 0, 120 and the last one, for every clip, before committing.** `tools/gif.py` writes a
plain full-frame GIF, so any image viewer or `read` can open it directly — no coalescing step needed.

Each frame must show:

- faction colours that tell villagers apart at a glance;
- legible balloons, with no clipped text;
- the HUD (`Day n · hh:mm`, the rumor's reach, friendships and rivalries);
- rumor halos appearing only after frame 30 and spreading;
- the scene's own overlay legible as the thing it names — for `mood`, halos that differ in colour *and*
  size between villagers; for `opinions`, bars that differ in **both** colour and length; for `needs`,
  halos that visibly shrink over the clip; for `ties`, a graph rather than a handful of segments;
- **no villager intersecting a stall**, and nobody standing inside one;
- all three factions in frame at every moment, not just at the start.

Last checked 2026-11-28, all six clips, frames 0 / 120 / 239: HUD plate legible over the grass in
constant daylight (the clips no longer follow the clock — the last third of every one went dark blue and
the feature became unreadable), captions correct, every overlay distinguishing at least two villagers, no
blank first frame, no villager inside a stall. The `factions` clip deliberately carries no tie lines, so its three colours are the only thing
on screen.

Any failure is a scene bug: fix `examples/common/scene.rs` or `visuals.rs` and capture again. Do not
paper over it in the encode.

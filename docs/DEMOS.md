# Demos

`docs/media/*.gif` are not hand-made. Each is a recording of `examples/capture` with a different
`--scene`, and this page is the recipe, so a later capture can be compared with it.

## One clip per feature

| file | `--scene` | what it shows | burned-in caption (the argument to `tools/gif.py`) |
|---|---|---|---|
| `village.gif` | `village` | the whole thing: rumor halos, tie lines, counters | `a rumor spreads · ties form · relationships change` |
| `mood.gif` | `mood` | mood halos — green pleased, amber neutral, red displeased; size is arousal | `mood: green pleased · amber neutral · red displeased · ring size = arousal` |
| `opinions.gif` | `opinions` | one opinion bar per villager on **honesty**: green high, red low | `opinion on honesty: green above the middle, red below` |
| `needs.gif` | `needs` | **hunger** halos — big green is fed, small red is hungry | `food: a full green ring is fed, a small red one is hungry` |
| `ties.gif` | `ties` | the tie graph alone: green friendship lines | `ties: green is friendship · the village forms them faster than it falls out` |
| `factions.gif` | `factions` | the three faction colours, no lines | `faction colour · orange millers, green weavers, violet watch` |
| `goods.gif` | `goods` | a ring per villager sized by their bread (gold holds some, small dark holds none), a white line for each loaf shared, and a HUD line counting loaves shared and who holds bread | `bread: gold ring = loaves held, dark = none · a white line is a loaf shared` |

All seven are the same village, same seed and the same run, **except `goods`**: `{"frames": 240, "rumor_reach": 0.080,
"friends": 24, "rivals": 0, "barks": 637}`, recorded 2026-10-25 against the pack that has `share` and a three-loaf
`bake`. Only what is drawn differs, which is the point — a feature clip is a different *view* of one simulation,
not a different simulation.

**`goods` also opens with bread handed out** (`stock_bakers` in `examples/capture.rs`): every other villager holds
four loaves at frame 0. The settled village's stores sit near zero — a loaf is eaten as fast as it is baked, 0.05 a
person measured over nine seeds — so a clip taken from the settled state would show 24 dark rings and nothing to
see. The model, its rates and every number printed above are untouched; only the recording's starting state
differs, and the README says so. `{"frames": 240, "rumor_reach": 0.040, "friends": 23, "rivals": 0, "barks": 613}`.
In its six game-hours **5 loaves are shared** and the people holding bread go from 12 to 17.

### Which teller, and what the rumor line means

`PlantRumor` is written at frame 30 with a weaver as teller. **Which weaver is chance.** A rumor moves only when its
holder happens to choose `gossip` beside somebody, so inside six game-hours some tellers pass it on and some do
not. Tried on 2026-10-25 against this village, seed and settle:

| villager index | rumor reach after 240 frames |
|---|---|
| 1, 3, 8, 16, 18, 20 | **0.040** — the teller alone, nobody told |
| 5, 10, 12, 14 | **0.080** — one person told |

The first weaver (8) was the old choice and now tells nobody, because the economy changed what the settle did.
The capture uses **10**, the next weaver along. A busier teller picked by neighbour count was tried and told
nobody either, so it was removed. The hero's "2 of 24" is therefore **a choice among ten, four of which work**,
and the README says so.

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

### Rebuilt again on 2026-10-25: two more faults, and numbers that were not true

| fault | what it looked like | cause |
|---|---|---|
| the HUD and the rings described the wrong rumor | at frame 0, before the clip had planted anything, the HUD read `“Bram stole bread” — known by 8/24` with a ring under eight villagers, and the count wandered 8, 9, 8 | `update_hud`, `draw_rumor_rings` and `stats` read the **newest alive rumor**, which is whichever deed the village last made a story of; the settle steps create dozens. A fix to this exact mistake (`RumorId(0)`) had picked a different wrong rumor. They now read `visuals::PlantedRumor`, set when the host writes `PlantRumor` and resolved to the rumor that message created |
| the watch faction was grey in every GIF | violet capsules rendered grey-green | `tools/gif.py` quantized the first frame with median cut, which spends its buckets on the grass: **0 of 847** bluish pixels survived. Fixed in the encoder |
| every printed number was about another rumor | `rumor_reach 0.520`, a threshold met | same cause as the first row. The planted rumor reaches **2 of 25** in the clip, and the README had said twenty-three of twenty-five |

The HUD's rumor line and the rings now appear only after the plant at frame 30 (and, in the interactive
`village` example, after `R`).

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

## Building on big

Compilation for this crate happens on **big** (`ssh big`, Linux x86_64, 24 cores), not on the laptop:

```
./tools/big-cargo.sh test --release --all-features
./tools/big-cargo.sh --no-sync clippy --all-targets --all-features -- -D warnings
```

The wrapper rsyncs the working tree (minus `.git`, `target`, the GIFs and `frames/`) to `~/bs-build` on big, then
runs cargo there with `RUSTUP_TOOLCHAIN=1.98.1` and `CARGO_TARGET_DIR=~/bs-target`, under `nice -n 5` because the
machine is shared. `--no-sync` reuses the last copy, which is what a second command wants. Override with
`BS_REMOTE_DIR`, `BS_REMOTE_TARGET` or `BS_TOOLCHAIN`.

**Capture, `gif.py` and the evidence plots stay on the laptop**: big has no GPU renderer for a Bevy window and no
`ffmpeg`, and a PNG rendered there is not comparable with one rendered here (different GPU). Compilation and tests
move; pixels do not.

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

- Settings: `seconds_per_day 32.0`, `tick_secs 0.1`, `start_hour 15.0`. A frame is 1/30 s, so the clip's
  240 frames are 8 s, a quarter of a game-day, **six game-hours**: 14:00 to 20:00 on the HUD.
- 3 800 unrecorded steps first, so ties have formed before the first captured frame.
- `warm_up(3)`, then 240 recorded frames at 30 fps (8 s). The encode is stride 1, 30 fps; if a clip
  exceeds about 6 MiB, pass `stride 2` and `fps 60` to `tools/gif.py`, which halves the file at the same
  speed. None did on 2026-10-25 (2.4 to 3.2 MiB).
- At recorded frame 30 it writes the same `PlantRumor` the village example's `R` key writes: Bram stole
  bread, told by a weaver.
- The camera orbits from 20° to 50° around the origin at radius 24, height 16, at 720×405.
- It prints one JSON line and exits non-zero if fewer PNGs exist than frames requested:

```json
{"frames": 480, "rumor_reach": 0.0, "friends": 0, "rivals": 0, "barks": 0}
```

The clip is accepted when `rumor_reach ≥ 0.08` (the planted rumor has left its teller), `barks ≥ 6`,
`friends ≥ 3` and `rivals ≥ 1`. If it is not, tune **only** `examples/assets/village.society.ron` (the
gossip, compliment and snub weights) and the capture's `bark_chance`. Never change a crate default to make
a demo look better: the defaults are the model.

### Current result

```
{"frames": 240, "rumor_reach": 0.080, "friends": 24, "rivals": 0, "barks": 637}
{"frames": 480, "rumor_reach": 0.080, "friends": 23, "rivals": 0, "barks": 677}
```

The first line is the clips of 2026-10-25, the second the same village recorded twice as long. Three of the four
thresholds are met (`rumor_reach ≥ 0.08`, `barks ≥ 6`, `friends ≥ 3`); **`rivals ≥ 1` is not**, and has not been
since the clips were rebuilt: a rivalry takes game-days to form and the clip is six game-hours, so no clip draws
one and the README says so. **The rumor threshold is not what it was**: an earlier documentation claimed
`rumor_reach` of 0.520 and 0.360 against a threshold of 0.5, but that number counted the **newest alive rumor**,
which is whichever deed the village last made a story of — the settle steps create dozens — and not the one the
clip plants. Measured against the planted rumor the reach is **2 of 24 at 240 frames and still 2 at 480**. It is
slow because telling is one choice among many and Daley–Kendall stifles a teller whose listener already knew.

The capture settles for 3 800 steps before recording. An earlier clip ran 900 ticks — 4.1 game-days — and
formed no friendships: `TieLabel::classify` needs `affinity ≥ 0.4`, affinity moves at
`mere_exposure_per_day = 0.05` against `affinity_relax_per_day = 0.01`, and four days is not enough.

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

- **One palette for the whole clip**, `256` colours, by octree quantization of a montage of five frames
  (first, quarter, middle, three-quarter, last). Three versions preceded it. A vertical montage of eight
  frames with median cut turned every faction grey-green; 128 colours compressed the amber, green and
  violet capsules into one another; and **median cut on the first frame** kept **0 of 847** of the
  watch faction's bluish pixels (measured against the PNG, in the old
  clips and in the rebuilt ones) because it spent its buckets on the grass gradient, and would have had
  no colour for the rumor ring, which first appears at frame 30. Octree on a five-frame montage keeps
  867 of 847, with the ring, at a mean error of 3.2 against 1.5. A demo whose subject is colour needs
  the format's full depth and a palette that has seen the late frames.
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

## The clip has no rivalries, and the caption says so

**The README said "rivalries red" and no clip has ever drawn one.** Measured: the capture settles 3 800
steps inside a 32 s game-day and records six game-hours, which is far too short for a village to form a
rivalry — and the full 16-day `village-evening` harness does produce them (24 with the reciprocity step,
`became_rival` 271), so this is the window, not the model. Two attempts to fix it by widening the settle
(9 000 and 8 000 steps) both still read **0 rivals**.

The caption is now what the clip shows. When a clip of rivalries is wanted, it needs its own scenario with
a faster clock, not a longer settle in this one.

## Visual check

**Look at frames 0, 120 and the last one, for every clip, before committing.** `tools/gif.py` writes a
plain full-frame GIF, so any image viewer or `read` can open it directly — no coalescing step needed.

Each frame must show:

- faction colours that tell villagers apart at a glance, **in the encoded GIF as well as the PNG**: count the
  bluish pixels of the `watch` faction in `docs/media/<clip>.gif` against the frame (about 650 to 1 250,
  depending on the scene), because the encoder once turned them grey;
- the HUD (`Day n · hh:mm`, friendships and rivalries, and — after frame 30 only — the rumor's reach);
- no rumor line and no halo at frame 0, one halo and `known by 1/24` at frame 120, and the count growing
  from there (`village` and the HUD of every clip);
- the scene's own overlay legible as the thing it names — for `mood`, halos that differ in colour *and*
  size between villagers; for `opinions`, bars that differ in **both** colour and length; for `needs`,
  halos that shrink over the clip; for `ties`, a graph rather than a handful of segments;
- **no villager intersecting a stall**, and nobody standing inside one;
- all three factions in frame at every moment, not just at the start.

`capture` draws **no speech balloons** by design (the dialogue plugin is disabled, so the frames would
never show them); balloons are the interactive `village` example's, so they are not part of this check.

Last checked **2026-10-25**, all seven clips, frames 0 / 120 / 239 of each, against the run
`{"frames": 240, "rumor_reach": 0.080, "friends": 24, "rivals": 0, "barks": 637}` (and `goods`'s own):

| clip | frame 0 | frame 120 | frame 239 | verdict |
|---|---|---|---|---|
| `village` | 26 friendships, 0 rivalries, no rumor line, no halo | `known by 1/24` | `known by 2/24`, two magenta halos | pass: the rumor starts at frame 30 and spreads, slowly |
| `mood` | mostly amber and yellow rings, a few orange-red; sizes differ | same | same | pass, but **subtle**: most villagers are near neutral, so the clip differs by a few rings |
| `opinions` | red bars on two thirds of the villagers, green on the rest, lengths differ | same | same | pass: both colour and length differ |
| `needs` | every ring full and green | same | same | pass, but **very subtle**: nobody is hungry in the six game-hours, so the rings do not visibly shrink |
| `ties` | 26 friendships, 0 rivalries | 26 | 24 | pass: a graph, but a sparse one |
| `factions` | orange, green and violet, no lines | same | same | pass |
| `goods` | 0 shared, 12 hold bread, 12 do not; gold rings and small dark ones | 1 loaf shared, 13 hold bread, 11 do not | 5 shared, 17 hold bread, 7 do not | pass: readable, and the HUD line is what makes it legible, because a share is a half-second white line |

Also seen and left alone: the fourth stall sits mostly below the bottom edge of every frame, and two or
three orange villagers touch that edge at frame 120. Nobody is inside a stall; villagers stand against
their edges, which is what the `Obstacle` resolver leaves. All three factions are in frame at every one of
the three frames.


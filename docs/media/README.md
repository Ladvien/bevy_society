# Media

Every image in this folder is a recording of `examples/capture` on big_mac, encoded with `tools/gif.py`. Nothing here is hand-drawn, and no file is in Git LFS. The recipe for each scene, its camera and its visual review is in [`../DEMOS.md`](../DEMOS.md).

## Regenerating one

```sh
cargo +1.98.1 run --release --example capture -- \
    --font /System/Library/Fonts/Supplemental/Arial.ttf \
    --out frames/<scene> --frames 240 --scene <scene>
python3 tools/gif.py frames/<scene> docs/media/<scene>.gif "<the caption burned into the strip>"
```

`tools/gif.py` needs only Pillow. The ffmpeg variant, `tools/gif.sh`, needs ffmpeg and ImageMagick and is not used here.

## The files

All seven recorded on **2026-10-31** against `bevy_society` **0.9.0** (`fc53528`), 240 frames each at 720×405, 30 fps, 8 s, one palette per clip.

| file | `--scene` | caption burned in | size |
|---|---|---|---|
| `village.gif` | `village` | `a rumor spreads · ties form · relationships change` | 3.2 MiB |
| `village.png` | `village` | a still, frame 120 of the same run: the hero clip's midpoint, used as the README's still image | 235 KiB |
| `mood.gif` | `mood` | `mood: green pleased · amber neutral · red displeased · ring size = arousal` | 3.1 MiB |
| `opinions.gif` | `opinions` | `opinion on honesty: green above the middle, red below` | 2.7 MiB |
| `needs.gif` | `needs` | `food: a full green ring is fed, a small red one is hungry` | 3.1 MiB |
| `ties.gif` | `ties` | `ties: green is friendship · red is rivalry` | 3.2 MiB |
| `factions.gif` | `factions` | `faction colour · orange millers, green weavers, violet watch` | 2.5 MiB |
| `goods.gif` | `goods` | `bread: gold ring = loaves held, dark = none · a white line is a loaf shared` | 3.2 MiB |

The still:

```sh
ffmpeg -i frames/village/frame0120.png -frames:v 1 docs/media/village.png
```

## What the run reported

The capture prints one JSON line per scene. On 2026-10-31 the `village` run reported:

```
{"frames": 240, "rumor_reach": 0.080, "friends": 27, "rivals": 1, "barks": 592}
```

Its HUD reads `Day 4 · 14:01  Friendships 30 · Rivalries 1` on the first frame and `Day 4 · 20:01  Friendships 27 · Rivalries 1` with `“Bram stole bread” — known by 2/24` on the last, so the clip shows a rivalry forming for the first time: the set recorded before `Tuning::forgiveness` (2026-10-30) drew none, and the captions said so. `rivals: 1` meets the `rivals ≥ 1` acceptance threshold in `DEMOS.md` for the first time since the clips were rebuilt.

The `goods` scene is the one whose starting state differs: it hands every other villager four loaves before recording (`stock_bakers` in `examples/capture.rs`), because a settled village's stores sit near zero and a clip from that state would draw 24 empty rings.

## Clips this replaced

The seven files above replace the set recorded on 2026-10-25, whose captions said `Rivalries 0` and whose `ties.gif` caption promised a red rivalry line it never drew. Nothing was deleted from the repository: the old set is in the history at `13ac880`, and the faults a visual review found in earlier rebuilds are recorded in [`../DEMOS.md`](../DEMOS.md) because each has a shape a later change could reintroduce.

# Eval

## Six clips, one run

All six captured with the same seed and spawn; only the overlay and camera differ.

```
village:   {"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}
mood:      {"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}
opinions:  {"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}
needs:     {"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}
ties:      {"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}
factions:  {"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}
```

Identical numbers are the proof that a feature clip is a different *view* of one simulation.

## The three defects the clips exposed

| | before | after |
|---|---|---|
| overlays drawn | invisible (6 cm, occluded by capsules and ground depth) | visible at chest/head height |
| ties clip | **two short segments** | a network |
| hero `rumor_reach` | **0.000** | **0.880** |
| mood rings | identical dull olive | colour *and* radius distinguish villagers |
| opinion columns | invisible (short bar) | one readable column each |

## Files

| file | size |
|---|---|
| `hero.gif` | 2.6 M |
| `mood.gif` | 2.7 M |
| `opinions.gif` | 2.4 M |
| `needs.gif` | 2.6 M |
| `ties.gif` | 2.6 M |
| `factions.gif` | 2.3 M |
| **total** | **15 M** |

Encoded at `WIDTH=600 STRIDE=2 FPS=60`, which halves the file with no loss of speed: `STRIDE=2` keeps
every other frame and doubling `FPS` restores real-time playback.

## Visual check

Frames 0, 120 and 239 of every clip, **coalesced**, inspected:

| clip | frame 0 | frame 120 | frame 239 |
|---|---|---|---|
| `hero` | villagers, tie graph, HUD plate | dusk, rumor rings spreading | night, 23/25 known |
| `mood` | rings, varied size and colour | same | same |
| `opinions` | one green column per villager | same | same |
| `needs` | red halos on hungry villagers | same | same |
| `ties` | green and red graph | same | same |
| `factions` | three colour clusters, no lines | same | same |

No blank first frame. HUD legible over grass and over night sky. Every overlay distinguishes at least
two villagers.

## The extraction trap, recorded

`tools/gif.sh` encodes with `-layers optimize`, which stores later frames as partial rectangles
(`640x311+0+49`). `magick file.gif[60]` then yields a corrupted image — **the extractor, not the GIF**.
`docs/DEMOS.md` now says to `-coalesce` first, because I lost time to this and the next person would
too.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **27** binaries |
| `cargo test --no-default-features` | 27 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `examples/common/scenes.rs` is example code, and every function in it is reached by one of the
six scenes.

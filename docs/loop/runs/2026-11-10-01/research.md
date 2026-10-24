# Research: one clip per feature, and what the clips exposed

## Why this iteration

The user's instruction: *"record GIFs for the README… one GIF per feature… visually inspect them at
different frames."* The crate had one clip (`hero.gif`) covering one scene, and the README said so.
Nine of the crate's features had no picture.

## What the clips exposed in the crate's own demo code

Three defects, all found by looking at frames rather than at code.

### 1. Every overlay was invisible

`MARK_HEIGHT` is 0.06 m, and the tie lines, rumor rings and (new) need rings all drew there. The
villager capsules stand on the ground and the ground plane is a mesh with depth, so at 6 cm the
geometry is occluded: measured, the ties clip showed **two short segments** where the graph has
hundreds of edges. Everything now draws at chest or head height.

### 2. The overlays and the printed stats read `RumorId(0)`

The same mistake the evidence harness made twice and `docs/DEMOS.md` recorded as fixed. It was not:
`draw_rumor_rings`, `stats()` and the HUD's "known by N" line all read the **first** rumor the run
created, which for this clip is a deed from the settle steps, not the theft the caption names.

| | reported reach | actual reach of the planted rumor |
|---|---|---|
| before | **0.000** | 0.880 |
| after | **0.880** | 0.880 |

The hero clip's whole subject — the rumor spreading — was invisible in its own numbers.

### 3. The colour scales were unreadable

PAD in this village spans only ±0.4 (measured over 4 200 ticks: pleasure [-0.387, 0.382], arousal
[-0.361, 0.447], dominance [-0.413, 0.513]). Mapped onto [-1, 1], every villager got the same dull
olive ring. The scale is normalised to `MOOD_SPAN = 0.45`, and the opinion columns likewise to
`OPINION_SPAN = 0.5`, both of which are those measurements rather than guesses.

## The scenes

`examples/capture --scene <name>` selects one of six views of **the same run**: same seed, same
spawn, identical `{"frames": 240, "rumor_reach": 0.880, "friends": 36, "rivals": 45, "barks": 2810}`.
A feature clip is a different *view* of one simulation, not a different simulation — which is what
makes them comparable.

| scene | overlay |
|---|---|
| `village` | everything: rumor rings, tie lines, HUD |
| `mood` | PAD ring, colour = pleasure, radius = arousal |
| `opinions` | one column per villager on the honesty topic |
| `needs` | food halo, radius and colour = how fed |
| `ties` | the tie graph alone |
| `factions` | faction colours, **no** tie lines |

## Literature

No new model. The relevant constraint is the one `docs/DEMOS.md` already states — a demo must not
change the thing it demonstrates — and the new overlays obey it: every one is a read-only `Gizmos`
system. The capture's four thresholds (`rumor_reach ≥ 0.5`, `barks ≥ 6`, `friends ≥ 3`, `rivals ≥ 1`)
are unchanged and met by a wide margin (0.880 / 2810 / 36 / 45).

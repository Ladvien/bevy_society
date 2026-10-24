# Review

## Pass 1 — Behavior: does each clip show the feature it names?

**GREAT**, after seven fixes. Each clip was opened at frames 0, 120 and 239 and read as a stranger would.

| clip | what the frame shows | matches its caption? |
|---|---|---|
| `village` | magenta halos on 10 of 24 villagers, green and red tie lines, counters reading 10/24 | yes — the halo count and the number agree |
| `mood` | halos in green, amber, orange and one red, of visibly different size | yes |
| `opinions` | radial bars, red and green, of different length, from **every** villager | yes — and both colours are present, which took four attempts |
| `needs` | large green halos at frame 0, small red ones by frame 239 | yes — the scale is visible *changing* |
| `ties` | a graph of green and red lines entirely within frame | yes |
| `factions` | orange, green and violet capsules, all three groups in shot | yes |

The two that mattered most: `needs` and `opinions` had been drawing the **wrong index** — `rest` and
`kindness` — while their captions said food and honesty. No amount of camera work would have fixed that.

## Pass 2 — Zoom: read the frames closely

**GREAT.** Checked specifically for the faults this iteration was about:

- **Clipping**: none. Every capsule is beside its stall, not through it. Earlier frames had half-bodies.
- **Motion**: villagers visibly differ in position between frames 0, 120 and 239 — the old clips were
  pixel-identical clusters.
- **Saturation**: `needs` runs the full range green→red across the clip; `mood` shows four distinct colours
  rather than one.
- **Framing**: all three factions in shot at all three sampled frames, ground edge absent.
- **Legibility**: HUD two lines at 13 px, caption burned in below, no plate covering the village.
- **Dead agents**: none — every capsule moves at some point in the clip.

## Pass 3 — Fresh eyes: three separate visual passes

**GREAT**, and this is where the work was. Each pass found faults the previous one missed:

1. First pass on the rebuilt scene: **no clipping**, but the camera was too tight and the HUD stale (`/25`
   against 24 villagers).
2. Second pass: factions readable, but the **millers had walked out of shot** and every mood halo was red.
3. Third pass: crowd-following camera held all three factions, and then the remaining two index bugs and the
   `RumorId(0)` bug surfaced — all three invisible in any single frame, all obvious when comparing frames.

A host developer reading the README now sees, in one sentence each: a rumor spreading between coloured
groups; mood as halo colour and size; opinion as bar colour and length; hunger as a halo that shrinks;
friendship and rivalry as lines; and three factions as three colours.

## Verdict

Three GREAT. Shipped.

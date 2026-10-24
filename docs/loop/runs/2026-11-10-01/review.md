# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does each clip show the feature it claims?

Claim: six clips, one per visualisable feature, each showing that feature and only that feature.

**Evidence.** All six captured from one run and printing identical numbers
(`rumor_reach 0.880, friends 36, rivals 45, barks 2810`), which is what makes them comparable views
rather than different simulations. `docs/DEMOS.md` carries the `--scene` table.

**Critique.** `mood` and `opinions` are overlays on a third-person village view; `ties` and `factions`
are the same view with different overlays. A viewer comparing `ties` and `factions` side by side is
seeing the same camera — that is deliberate (the comparison is the point), but it does mean the six
clips are not six *scenes*, they are six *views*. The README says "same village, same seed, same run —
only the overlay changes", which is honest about it.

**Verdict: GREAT.**

## Pass 2 — Zoom: looking at the frames

**Critique — this is where the work was.** Three defects, none visible in code:

1. **Every overlay drew at 6 cm and was invisible.** Occluded by the capsules and the ground plane's
   depth. The ties clip showed *two segments* where the graph has hundreds of edges.
2. **`RumorId(0)` again** — in `draw_rumor_rings`, in `stats()` and in the HUD line. The hero clip's
   headline number was **0.000** while the actual planted rumor reached 0.880. The clip's entire
   subject was missing from its own statistics.
3. **Colour scales mapped a ±0.4 range onto ±1**, so every mood ring was the same dull olive. Fixed by
   measuring the range (4 200 ticks) and normalising to it.

A fourth was my own: extracting frame 60 from the GIF showed solid green, and I briefly concluded the
GIF was corrupt. It was the extractor — `-layers optimize` stores partial rects, and `magick
file.gif[60]` needs `-coalesce`. That trap is now written down.

**Critique of the remaining weakness.** The `factions` clip and the `ties` clip use the same camera
and the same villagers, differing only by the tie lines. A reader could reasonably ask whether
`factions` needed its own GIF. It earns its place because removing the lines is what makes the three
colours legible — but it is the thinnest of the six.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: would these read to someone who has not seen the crate?

**Critique.** Each GIF carries a caption naming its feature in plain words (`"mood - green pleased, red
displeased, ring size is arousal"`), and the HUD shows `Day · time`, friendships, rivalries and the
rumor's reach. A stranger can look at `needs` and see that some villagers have red rings and some have
green, with the caption saying which is hungry. That is the bar the brief sets: readable, with a reason
for each thing on screen.

**Critique.** The HUD still names the rumor as "Bram stole bread" in every clip, including the ones not
about rumor. Minor, and the alternative — a per-scene HUD — would make the six clips look less like one
simulation.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — six views of one run, identical numbers, one caption each |
| Zoom | GREAT — three real demo defects found by looking, including reach reading 0.000 |
| Fresh eyes | GREAT — captions in plain words, HUD legible over grass and night sky |

## Top 5 remaining

1. **`Place`/`Person` completion asymmetry** — 78 of 108 `eat`s interrupted against 0 of 152 socials.
   This is what starves `food`, `fun` and `rest`, and it is now the top model item.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
   `needs-decision/retiring-untellable-rumors.md`.
3. **The two-clocks decision** — warned about and measured, but the API question is still open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`; an arbitrary villager cannot be
   the town crier.
5. **`need-cycle`'s two remaining needs** — `social` and `rest` never cross 0.35 at the shipped rates.

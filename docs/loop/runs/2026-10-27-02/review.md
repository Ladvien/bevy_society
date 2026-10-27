# Review — a rivalry held through a band

Three passes, each written before moving on. The change is one clause in `TieLabel::classify_from` and one
`Tuning` field, defaulted by a three-seed sweep.

## Pass 1 — Behaviour: does the change do what the paper says, at the right time scale?

**The defect was measured before the fix, and in both directions.** `tests/zz_scratch_rival_spells.rs` (deleted
after the run) recorded, at `village-evening` settings over 2.93 game-days and three seeds, that a `Rival` was
released at a margin of **0.0001–0.0002** above its bar with a median spell of **0.14 / 0.29 / 0.38 game-days**,
and that **none of 149** released spells ended for want of familiarity. Re-run at the shipped band: median spell
**2.637 game-days**, release margin **0.1002**. That is the mechanism doing what it claims, measured at the level
the label lives at (pairs), not inferred from a village total.

**The mechanism tests bite in both directions.** `the_rival_bar_has_a_band_and_a_zero_band_is_a_single_line`
asserts a `Rival` inside the band stays one, a `Rival` past it becomes an `Acquaintance`, and `rival_band = 0.0`
restores the single bar exactly. `a_rivalry_is_held_through_a_shallow_rise_and_lasts_longer` runs the village and
asserts the release count **falls**, the median spell **lengthens**, **a release still happens** (the band is not
permanence), and the **friend count is unchanged**. All four pass with the clause and the first three fail
without it.

**The paper's two halves are both satisfied.** Baumeister et al. 2001 (DOI `10.1037/1089-2680.5.4.323`) says bad
is *quicker to form* and *more resistant to disconfirmation*, and that good prevails *by superior force of
numbers*. Before this change the crate had the first, inverted the second, and satisfied the third. After it, the
village still holds **25 / 21 / 19 friend pairs against 7 / 1 / 8 rival pairs** — good prevails on count, by 3×,
while a single bad event lasts longer than a single good one. Per-event asymmetry, aggregate dominance: exactly
the paper's shape.

**The time scale is right.** A rival spell of 2.6 game-days in a 17-day run is a relationship a host can narrate
and a player can notice. 0.29 game-days is not — that is under a day, and the village's day is the unit the whole
crate measures in.

**Verdict: GREAT.**

## Pass 2 — Zoom: the plot, and where it could be lying

`evidence/plots/rival-band.png` — three seeds, rivals solid, friends dashed, before and after on one axis each.

- **The friend curves are superimposed exactly.** Not "close": at every seed the red dashed line is hidden behind
  the blue one for the whole run. That is the strongest available visual check that the clause guarded one
  category, and it agrees with the table (25 / 21 / 19 at every band, 0.0 through 0.15).
- **The rival curves hold instead of collapsing.** Before, the red line touches zero and returns repeatedly at
  seeds 7 and 42 — which, now visible, is the defect drawn: a label blinking. After, the blue line steps up and
  stays.
- **No saturation, no clipping.** The rival count reaches 7–8 pairs of 276 possible and is still climbing slowly at
  day 17 at seeds 7 and 42; nothing is pinned at the clamp. The friend line plateaus around 25, which it did
  before this change and is a property of the village, not of this clause.
- **Seed 11 is the honest weak case and is left in the plot.** Its rival line is 1–2 pairs after and 0–1 before,
  and its friend line *falls* from 25 to 19 over the run. The band did not cause that — the before curve falls
  identically — but it means "rivals rise" is a statement about seeds 7 and 42 with a much smaller effect at 11,
  and the plot must be read that way rather than as a uniform shift.

**A defect found while zooming, which is not this change's and was fixed anyway.** The narrative had no
interaction line and no `finished` line at all: the counters drained `InteractionResolved` and `ActivityEnded`
before `drain_narrative` read them. The numbers were unaffected (regenerating after the fix reproduces every
figure exactly), but the file a host developer reads first showed `started snub` with no target and no outcome —
on the iteration about which pairs are rivals. Both messages are now drained once into a `Vec`.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: a different seed and a different population

Regenerated from scratch at **seed 23, 16 people**, both bands, 17.62 game-days:

| | band 0.0 | band 0.10 |
|---|---|---|
| friends | 17 | **17** |
| rivals | 0 | **1** |
| `became_rival` | 10 | **4** |
| `left_rival` | 10 | **2** |
| label messages | 523 | 509 |

The direction holds and the friend count is again identical. **The effect is weaker at 16 people**, and saying so
is part of the verdict: fewer pairs means fewer grudges to hold, and the village's day is short enough that most
pairs never accumulate one. A host running a small society should expect the band to change little; the change is
about what happens to the hostility a village *does* produce, not about producing more of it.

Told to a host developer without reading code:

> *"A rivalry used to end the moment the two people's affinity ticked back above the line — one step of ordinary
> drift, with nothing happening between them. Now, like a friendship, it lasts until something actually improves
> the relationship. Your `TieLabelChanged` messages for rivals drop by about two-thirds, and the rivalries you
> do get last days instead of hours. Nothing about friendships changed; measured at three seeds, the counts are
> identical. `Tuning::rival_band = 0.0` turns it off."*

That is explainable, and it is a behaviour a host can build on.

**Verdict: GREAT**, with the small-population limit stated rather than glossed.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — release margin 0.0002 → 0.1002, median spell 0.29 → 2.64 game-days, both directions tested |
| Zoom | GREAT — friend curves superimposed at three seeds, rival blink visible before and gone after; harness narrative fixed |
| Fresh eyes | GREAT — seed 23 / 16 people holds the direction with identical friends, and the weaker effect is recorded |

## What this iteration records as still wrong

1. **`patient_reaction` still moves no affinity** — a victim keeps liking their insulter. Now the clearest gap
   left in the reputation chain: with rivalries persisting, the *reason* the first one formed is still
   one-sided. Needs a cited rate or an explicit design-default decision.
2. **Hostility saturation** in a crowd with no positive activity — `run 2026-10-27-01` measured affinity pinning
   at −1.0. Unchanged by this iteration; the band holds grudges that already exist.
3. **The clock decision** (`travel_timeout_hours` in game hours against real-time walking) — still the largest
   host-ergonomics tax and still open in `needs-decision/`.
4. **`examples/benchmark` builds an unsittable home** for every population, so every perf number in the repo is
   the cost of a society that cannot sleep. Recorded twice now, fixed by nobody.
5. **Seed 11's village has almost no hostility at all** (0 rivals before, 1 after). Not investigated; the crate
   can produce a village with no hostility and nothing warns.

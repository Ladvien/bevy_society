# Eval — a victim is moved by what was done to them

Same pack (`village-evening`), same seeds, 67 200 ticks (17.0 game-days), 24 people. `evidence/before/` is the tree at
`417be32`; `evidence/after/` is this change. `sweep_0.0_s7` is `negativity_ratio = 0.0`, which reproduces *before* exactly
(identical friends/rivals/became_rival to `before/s7`), so it is the before-state for the counters `before/` did not have.

| seed 7 | ratio 0.0 (before) | 1.0 | **5.0 (shipped)** |
|---|---|---|---|
| victims who dislike their insulter (of 390 accepted insults) | 66 | 151 | **350** |
| friends at end | 27 | 17 | **16** |
| rivals at end | 7 | 13 | **27** |
| became_rival | 27 | 40 | **121** |

| | friends before → after | rivals before → after |
|---|---|---|
| seed 7 | 27 → 16 | 7 → 27 |
| seed 11 | 20 → 23 | 1 → 35 |
| seed 42 | 21 → 19 | 8 → 40 |

The defect is closed: 66 of 390 victims disliked their insulter before; 350 do now. The monotone shape in the ratio is
the check that the field is read.

**The cost is large and is recorded, not hidden.** Rivals rise 4–35×, and at seeds 7 and 42 rivals outnumber friends. The
cause is not the ratio alone: ratio 1.0 (symmetric, uncited) already halves seed 7's friends. The village's pack has
accepted insults to accepted compliments at 390 : 1 136 (about 1 : 2.9). Baumeister et al. 2001 reporting Gottman: a
relationship needs positive:negative of at least 5:1 to survive. The village is below the threshold the paper names, and
the model now says so. Whether the pack should insult this often is a pack decision: `needs-decision/a-village-that-is-never-pleased.md`
and the hostility-saturation item in the log already track it.

# Review: a task is not a skill

## Pass 1 — Behaviour: does the change do what the paper and the scenario say?

**GREAT, for the claim made.** Acemoglu & Autor separate a **task** ("a unit of work activity that produces
output") from a **skill** ("a worker's endowment of capabilities"), and say the allocation of one to the
other is endogenous. The crate now models both and matches them by score rather than by gate:
`tests/profession.rs::the_skill_decides_who_does_the_task` shows 5 / 4 / **0** craft starts across
conscientiousness 0.95 / 0.50 / 0.05, and `a_low_skill_person_still_takes_the_task` shows the weakest
villager still working when there is nothing else — comparative advantage, not assignment.

## Pass 2 — Zoom: is the mechanic in the right place, and what did it cost?

**GREAT, and this pass stopped a bigger change.** Zooming into the pack showed that the obvious application
— putting the skill on the village's *dominant* activity — is destructive:

| seed | `work` gated on the skill | additive `craft` task instead |
|---|---|---|
| 42 | rivals 29 → **55**, reach 0.50 → **0.08** | rivals 29 → 33, reach 0.50 → 0.08 |
| 11 | rivals 30 → 37, reach 0.67 → 0.13 | rivals 30 → **27**, reach 0.67 → 0.04 |

The first column is a village whose whole day depends on a trait; the gossip network that rode on the
workplace fragments. The mechanic ships additive, where it competes for hours instead of replacing the day —
and the cost is still visible (seed 11's reach falls), which `eval.md` states rather than hides.

**The first measurement of the mechanism was wrong and would have passed a broken feature.** One activity,
three villagers, 10 / 10 / 10 starts — the skill did nothing, because with a single option the score never
decides anything. The rival task fixes the fixture, and the test file carries that lesson in a comment so the
next person does not delete `potter` as noise.

**A second wrong measurement, in the clips.** `Camera3d::default()` is a 45° vertical field, so the half-height
is `tan(22.5°) = 0.414`. Projecting the four stalls through the camera puts `workshop` at **|y| = 0.66** and
`market` at 0.49 at a 12 m camera distance — both outside the frame. A small village always hits the distance
*floor*, so the floor was cropping. The −2.2 m tilt bias added earlier was equally wrong in the other
direction (`market` at +0.385). The floor is 20 m, where every stall sits inside |y| ≤ 0.37.

## Pass 3 — Fresh eyes: different seed, and is it explainable?

**GREAT.** Three seeds measured for both the gated and additive forms. The narrative is one sentence a host
can follow without reading code: *a villager good with their hands does the craft work, and a villager who
is not does something else.* Determinism is pinned, and the skill is a pure function of a trait, so there is
no state to diverge.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**, after two measured rejections and one wrong probe. Pass 3 **GREAT**.

## What a stranger would still complain about, correctly

- A skill never improves, and `Profession` is a label rather than a mechanism. Both are in `eval.md` and in
  the paper note's "what the model leaves out".
- There is no economy. The design exists in `needs-decision/`, not in the crate.
- Seed 11's rumor reach falls with the additive task. Recorded, not explained away.

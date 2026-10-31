# Review — body clocks

Evidence cited is committed: `evidence/chronotype.md`, `evidence/plots/{sleep,day}.png`, `evidence/{before,after,variants}/<run>/{sleep.txt,summary.json,narrative.txt}`, `evidence/narratives/`. The numbers are in `eval.md`.

## Pass 1 — Behavior: does the change do what the paper and the scenario say, at the intended time scale?

**Critique.** The paper's claim is that mid-sleep is near-normal across a population and independent of how long people sleep (Fischer et al. 2017), so a person's offset should move their night and leave its length alone. Three things say the model does that.

- *By construction and by test:* `tests/chronotype.rs::a_lark_a_plain_person_and_an_owl_go_to_bed_by_their_own_clocks` puts a -3 h, a 0 and a +3 h person in one bed from 18:00 and each lies down within an hour after the wall hour its own clock opens (19:00, 22:00, 01:00); the person with no offset goes at 22:00, the window the pack always had. The sampler is a normal of the requested spread, cut at three deviations, deterministic.
- *In a run:* Hilda (-2.30 h) lies down at 19:44, 19:50, 19:57, 20:09 (`narratives/lark-hilda-s7-6days.txt`; one 21:16 and one 21:42 where a `work` was still running), Quill (+2.34 h) at 00:24, 00:20, 00:34, 00:45, 00:22, 00:44 (`owl-quill-s7-6days.txt`). The offsets are `chronotypes.txt`'s.
- *At the scale a host sees:* on the library's own 240-second day the village's bedtime goes from a cliff (75 % of sleeps in the 22:00 hour, 4 clock hours with 10-90 % of the village asleep) to a ramp (19 %, 12 hours): `plots/sleep.png` first column, `plots/day.png`.

The second claim is the guard: a sleeper cannot be greeted. 349 / 378 / 324 / 362 exchanges of ~4 400 were aimed at a sleeper in the shipped village (a defect nobody could see while everybody slept at once); 0 / 1 / 2 / 3 now. With offsets and no guard it doubles (12-17 %), so the guard is a precondition, not an extra.

One prediction was wrong and is struck (`research.md`, `eval.md` section 3): interrupted sleeps do not fall. All of them end before the person reaches the bed (6 runs, 0 in bed); the walk is the cause. The change does not pretend to fix it, and in two of three towns the rate is a few points worse, unexplained.

**Verdict: GREAT** for what it claims (bedtimes follow the offset, sleepers are not greeted, no offset is bit-identical to 0.8.0 once the guard is off). The unexplained part is recorded, not hidden.

## Pass 2 — Zoom: oscillation, saturation, clipping, drift, convergence, dead agents, spikes at the tick boundary

**Critique.** Read at the figures' own resolution.

- *Bedtime histogram (`sleep.png`, bottom row).* After, the 24-person village's sleeps begin 19:00-03:00 with no tick-boundary spike; the busiest hours are 22-00, not 22:00 alone. The histogram is **not bell-shaped about 22:00**: the 00:00 hour (86 at seed 7, 93 at seed 99) is higher than the 22:00 hour (77, 50). Cause: a person re-decides only when the previous activity ends, so a window that opens at 22 + offset is entered a fraction of an hour late, and the whole histogram is a normal of 1.5 h convolved with that lag (about +0.7 h). Not clipping: the truncation at ±4.5 h is never reached, and nothing is pinned at 0 or 24.
- *Asleep curve (`sleep.png`, top row).* After: the 240-second curve rises 19:00 → 03:00 and falls 04:00 → 12:00 with no oscillation; a residue of 3-5 % asleep at 10:00-12:00 is the latest owls (+2.3 h bedtimes at 00:20 sleep to 08:20 and are then walking). The 64 s/day curve is nearly the same as before, which is the walk smearing it, and is said so in `eval.md`.
- *What the village does (`day.png`).* Work stays on the social clock (the pink band is the same shape before and after), so larks are awake before the workshop opens and owls rise into it: the second clock does what it should. The morning's social activity starts at 04:00 and spreads to 09:00 where it used to open in one step at 06:00.
- *Narratives.* No dead agents; no lark-with-nobody-to-talk-to problem in a village of 24; a lark who wakes at 04:00 eats and talks to the other larks.
- *Something in the zoom that is not this change's, recorded:* the owl's `gossip with Garet (refused)` four times between 22:43 and 23:37, and Hilda's six `compliment with Bram` between 07:11 and 07:48. Re-approaching a person who has just refused, and praising the same neighbour six times in an hour, are the partner-repetition defect (34 % of exchanges repeat the previous partner) that `friendship-has-no-diminishing-return.md` and pass 3 of 2026-10-30 have already worked at. Listed in the log.
- *Ties.* Friend ties go up in all seven pairs (41 → 79 and 6 others), rival ties fall in five of seven, insults do not rise; friend ties stay at 7-15 % of the possible pairs in a village and 3-4 % in a 60-person town, far from the clique `friendship-has-no-diminishing-return.md` describes. Most of the move is the guard, not the clocks (no-clocks column: 41 / 43 / 37 / 42 → 48 / 66 / 51 / 69). The mechanism is not established; it is one seed's trajectory per cell.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: a different seed and population, and can a host developer be told what this does without the code?

**Critique.** Regenerated from scratch at seed 99 (24 people), seed 99 (60 people, 30 days) and seed 3 (40 people, 20 days): the busiest bedtime hour is 64 % → 21 %, 47 % → 17 % and 48 % → 20 %; exchanges aimed at a sleeper 362 → 3, 1 620 → 118 and 692 → 11; friend ties 42 → 57, 89 → 111 and 69 → 79. The town's rival ties go the wrong way at two of its three seeds (116 → 123, 50 → 72); the village's go the right way at all four. Not a regression the pins catch, and not explained.

The explanation a host developer needs fits in four sentences, and `docs/guide/authoring.md` ("Two clocks") says them: a person has a `Chronotype` (hours behind the village's clock; you sample or author it, the crate draws nothing); write what the body decides (sleep) on `BodyHour` and what the village decides (the workshop opens) on `Hour`; an activity marked `unavailable` takes its person out of everyone's reach once they arrive; with no offset a person keeps the village's clock. The honest caveats are in the same note: the interrupted-sleep rate is the walk, not this, and the pack's change (sleepers unreachable) alters the shipped society.

**Verdict: GREAT.**

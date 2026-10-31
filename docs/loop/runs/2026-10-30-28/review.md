# Review: `Input::Stock` and the provisioning overlay

Evidence cited is committed: `evidence/provisioning.md`, `evidence/plots/provisioning.png`,
`evidence/plots/zoom-town-first-days.png`, `evidence/{before,after}/*/{narrative.txt,summary.json,activities.txt}`,
`evidence/diagnosis/`. Numbers are from those files unless marked.

## 1. Behavior

**Critique.** The change claims one thing: production that answers to the household's own shelf is a balancing loop on a stock,
and a clock-driven one is not, so the first scales with the population and the second does not. Read against the scenario the
loop recorded ("the 60-person town starves for good"):

- The supply collapse is gone at every size tried. Bread a head over the last six game-days: town of 60, seed 7, 0.00 → 1.16; seed 99,
  0.05 → 1.10; town of 40 (fresh seed), 0.58 → 1.29; town of 60 at the library's own 240-second day, 0.19 → 1.40
  (`provisioning.md`). Bakes a day answer to the shelves: 4.2 and 5.3 in the two towns against 0.0 and 0.3, and the price rests
  at 0.50-0.66 instead of sitting at 1.7-2.0 against the 2.0 ceiling (second half of the runs).
- The mechanism is the one the research names, and the two halves are pinned apart from the town: a hungry household with a bare
  shelf bakes at noon, where the dawn window is shut (clock-driven pack: 0 loaves for four hours; overlay: 48 after the first
  hour), and a household with bread in hand makes none (clock-driven pack's bread rises 36 → 72 on the dawn bake of full
  shelves; the overlay's only falls, 36 → 21), `tests/provisioning.rs`.
- The overlay also stops people walking to tables with nothing on them: in the 30-day town `eat` is *started* 1 847 times against 3 307
  on the default pack, while the floor falls from 100 % to 38 %.
- A stranger's reading of one villager's evening (`after/t7/narrative.txt`, d13): "18:07 snub, 18:14 gossip, 18:32 started bake,
  19:49 finished bake, started eat, 20:34 finished eat, started share (to Fenna, accepted 22:51), 23:03 sleep". A hungry person
  with nothing made bread, ate it, gave a loaf on, and slept; every step has the reason the pack gives it. On the default pack the
  same villager's narrative contains no `bake` line at all (0 lines).
- The default pack is untouched: a three-day run of this branch's harness and HEAD's gave byte-identical `series.csv`.

What it does not do, stated plainly: the overlay town still has 37 % of person-ticks at the floor on `food` with 1.1 loaves a head.
That is hunger competing with everything else by score, not the supply, and it is the next backlog item. The village is not
made worse (floor 19.4 / 21.7 / 17.7 / 21.9 % → 15.8 / 17.9 / 16.7 / 17.4 %). 38 % of the overlay town's bakes are interrupted
(50 of 130 at the 64-second day: a walk of 4+ hours times out), a cost of the harness clock that the 240-second run shares in
smaller measure. The starter pack does not take the change (the block, below).

**Verdict: GREAT** — for the claim made (supply), with the residual named above as a separate defect, not hidden in this one.

## 2. Zoom

**Critique.** `zoom-town-first-days.png` (the first eight game-days, tick by tick, three runs) and `provisioning.png`.

- **Lockstep at hour 24.** Every run, before and after, jumps from 0 to 36 villagers at the floor within a few ticks (at 24 h) and to
  38 in the fresh seed: everyone starts at `food` 1.0 and falls at the same rate. It is the heterogeneous-start item of pass 27
  and is not caused or cured here; both sides carry it identically. Listed, not fixed.
- **The two sides are the same for three days, then diverge.** The bread curves track until about hour 72, the first overlay
  bakes; then the overlay steps up to 1.1-1.3 and stays, while the default pack's falls linearly (0.22 and 0.15 at hour 192).
  A fix that diverges from its cause at the time the cause starts to matter reads right.
- **No oscillation at the scale of days in what matters.** Bakes a day are bursty (0-11 in the town, `provisioning.png` bottom
  row), because households are hungry at similar hours, but bread a head wobbles by about ±0.15 around 1.1-1.2 and the floor share
  by about ±10 points (25-48 % of the town). No growing amplitude over 30 days.
- **Clipping.** In the second half of each run some household sits at the 6-loaf ceiling on 2-11 % of ticks (town seeds 7 / 99) and
  at some tick someone holds nothing; the default village has someone at the ceiling 23 % of the time. The overlay does not
  hoard to the ceiling, which is the stock-driven part doing its job. The default town's mean sits below 0.05 on 100 % of the second
  half at seed 7.
- **The price does not pin.** 0.50-0.66 on the overlay against 1.99-2.00 (seed 7) and 1.68-2.00 (seed 99) on the default town.
- **Bake hours.** The hours bakes are chosen at (overlay, town) are spread across 07:00-19:00 (5-14 each), not a dawn herd.
  Nothing happens at 20:00-06:00 by construction of the window. A bake started in the last hour of the window (the narrative
  shows 19:57, timed out at 23:57) has a walk of four hours or more ahead of it at this clock; the interrupted bakes are the
  walk's time-out, which the harness's 64-second day makes long (not isolated from the window).
- **Villages.** The four village seeds do not move beyond their own swing (floor, 15.8-17.9 % against 17.7-21.9 %); bread is flat
  at 1.4-1.5 where the default's wanders from 1.0 to 2.8.

**Verdict: GREAT** — nothing saturates, drifts or oscillates in a growing way; the lockstep spike and the 38 % interrupted bakes
are existing or clock-bound effects recorded as such.

## 3. Fresh eyes

**Critique.** Regenerated from scratch with a population and seed nobody tuned against: **town of 40, seed 3, twenty game-days**
(`evidence/{before,after}/t40s3`). Default pack: bread 0.58 a head at the end and falling, 49 % of person-ticks at the floor over
the last six days (the second half of the run has 3 % of ticks with someone at the ceiling, 0 % with a mean under 0.05: it is
just entering the collapse). Overlay: bread 1.29, floor 22.6 %, bakes 3.3 a day, price 0.50, friends / rivals 69 / 50 → 70 / 54,
insults 555 → 521. The narrative (`after/t40s3/narrative.txt`, d19-d20): "16:13 started bake, 19:28 finished bake (completed), 20:32
an insult approached by Ivo (refused), 23:28 sleep; next day 08:55 started eat" reads without the code: a person with a bare shelf
made bread in the afternoon and ate in the morning.

Explained to a host developer without the source: *a household that is hungry and has no bread makes some; one that has bread does
not; nobody goes to a table that is empty.* Three terms in a `bake` consideration list say it (`Stock`, `Need`, `Hour`), and
`docs/guide/authoring.md` has the block to copy. A different seed and a different population show the same shape as the tuned one,
at a smaller size and a milder collapse, which is what a loop on a stock should do.

**Verdict: GREAT.**

## The block (not a review verdict)

The overlay falsifies the precondition of `tests/starting_larder.rs::a_starting_larder_removes_the_second_days_famine` (`empty >
0.2`: 0.057 on the overlay), and the starter pack is therefore unchanged; the decision and the patch are recorded for the owner
(`docs/loop/needs-decision/households-provision-themselves.md`, `provisioning-default.patch`). That is a decision about a default,
not a defect of the change.

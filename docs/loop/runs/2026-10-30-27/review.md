# Review: a starting larder (`GoodDef::initial`, starter pack bread `1.0`)

Evidence cited is committed: `evidence/floor.md`, `evidence/summary.md`, `evidence/long.md`,
`evidence/plots/larder.png`, `evidence/plots/zoom-first-days.png`, `evidence/{before,after}/*/{summary.json,narrative.txt}`,
`eval.md`, `perf.md`. Sentences marked `[INFERENCE]` are explanations I did not test.

## 1. Behavior — GREAT

**Critique.** The claim is narrow: a society that is created with every larder empty has a start-up famine the running
society does not, and starting each person with a loaf removes it. `research.md` predicted the signature (day 1 fed from
the larder, day 2's floor share near the settled share, `initial: 0.0` identical to HEAD) and the runs show it.

- Day-2 share of villager-ticks with `food` below 0.05 at the evidence harness's 64-second day (`evidence/floor.md`):
  village s7 / s42 / s11 / s99 **78.6 / 88.9 / 89.6 / 90.0 % → 12.6 / 15.6 / 9.3 / 14.7 %**. At the library's 240-second
  day (`tests/starting_larder.rs`): **38.7 % → 4.7 %**, and the paired test fails the day the larder is removed.
- The settled share is not made worse (last column of `evidence/floor.md`: 19.5 / 23.1 / 35.0 / 23.2 → 19.7 / 24.2 /
  16.7 / 22.9 %), so the change moves the first days and leaves the economy it starts on alone, which is what "start in
  a state typical of the running system" (Murray & Kelton 1988) should do.
- A loaf is exactly `trade_reserve`, so no giver holds a surplus at the start: no `share` is conjured by the larder
  itself (the `share` rows of `evidence/summary.md` move by seed noise, 200 → 166, 100 → 138, 104 → 159, 119 → 163).
- **What it cost.** At day 17 every village seed has more rival ties and fewer friend ties (`evidence/summary.md`:
  rivals 28 / 24 / 18 / 36 → 36 / 36 / 35 / 38, friends 62 / 65 / 47 / 54 → 41 / 43 / 37 / 42) and 9-21 % more insults.
  I treated that as a possible regression and ran two seeds for 40 game-days (`evidence/long.md`). Over days 20-40 the
  fed village holds about 4-8 fewer friend ties and 3-6 more rival ties (about 10 %), inside the ±10 swing of either
  trajectory, and the day-17 gap (rivals +2 to +17, friends −10 to −22) is larger than that residual. Rivalry starts
  earlier at two seeds (end of day 4: s11 3 → 24 rival ties, s42 7 → 24) and not at the others (s7 15 → 9).
  `[INFERENCE]` a fed village spends the first days talking instead of walking to empty tables; not tested. It is
  recorded in the log and the changelog, and it is two seeds.
- The town is **not** fixed and the change does not claim it: see the Zoom pass.

## 2. Zoom — GREAT, with a pre-existing failure it does not touch

**Critique.** `evidence/plots/zoom-first-days.png` is the first four game-days tick by tick. Before: at hour ~26 every
villager (22-24 of 24) is at the floor in one step, and 18-22 are still there at hour 72 at s11 and s99. After: the
same step is 9-13 villagers, the count is 2-7 within about six hours of it, and it stays there, with a second, smaller
rise at hours 60-70 (s7: 9, s11: 13). `[INFERENCE]` the step is the villagers who have not finished a meal by then: a
walk of 8-16 m takes two to four game-hours at a 64-second day (the harness's own warning: "a travel_timeout_hours of 4
at seconds_per_day 64 buys 14.9 metres"), so meals begun at first light end late. Bread a head starts at 1.00 and falls
to 0.7-0.8 before the first dawn bake lifts it, instead of sitting at 0.00 for 18-42 hours. There is no oscillation,
no clipping at 1.0, no spike at a tick boundary.

What the zoom also shows, and `evidence/plots/larder.png` confirms at 30 days: **the 60-person town goes to 100 % of
villager-ticks at the floor with no bread anywhere by about game-day 17, before and after** (price 2.000, `bread_mean`
0.000 in `evidence/*/t7/summary.json`). Those are dead agents. They are the same on both sides: the larder only delays
the collapse (a loaf a head is eaten down over ten days with 0-2 bakes a day), so this change neither causes nor cures
them. Not a reason to withhold the verdict on the larder; it is the reason the next pass exists (`eval.md`, "The town's
famine").

## 3. Fresh eyes — GREAT

**Critique.** Regenerated from scratch at a seed none of the earlier tuning used (s99, both the village and a 60-person
town) in `evidence/{before,after}/{v99,t99}`. The village: day-2 floor 90.0 → 14.7 % and the settled share unchanged
(23.2 → 22.9 %); the town: 83.3 → 38.1 % on day 2, then the same collapse as seed 7.

Read as a stranger, the tracked villager's second day (`evidence/after/v99/narrative.txt`, lines 36-110) is a day: woke at
07:07, talked and played through the morning, **ate at 10:03** (`started eat`, `finished eat` at 11:43: a walk and a
seat), worked, tried to gossip with Nera three times in the evening, went to bed at 22:02. Before
(`evidence/before/v99/narrative.txt`) the same person starts `eat` **five times on day 2** (09:03, 15:41, 18:42, 21:03,
21:48) in a village whose bread a head was about 0.17 (`evidence/plots/zoom-first-days.png`, s99): a reason for each
step (hungry), most of which could not work. A host developer can explain the after in one sentence: everyone begins
with a loaf, eats it at the first meal, and the dawn bake refills the shelves; and the before in another: nothing was
baked for the first 22 hours, so everyone starved on day 2. Both are in `docs/guide/authoring.md` and
`docs/loop/needs-decision/a-village-begins-with-empty-larders.md`.

# Review - a week

Evidence cited is committed under `evidence/`: `week.md` and `plots/week.png` (seven runs), `week-fresh.md` and `plots/week-fresh.png` (two runs, other seeds and size, `start_weekday` 3), `attempt1/` and `sweep/` (the versions that failed and the weight sweep), `narratives/bram-s7-{before,after}-workday-and-restday.txt`.

## 1. Behavior: does it do what the sources and the scenario say, at the intended time scale?

**Critique.** Two claims have to hold. (a) The primitive: an activity gated on `Days(0, 5)` is never started on days 5 and 6, a wrapping window and a mid-day edge land where they should, a body clock does not move the week, a script's `ctx.weekday()` agrees with the curve, and a bad definition is refused at load naming the activity. `tests/week.rs` (9 tests) and the curve unit tests check each on every day of two weeks, from a hand-computed calendar and not from `SocietyClock`; the `f32` edge at midnight is checked for all seven days. (b) The overlay shows what Sekara et al. describe: on a rest day people are more stationary and spend it with fewer, closer people. The first overlay did **not** (struck, `attempt1/`: 27-30 approaches a villager on a rest day against 10, friend ties halved, rival ties up 1.8-3×) and the second did not either (`sweep/fun-*`); the third does, in seven of seven runs and in the two fresh ones: 7.5 approaches a villager on a rest day against 10.7 on a workday, 3.6 different people against 4.7, 44 % of approaches to a friend against 37 %, 36.5 % of the day at leisure, work exactly 0 (`week.md`). The time scale is right: the rest day is one game-day of 4 096 ticks and the effect is read over 28 full game-days, four of each weekday.

**What it costs.** The `social` need is 0.04-0.08 lower on a rest day in every run (a quiet day is under-social), and friend ties in the villages are mixed (3 of 5 lower). The overlay's comment and the research note say both.

**Verdict: GREAT.** The mechanism is tested and the overlay does what its source says; the two failures that led here are recorded where a host will read them.

## 2. Zoom: the plots, closely

**Critique.** `plots/week.png`, the six panels, thin lines per run. Looking for oscillation, saturation, clipping, drift, spikes at a boundary, dead agents.

- **No saturation or clipping.** Laze is 33-40 % of a rest day, not 100 %; the lowest `social` need on any weekday of any run is about 0.67 and the highest `fun` 0.85; nothing sits at 0 or 1.
- **No dead agents.** Work is zero on days 5 and 6 by construction and 13-20 % on the other days (above the no-week figure in every run, because the work moved to five days; the towns of 60 and 40 are lower than the villages with or without a week); sleep is 27-33 % in every run's workday and rest-day rows. Nobody stops doing the rest of their life.
- **A spike at the week boundary: Monday.** Approaches a villager are 12.6 on weekday 0 against 10.1-10.5 on weekdays 1-4 (10.3 on a Monday with no week) and work is 19 % on Monday against 16-17 %. It repeats in the fresh two (12.1) with `start_weekday` 3, so it is not a game-day-0 artifact. My reading: the `social` need ran down over the quiet rest day (0.72, 0.69) and is spent on the first day with company again; I have not isolated it and have not tuned it. It is a transient of one day that returns to baseline on Tuesday, and it is what a week of two quiet days looks like in a need-driven model.
- **Day 5 is quieter than day 6** (6.6 against 8.3 approaches): the first rest day is the idler. Not explained; both are well below a workday.
- **The runs agree on direction** in every rest-day column (the per-run sign test in `eval.md` section 2: approaches, different people and `social` lower, friend share higher, seven of seven); the friend-share rise is small in the towns of 60 (1.4-1.8 points) and large in the villages and the town of 40 (4.6-13.0).

**Verdict: GREAT**, with Monday recorded in `eval.md` section 5 as a finding and not a defect.

## 3. Fresh eyes: another seed, another size, another calendar; can it be explained without the code?

**Critique.** `evidence/fresh/`: a village of 24 at seed 123 and a town of 50 at seed 5, `start_weekday` 3 (so the rest days fall on game-days 2-3 of each seven, which the ledger finds on weekdays 5 and 6), neither used to choose the weight. Result: approaches 8.5 and 6.9 on a rest day against 10.4 and 10.2, different people 3.35 and 3.52 against 4.01 and 5.21, friend share 46.0 % and 39.9 % against 34.5 % and 33.2 %: the same signs and sizes as the seven. Rival ties 26 → 24 and 101 → 47; insults 412 → 461 in the village (up) and 957 → 768 in the town. The village's friend ties fall 74 → 39, as they do in three of the first five villages.

**Read as a stranger.** `narratives/bram-s7-after-workday-and-restday.txt`: on the workday (d3) Bram talks to people from the morning, in two bursts; on the rest day (d6) he complimented Dena, chatted with Hilda, lazed at 09:06 and 11:06, ate at 13:06, lazed at 15:35, 17:35 and 19:47, took two chats and a gossip from people who came by, and slept at 22:00. That reads as a day off. Two things a stranger would question: the same person is complimented four times across the two days (the known repetition of accepted approaches, `log.md` 2026-10-31-03 item 2, not this change), and an insult arrives at 22:14, after he has gone to bed, because a person still walking to bed can be approached.

**Can a host developer be told it without the code?** Yes, in four sentences: `Input::Week` and `Days(from, to)` are `Hour` and `Hours` for the week; `start_weekday` says which day game-day 0 is; shutting `work` on rest days is not enough because the hours go to company in this pack, so the overlay adds a `laze` whose `weight` says how idle a rest day is; the starter pack is unchanged.

**Verdict: GREAT.**

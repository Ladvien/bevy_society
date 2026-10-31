# A week: some days are rest days

`src/model/curves.rs` (`Curve::Days`, `day_window`, `DAYS_PER_WEEK`), `Input::Week` / `Signal::Week` (`src/defs.rs`, `src/behavior.rs`), `SocietySettings::start_weekday` and `SocietyClock::weekday` (`src/core.rs`), `ctx.weekday()` (`src/script.rs`, `src/snapshot.rs`), the opt-in overlay `examples/assets/weekly.society.ron`. Run notes with the evidence: `docs/loop/runs/2026-10-31-04/`.

## The problem

The crate had an hour of the day and a count of game-days and nothing between, so every day of every society was the same day: the workshop opened at 08:00 on all of them and nobody had a rest. A pack that wanted a rest day could only write `Input::Script("ctx.day() % 7 ...")`, a script evaluation per consideration per person per tick, unvalidated at load, and invisible to an editor that shows `Hours(8, 17)` as a window.

## Sources

- **Sekara, Stopczynski & Lehmann 2016**, *Fundamental structures of dynamic social networks*, PNAS 113(36), DOI `10.1073/pnas.1602803113` (supplement, predictability by weekday; the paper is already cited in `relationships.md` for networks that change). "During weekends we are more stationary ... Fridays and Saturdays are special because those days are used to explore new locations ... Weekends are again special, interestingly we here observe a drop in the number of social states, because we are not required to go to work or school ... our participants reserve weekends to socialize with a few selected friends."
- **Dodds, Harris, Kloumann, Bliss & Danforth 2011**, *Temporal patterns of happiness and information in a global social network: hedonometrics and Twitter*, PLoS ONE 6(12):e26752, DOI `10.1371/journal.pone.0026752`. The weekly cycle exists and is stable ("the three day midweek low followed by a peak around Saturday" in each of four time ranges); in the text read, 6.06 on the best day against about 6.03 on Wednesday and Thursday. Cited for the existence of a seven-day cycle; **not modelled**, the effect being too small to be a parameter the crate could calibrate.

The sources give a direction (a rest day is more stationary and narrower in company: a few selected friends) and no number. Every size in the overlay is a design default and says so.

## The model

- **`Curve::Days(from, to)`**: `1` inside the wrapping window `[from, to)` over the week, measured in days from its start, otherwise `0`; `Days(5.0, 7.0)` is the last two days, `Days(6.0, 1.0)` wraps, `Days(4.75, 6.0)` opens at 18:00 on day 4. The exact analogue of `Hours` on a seven-unit clock (`day_window` and `hour_window` share one function). Both ends must be in `0..=7`, or the definition is refused at load naming the activity.
- **`Input::Week`**: `(weekday + hour / 24) / 7`, in `[0, 1)`. It carries the hour so an edge can fall mid-day and the midnight tick belongs to the new day (`curves::tests::days_curve_puts_the_midnight_tick_on_the_new_day` checks all seven in `f32`). A `Days` curve needs it; it may also be read through any other curve.
- **`SocietySettings::start_weekday`** (`u8`, default 0, wraps modulo 7): the weekday of game-day 0. The crate has no names for days. `SocietyClock::weekday(start)` and `ctx.weekday()` (0 to 6, read from the tick-start snapshot like every `ctx` value) give the same calendar. Settings are not saved, so a host restoring a save passes the `start_weekday` it started with.
- **The week is the village's clock, never the body's.** A `Chronotype` shifts `BodyHour` and leaves `Week` alone: a person twelve hours behind starts no work on a rest day either (`tests/week.rs::a_body_clock_does_not_move_the_week`).
- **A week is seven days** (`DAYS_PER_WEEK`), a calendar convention and not a rate, so it is not a `Tuning` field. A host with another calendar writes a script.
- **The starter pack has no week.** A society that never writes `Week` is the previous release's, byte for byte (the harness at seed 7, 29 game-days, `cmp`: `summary.json`, `series.csv`, `narrative.txt`, `activities.txt`, `partners.txt`, `reapproach.txt`, `sleep.txt` and `floor.txt` are identical to `main`'s).

## The overlay, and what it took to make a rest day

`examples/assets/weekly.society.ron` replaces `work` with a copy shut on days 5 and 6 and adds `laze`: two hours of staying put (`Itself`) in daylight on a rest day, a flat score and a `weight` of 3.

**Three versions were measured, and the first two failed** (`docs/loop/runs/2026-10-31-04/evidence/attempt1/` and `evidence/sweep/`). The pre-registered prediction was that a rest day with no work would show fewer different people met and a larger share of approaches to friends (Sekara). It was **struck** at first:

1. *Shut `work`, add a `visit` that scores a liked person high.* Rest-day approaches a villager rose from 10 to **27-30** (villages; 19-20 in the towns), different people from 4.2-4.7 to 6.6-7.1, and the village was worse for it: friend ties 51 / 59 / 64 / 58 → 25 / 21 / 34 / 22 and rival ties 44 / 51 / 33 / 34 → 80 / 134 / 98 / 102 at seeds 7 / 42 / 11 / 99 over 29 game-days. The four hours a workday spends on work go to whatever else is open, and in this pack that is company: `compliment`, `gossip` and `snub` score on affinity or grudge, not on a need, so free time becomes contact, and contact breeds rivals. `visit` was chosen 0.03-0.07 times a villager-day.
2. *A `laze` that scores by the `fun` need.* It fills `fun` and drops out of the draw after a spell, so leisure is 12-14 % of a rest day at any weight from 4 to 16 (3 % on a workday) and a villager still makes 21-23 approaches.
3. *A `laze` with a flat score and a `weight`.* The `weight` is the one number that says how much of a rest day is idle: at seed 7, weights 1 / 3 / 8 give rest-day approaches 11.5 / 7.8 / 6.2 a villager against 10-11 on a workday, different people 4.4 / 3.4 / 3.1, rival ties 35 / 17 / 21 against 44 with no week, and a rest-day `social` need of 0.78 / 0.71 / 0.64. **3 is a design default**: the first weight at which a rest day is clearly quieter and narrower, at the price of a `social` need about 0.05 lower.

The `visit` of version 1 was carried into version 3 and then removed: with it, at seeds 7 and 42, the rest-day friend share is 47.0 / 48.5 %, without it 49.5 / 55.3 % (`evidence/sweep/with-visit-*`). The narrowing comes from the lazing, and an activity that adds nothing measurable is dead weight.

## What it does

Seven runs: villages of 24 at seeds 7 / 42 / 11 / 99, towns of 60 at seeds 7 / 99 and of 40 at seed 3, 29 game-days at 64 s/day, full days only (`evidence/week.md`, `evidence/plots/week.png`). Mean of the seven, no week against the overlay:

| | no week, workdays | no week, rest days | overlay, workdays | overlay, rest days |
|---|---|---|---|---|
| share of villager-ticks at work | 15.5 % | 15.1 % | 17.2 % | **0.0 %** |
| leisure (relax, wander, laze) | 2.3 % | 2.3 % | 2.4 % | **36.5 %** |
| approaches begun a villager-day | 10.7 | 10.8 | 10.7 | **7.5** |
| different people met a villager-day | 4.64 | 4.73 | 4.65 | **3.59** |
| share of approaches to a friend | 33.4 % | 32.8 % | 36.8 % | **44.0 %** |
| mean `social` need | 0.76 | 0.76 | 0.76 | 0.71 |
| mean `fun` need | 0.62 | 0.61 | 0.64 | 0.83 |

In every one of the seven runs a rest day, within the overlay's own week, has fewer approaches (by 2.6-4.3 a villager), fewer different people (by 0.7-1.5), a larger share of approaches to friends (by 1.4-13 points) and a lower `social` need (by 0.04-0.08) than a workday. A fresh village (seed 123) and town (50 people, seed 5) with `start_weekday` 3, not used to choose the weight, do the same (`evidence/week-fresh.md`): 8.5 and 6.9 approaches against 10.4 and 10.2 on a workday, different people 3.35 and 3.52 against 4.01 and 5.21, friend share 46.0 % and 39.9 % against 34.5 % and 33.2 %.

**What else moved, which the overlay does not claim to explain.** Approaches over the whole week fall about 9 % (9.8 a villager-day against 10.7), and with them rival ties fall or hold in all seven runs (44 → 17, 51 → 20, 33 → 24, 34 → 33, 109 → 82, 120 → 54, 65 → 24; in the fresh two 26 → 24 and 101 → 47) and insults fall in all seven (491 → 441, 433 → 390, 466 → 452, 429 → 426, 1404 → 1103, 1374 → 1062, 733 → 520; fresh 412 → 461 rises, 957 → 768 falls). Friend ties are **mixed in the villages** (51 → 39, 59 → 67, 64 → 44, 58 → 71; the fresh village 74 → 39) and higher in the towns (130 → 132, 116 → 168, 74 → 162; fresh 105 → 186). **Monday is busier than any other workday** in the overlay's week (12.6 approaches a villager against 10.1-10.5 on the next four days and 10.3 on a Monday with no week): the `social` need ran down over a quiet rest day (0.72 and 0.69 on days 5 and 6) and is spent when the work week begins.

## Not modelled

- **A weekly mood cycle** (Dodds 2011): too small to calibrate.
- **Social jetlag** (Wittmann et al. 2006, `chronotype.md`): sleeping later on free days. `BodyHour` and `Week` are two inputs a pack can combine, but no pack here does.
- **Exploring new places on Friday and Saturday** (Sekara): no activity targets a place the person has not been.
- **A calendar other than seven days, named days, months and seasons**: a script, or a later input.

# A week is seven days, and the starter pack has none

> **Assumed decision (owner may override), 2026-10-31, pass 2026-10-31-04.** Built as recommended below.

## The problem

The crate had an hour of the day and a count of game-days and nothing between, so every day of every society was the same day. A pack that wants a rest day, a market day or a Sunday visit could only write `Input::Script("ctx.day() % 7 ...")`: a script evaluation per consideration per person per tick, unvalidated at load, and invisible to an editor that shows `Hours(8, 17)` as a window.

## The decisions, with the options

1. **How a pack says "which days".**
   - (a) `Input::Week` and a `Curve::Days(from, to)` window, the analogue of `Hour` and `Hours` (**picked**: one mental model, validated at load, an editor shows it the same way).
   - (b) An `ActivityDef::days: Option<Vec<u8>>` filter. Rejected: a second mechanism beside the considerations, a hard gate where every other schedule is a consideration, and no mid-day edges.
   - (c) Leave it to scripts. Rejected: it is the problem.
2. **What the input carries.** `(weekday + hour / 24) / 7`, the hour included (**picked**), against the weekday alone. A window may then start mid-day (`Days(4.75, 6.0)` is Friday 18:00 to the end of Saturday) at no cost, and an input in `[0, 1)` is what every other input is.
3. **How long a week is.** Seven, a constant (`model::DAYS_PER_WEEK`), not a `Tuning` field. A week is a calendar convention and not a model rate; a host with a six-day week writes a script. Making it a field would make `Days(5.0, 7.0)` mean different things in different societies and force every window to be validated against a setting.
4. **Which day is day 0.** `SocietySettings::start_weekday` (`u8`, default 0, wraps modulo 7), the weekday of game-day 0. The crate has no names for days; day 0 is whatever the host calls the first day of its week. The setting is not saved (settings never are): a host restoring a save passes the one it started with.
5. **Body clocks and the week.** `Week` is the village's clock, never shifted by a `Chronotype`: a rest day is a social fact (`tests/week.rs::a_body_clock_does_not_move_the_week`).
6. **Does the starter pack change.** No. An opt-in overlay (`examples/assets/weekly.society.ron`) carries the rest day: the starter's `work` shut on days 5 and 6, a `laze`, and a rest-day `visit`. Options: put the week in the starter (changes every society, and the protected village pins read a society with no week), or ship the overlay only (**picked**, reversible; a society that never writes `Week` is the previous release's, byte for byte).
7. **What the overlay's rest day is.** The overlay's first version only shut `work` and added `visit`; measured, it **damaged the village** (friends roughly halved, rivals more than doubled, `evidence/attempt1/`) because the hours a workday spends on work go to whatever else is open, and in this pack that is company. A `laze` that scores by the `fun` need did not fix it (`evidence/sweep/fun-*`). A `laze` with a flat score and a `weight` of 3 does. The weight is a design default, swept at 1 / 3 / 8.

## What would change the pick

- If the owner wants a society's week on by default: apply the overlay to `assets/starter.society.ron` (replace `work`, add `laze` and `visit`) and give `craft` in the village pack the same `Days(0.0, 5.0)` consideration. Several protected pins read the starter's society and would need re-measuring first; this pass did not run them against a starter with a week.
- If a host needs another calendar (a six-day week, a lunar month): the right shape is a second input and curve over a `days_per_cycle` it supplies, not a field on `Tuning`. Not built: nobody has asked.

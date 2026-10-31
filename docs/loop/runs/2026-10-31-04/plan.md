# Plan - a week

## Candidates (ranked) and the pick

No open issues (`gh issue list` empty; bevy_emerge #104-#106 closed). Parked branches (passes 20-40) older, ignored. Ranked candidates:

1. **The society has no week** (ranked item 2 and idle ladder 2, "a model a shipping game expects and this crate lacks"; schedule-driven NPC lives with rest days and market days are the baseline of the genre). Sourced (Sekara 2016, Dodds 2011), purely additive, opt-in overlay, nothing protected can move. **Picked.**
2. Satiation after an accepted approach (pass 03's item 2): no source and no mechanism; the share of repeated partners (21-25 %) is not obviously wrong against real inner-circle data. Runner-up.
3. The walk home (`travel-time-in-two-clocks.md`): a decision the owner settled with `Locomotion::Host`; the residue is the harness's 64 s/day.
4. Re-record README clips: needs a display.

(1) beats (2): it is a missing capability a host cannot express cheaply, with two sources read; (2) lacks a source.

## Design

- `src/model/curves.rs`: `DAYS_PER_WEEK` (7), `day_window(from, to, day)` (the `hour_window` logic over a 7-unit period, shared), `Curve::Days(from, to)` = `day_window(from, to, x * 7)`.
- `src/defs.rs`: `Input::Week`, `Signal::Week`, validation (`Days` only with `Week` and `Week` only with `Days`; both ends finite and within `0..=7`).
- `src/core.rs`: `SocietySettings::start_weekday: u8` (0), `SocietyClock::weekday(start) -> u8`.
- `src/behavior.rs`: `Decision::week` (`weekday + hour/24`), `Signal::Week => week / 7`.
- `src/snapshot.rs`, `src/script.rs`: `ctx.weekday()`.
- `examples/assets/weekly.society.ron`: opt-in overlay (`work` on workdays, `visit` on rest days). `examples/evidence.rs`: `--weekly`, `--start-weekday`, `WeekLedger` -> `week.txt`; `tools/plot_weekly.py`.
- Docs: `docs/research/weekly-rhythm.md`, authoring guide (curve table, input), scripting guide (`ctx.weekday()`), concepts, CHANGELOG, README line; `needs-decision/a-week-is-seven-days.md`.

## Tuning fields

None. `start_weekday` is a world setting (like `start_hour`), `DAYS_PER_WEEK` is a calendar constant, and the overlay's numbers are pack data labelled as design defaults.

## Tests (`tests/week.rs`)

1. `Days` windows: inside / outside / wrap / empty / the midnight tick of each of the seven days.
2. `Input::Week` through a real decision: an activity gated on `Days(0, 5)` is chosen on days 0-4 and never on 5-6; `start_weekday` shifts which days; a wrapping `Days(6, 1)` window.
3. Validation: `Days` with `Hour`, `Week` with `Hours`, a window outside `0..=7`, are refused with the activity's id.
4. `ctx.weekday()` in a script agrees with the clock and `start_weekday`.
5. Determinism / save: a restore mid-week keeps the weekday (clock survives; settings re-supplied).
6. Default pack: `Input::Week` is absent, so the existing pins are untouched.

## Evidence plan

Same seeds (village 7/42/11/99, towns 60 at 7/99, 40 at 3), 17-30 game-days at 64 s/day: `--weekly` against off. `week.txt` per weekday: person-ticks by activity group, distinct partners per person-day, exchanges with close ties, approaches. Plot `plots/week.png`. Narrative: one person's rest day beside a workday.

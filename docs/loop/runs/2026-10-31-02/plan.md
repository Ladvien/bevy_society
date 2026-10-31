# Plan — body clocks (`Chronotype`, `Input::BodyHour`, `ActivityDef::unavailable`)

Research: `research.md`. One change: a person has their own clock, and a sleeper cannot be approached.

## Design

| Piece | Where | What |
|---|---|---|
| `Chronotype(f32)` | `src/mind.rs`, required by `Person` (`src/core.rs`) | Offset in hours of the body clock behind the wall clock; `Default` 0.0. `body_hour(clock_hour)` = `(clock_hour - c).rem_euclid(24)`. `sample(rng, sd_hours)`: Irwin-Hall sum of twelve `unit()` draws gives N(0, 1), times `sd_hours`, truncated at ±3 sd; no draw is made by the crate itself. `Chronotype::POPULATION_SD_HOURS` = 1.5 (design default, `research.md`). |
| `Input::BodyHour` / `Signal::BodyHour` | `src/defs.rs`, `src/behavior.rs` (`signal_value`) | The hour on the actor's own clock, divided by 24 exactly as `Hour` is. `Hours(from, to)` accepts `Hour` or `BodyHour`. `Decision` gains `body_hour`, computed once per person. |
| `ActivityDef::unavailable` | `src/defs.rs` (`ActivityDef`, `ActivitySpec`), `src/behavior.rs` | `bool`, default `false`. A person doing an `unavailable` activity is left out of every other person's `near` list in `decide_activities` (so never a `Person` target and never a `Distance` neighbour), and `end_activities` ends, `completed: false`, a not-yet-arrived activity directed at one. |
| Save | `src/save.rs` | `PersonSave::chronotype: Chronotype`, `#[serde(default)]` (older files restore as offset 0, the state they were saved without); `VERSION` stays 5, as for the other additive per-person fields (passes 18-21 of 2026-10-30). Captured and restored with the rest of the person. |
| Placement | `src/placement.rs` | `PersonDef::chronotype_hours: f32`, `#[serde(default)]`, validated finite and within ±12; written by `spawn`, read by `capture`. |
| Starter pack | `assets/starter.society.ron` | `sleep`: `(input: BodyHour, curve: Hours(22.0, 6.0))` and `unavailable: true`. A person with no `Chronotype` has `body_hour == hour`, so the only change to today's society is that nobody is approached while asleep. |
| Harness | `examples/evidence.rs` | `--chronotype-sd H` (default 0 = off) gives each villager a chronotype from its own stream (`seed ^ salt`, so personalities and every other draw are untouched); `sleep.txt` (share of the village asleep by clock hour, sleep start and wake histograms, interrupted sleeps, exchanges aimed at a sleeper); `chronotypes.txt`. |

No new `Tuning` field: the offset is per-person host data (like `Personality`), and the spread is chosen by the host when it samples. `Chronotype::POPULATION_SD_HOURS` is a documented constant, not a rate.

## Invariants kept

- Determinism: the crate draws nothing new; `sample` takes the host's `DetRng`. Persons with no `Chronotype` behave as before.
- Scripts and the snapshot are untouched (`ctx.hour` stays the wall clock; a script reading a body clock is a follow-up, listed in the log).
- `src/model/` stays engine-free: nothing added there (the arithmetic is two lines in `mind.rs`).

## Tests (`tests/chronotype.rs` and unit tests)

1. `body_hour` wraps in both directions and is the identity at 0.
2. `sample`: same seed same value; over 20 000 draws mean within 0.05 h of 0 and sd within 5 % of the request; never beyond 3 sd.
3. Scenario: a lark (-3), a person with no offset and an owl (+3) in one village with the pack's sleep rule: each starts sleeping at the wall hour their offset predicts, in order lark < none < owl, three hours apart within one decision step.
4. `unavailable`: a person awake beside a sleeper never targets them (with the flag), and does when the flag is off (the test fails before the change); a walk toward someone who then falls asleep ends not-completed.
5. `Hours` with a third input is refused with a message naming both inputs; `BodyHour` is accepted.
6. Save round trip (feature `serde`): the offset survives; a file without the field restores as 0.
7. Placement round trip: `chronotype_hours` is spawned and captured.
8. Determinism: same seed and same chronotypes twice give the same state hash.

## Evidence (section 6)

Village, seed 7, 17 game-days: `--chronotype-sd 0` (= HEAD, byte-compared) against `--chronotype-sd 1.5`; town of 60 (seed 7); fresh eyes: seed 99, 40 people. Plots: share asleep by clock hour, sleep starts by hour, interrupted sleeps, one lark's and one owl's day side by side, needs and ties must not regress (friends/rivals/insults/rumor reach next to HEAD). Perf at 100/1 000/5 000 people.

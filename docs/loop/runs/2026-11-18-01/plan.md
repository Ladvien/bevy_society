# Plan: check the observable per person, and stop recording a row that is not a sample

## Why this over the runner-up

Runner-up was `examples/capture.rs` still hardcoding 8.0 s/day. Lost on one sentence: **the log's #1 was
that the observable says "every person" while the evidence was a mean**, and measuring it falsified a
`measured` status — which is worth more than a third configuration nobody reads.

## Design

No library change. Three harness corrections and one test:

- **The tick-0 row is not recorded.** `size_person_state` runs on the first society tick, so a row taken
  before it reads `Needs([])` and `0.000` for every need. That single row made "crosses below 0.35" true
  of the mean while no individual came close.
- **A person with empty `Needs` is not counted** in the mean, so the pre-sizing state averages nothing.
- **`tests/need_cycle.rs` rewritten** to measure per person and to assert what is true: `food` cycles for
  all eight, `rest` never does.
- **The scenario goes back to `pending`**, with the per-person numbers in its note.

## What the measurement found

| need | lowest any person reached | cycles for all 8? |
|---|---|---|
| social | **0.39** | **no** |
| rest | **0.50** | **no** |
| food | 0.00 | **yes** |
| fun | 0.12 | 6 of 8 |

Only `food` cycles for everyone. Three needs staying high is correct at these rates; the observable assumes
starvation that does not happen, which is a design question.

## Touched

- `examples/evidence.rs` — the tick-0 skip, the need-count guard.
- `tests/need_cycle.rs` — per-person, plus a test that the pre-update state has no needs.
- `docs/loop/scenarios.yaml` — `need-cycle` back to `pending` with the numbers.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `only_food_cycles_for_every_person_and_the_scenario_says_so` | `food` cycles for all 8 and `rest` for none; the second assertion fails the day the rates change |
| `a_series_row_taken_before_the_first_update_is_not_a_sample` | the pre-update state has **0** needs and the first society tick has 4 |

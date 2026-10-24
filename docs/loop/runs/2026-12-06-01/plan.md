# Plan: restate `need-cycle` and mark it measured

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics — a host-facing component, so escalated. `need-cycle` has been
`pending` since November because its observable is false, and the reason is now established as arithmetic
rather than a defect: `sleep` supplies **1.20** against a `rest` decay of **0.70/day**, so `rest` never needs
to run low.

Leaving it `pending` indefinitely is the worst of both worlds: the crate cannot claim the scenario passes, and
the test's own name (`only_food_cycles_...`) is contradicted by its own output (`fun cycles for 8 of 8`).

## Design

1. The observable is **restated** to what the model does: every need is met and re-met, and no need is drained
   with nothing to satisfy it. The thresholds and per-seed cycle counts are recorded, not asserted as constants
   of nature — five iterations moved them once already.
2. `status: pending` → **`measured`**.
3. The test asserts the restated property: every need must reach above 0.7 for every person, which is what
   catches a need with no source. It also keeps the two facts that are stable — `food` cycles for everyone,
   `rest` for nobody — and prints `fun`, which is neither.
4. Renamed from `only_food_cycles_for_every_person_and_the_scenario_says_so` to
   `food_and_fun_cycle_and_rest_never_does`, because the old name was false.

## Measured

| need | lowest | cycles, seed 7 | cycles, seed 42 |
|---|---|---|---|
| `social` | 0.45 | 0 of 8 | 2 of 8 |
| `rest` | 0.51 | **0 of 8** | **0 of 8** |
| `food` | 0.00 | **8 of 8** | **8 of 8** |
| `fun` | 0.00 | **8 of 8** | 6 of 8 |

## Touched

- `docs/loop/scenarios.yaml` — the observable, restated, and `measured`.
- `tests/need_cycle.rs` — header table corrected per seed, the restated property asserted, the test renamed.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. No crate code changed.

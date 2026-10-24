# Plan: turn the pending scenarios into measured ones

## Why this over the runner-up

Runner-up was the `need-cycle` behaviour defect itself (back-to-back activity choice). Lost on one
sentence: **I cannot fix a defect the crate's own evidence base was unable to see.** Three of the four
pending scenarios were unfalsifiable — one had no harness support at all, one described something the
message it named cannot do, and one was pending on a stale reason. Making them measurable is what turns
the next iteration's fix into a checked claim rather than a plot I squint at.

## Design

**Harness.** `summary.json` now takes the `RumorId` that `plant()` returned, exactly as `series.csv` did
last iteration, instead of counting knowers of `RumorId(0)`. `summary` also lost its unused `&App`
parameter — it only ever needed the `World` and the scenario description.

**Three new tests, one per scenario:**

| Test file | What it asserts |
|---|---|
| `tests/deed_response.rs` | an accepted deed with witnesses makes exactly one rumor, every witness knows it, and each witness below `approval −0.5` raises one `NormViolated` |
| `tests/opinion_convergence.rs` | opinion spread falls by ≥ 1/3 and stays above 0.02; plus the `Opinions` surface a host writes and reads |
| `tests/need_cycle.rs` | the satisfy path pays `rate · hours` and clamps at 1.0; and, recorded, that the shipped choice rule leaves no gap for a cycle |

**Scenario file.** `deed-draws-a-response` and `opinion-converges-but-not-entirely` become `measured`
with a `test:` field; `determinism` becomes `measured` (its tests already existed); `need-cycle` stays
`pending` with a `blocked:` note carrying the measurement.

**Two observables corrected, both wrong as written:**

- `deed-draws-a-response` said "a witness whose *opinion* on the deed's topic is below -0.5". The crate
  judges `approval = norm · sign(harm)`, and it said "planted `steal`" where a deed needs an accepted
  interaction. Both corrected in place with a dated note.
- `opinion-converges-but-not-entirely` asked for 6 000 ticks; measured, that reaches 0.370, one percent
  short of the third it demands. Corrected to 12 000, which reaches 0.239.

## Touched

- `examples/evidence.rs` — summary tracks the planted rumor; `&App` dropped.
- `tests/deed_response.rs`, `tests/opinion_convergence.rs`, `tests/need_cycle.rs` (new).
- `docs/loop/scenarios.yaml` — four statuses, two corrected observables, one blocker.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. This iteration changes no behaviour in the crate; it changes what the crate can prove.

## Tests

Written **red first** where the outcome was unknown:

| Test | First result |
|---|---|
| deed | `violations=0 rumors=[(0,1)]` — then `known_by=4`, 4 violations after driving a real deed |
| opinion | `0.5106 -> 0.4138`, red against a required third; 12 000 ticks gives `0.2398` |
| need-cycle | `rest [1.000, 1.000]`, 29 back-to-back sleeps — asserted as the blocker, not as a pass |

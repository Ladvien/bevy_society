# Plan: honour the spec's seed, and re-derive every scenario at its own settings

## Why this over the runner-up

Runner-up was `food` at 15% of ticks at zero. Lost on one sentence: **that number was measured at 4 s/day
and does not survive the correction** — `food` now cycles once a day with a mean of 0.58 in `need-cycle`
and 0.24 in the village. Re-deriving was the log's #1, and it retires a queue of stale conclusions.

## Design

- `Spec` gains `seed`, read from the file's `seed:` line. `--seed` still overrides it, so a fresh-eyes pass
  can move the seed without editing the spec.
- Every scenario is re-run at its declared settings and seed, and the results are recorded.
- `need-cycle` moves from `pending` to `measured`, with the re-derivation written into its observable.

No library change: `src/` is untouched.

## What the re-derivation found

| scenario | need means | all four cycle? |
|---|---|---|
| `need-cycle` | [0.86, 0.81, 0.58, 0.79] | **yes** |
| `village-evening` | [0.29, 0.53, 0.24, 0.26] | **yes** |
| `rumor-crosses-a-cluster` | [0.64, 0.15, 0.40, 0.61] | **yes** |
| `deed-draws-a-response` | [0.86, 0.81, 0.63, 0.81] | **yes** |

and `rumor-crosses-a-cluster` reaches **0.583**, above its 40% observable — it had been `measured` while
failing.

## The plot nearly fooled me

At full range `need-cycle` looks like violent thrashing. Counting troughs below 0.4 gives **1.0 per day**,
and only for `food`: the dense variation is small high-frequency excursions, not cycling. The zoomed plot
over three days is what one day actually looks like, and it is a clean daily cycle. Both plots are in
`evidence/`, with the counts that separate them.

## Touched

- `examples/evidence.rs` — `Spec::seed`.
- `docs/loop/scenarios.yaml` — `need-cycle` → `measured` with its re-derivation; the seed is now read.
- `tests/scenario_spec.rs` — a test that every scenario declares a seed.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `every_scenario_declares_a_seed` | the file states a seed for each scenario, so none silently runs at the crate default |
| the existing three | unchanged: a day length, a fitted timeout for the village scenarios, and the merge still compiling |

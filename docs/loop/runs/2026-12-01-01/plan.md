# Plan: add the positivity offset

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics (49.8% of person-ticks) — a component a host reads, so
escalated. Last iteration's own decision file had three options and I found a **fourth** with a source: the
paper that names the missing mechanism says the offset is the intercept of the positive channel, and the
crate has the gain and not the intercept. That is a model omission, not a tuning disagreement, and it is
one term.

## Design

`Tuning::positivity_offset` (default 0.1), threaded to `MoodRates::positivity_offset` and applied in
`Mood::step`:

```rust
let resting = baseline.pleasure + rates.positivity_offset;
self.pleasure += (resting - self.pleasure) * back;
```

Pleasure only. Arousal and dominance have no offset in the model — the paper's positivity offset is about
approach/withdrawal, which is the pleasure axis — and a test pins that they still return to the bare
baseline.

## Measured

`village-evening`, seed 7, 24 people, 17 game-days:

| offset | mean pleasure | **max pleasure** | stdev |
|---|---|---|---|
| 0.0 (old) | −0.154 | **+0.0000** | 0.035 |
| **0.1 (new default)** | −0.122 | **+0.0155** | 0.032 |
| 0.3 | −0.058 | **+0.0476** | 0.036 |

Monotonic in the offset, and the maximum **crosses zero for the first time**. Needs are unaffected
(0.596 → 0.603), so this is a mood change only.

## Honest limit

The mean is still negative. The offset tilts the resting point; a saturated negative emotion centre dilutes
it in proportion to intensity (`x = (pull·I·c + back·rest) / (pull·I + back)`), so at `I = 0.5, c = −1.0`
pleasure sits near −0.6 regardless. The **appraisal balance** is the remaining lever and it stays a decision.

## Touched

- `src/core.rs` — the `Tuning` field.
- `src/mind.rs` — `MoodRates`, `Mood::step`, and two unit tests updated to the new resting point.
- `tests/mood_offset.rs` (new), `tests/mind.rs` (updated).
- `examples/evidence.rs` — `--positivity-offset`, so a host can re-read a scenario at a different resting point.
- `CHANGELOG.md`, `docs/research/personality-and-mood.md`, `docs/loop/log.md`.

## New `Tuning` fields

| field | default | source |
|---|---|---|
| `positivity_offset` | 0.1 | Cacioppo, Cacioppo & Gollan 2013, DOI `10.1017/S0140525X13002537`; the magnitude is a design default |

## Tests

| Test | What it proves |
|---|---|
| `tests/mood_offset.rs::a_person_with_no_emotions_rests_above_neutral` | an idle person's pleasure is **above zero** with the offset and exactly zero without |
| `tests/mood_offset.rs::a_zero_offset_leaves_the_old_behaviour` | the old behaviour is a setting, not a regression |
| `mind::tests::mood_relaxes_to_baseline_plus_the_positivity_offset` | pleasure returns to baseline **plus** the offset; arousal still returns to the bare baseline |

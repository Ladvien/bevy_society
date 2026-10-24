# Plan: put the label thresholds on `Tuning`

## Why this over the runner-up

Two candidates: a sixth attempt at the friendship clique (blocked by the five-failure rule and documented), and
the mood residual (−0.05, already near neutral after three iterations). Neither is a good use of a round. The
README makes a claim the crate does not honour, and no iteration had audited it.

## Design

| field | default | was |
|---|---|---|
| `friend_affinity` | 0.4 | literal in `classify` |
| `friend_familiarity` | 0.3 | literal |
| `rival_affinity` | 0.4 | literal |
| `rival_familiarity` | 0.2 | literal |
| `stranger_familiarity` | 0.1 | literal |
| `rejection_affinity_penalty` | 0.05 | literal in `resolve_interactions` |

`TieLabel::classify(affinity, familiarity, &Tuning)`.

## Measured

| | before | after |
|---|---|---|
| `village-evening` series rows | 67 199 | **67 199 identical** |
| friends / rivals | 22 / 33 | **22 / 33** |

The defaults equal the literals, so nothing moves. Verified by comparing every series row.

## The test

`tests/tuning_covers_rates.rs`: the five defaults equal the old literals, and **raising `friend_affinity` to 0.9
demotes a tie that was a friend** — which proves the field is genuinely read, not merely declared.

## Touched

- `src/core.rs` — six fields and their docs.
- `src/relationships.rs` — `classify` takes `&Tuning`; its unit test updated.
- `src/behavior.rs` — the rejection penalty reads the field.
- `tests/tuning_covers_rates.rs` (new), `tests/exposure_equilibrium.rs` (caller updated).
- `CHANGELOG.md`, `docs/research/relationships.md`, `docs/loop/log.md`.

## New `Tuning` fields

Six, all design defaults, all equal to the literals they replace.

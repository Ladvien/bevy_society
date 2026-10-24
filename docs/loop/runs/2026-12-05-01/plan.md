# Plan: assert the observable, and state both plant ticks

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics — a host-facing component, so escalated. But a scenario marked
`measured` whose observable **nothing asserts** is a larger problem than any single behaviour: every reading
of it has been by eye, and four iterations of model change went through it unverified. A flake was sitting in
the file.

## Design

1. `tests/village_evening.rs` asserts the rumor condition across seven seeds, counting **villagers only** —
   the player is a `Person` without `Autonomous` and never gossips, so a rumor reaching only the player died.
2. `scenarios.yaml` and `examples/evidence.rs` both state that the harness plants at **8 960**, not the
   **300** the file names, and why the conversion exists.
3. No model change.

## Measured

| planted at | seeds spreading it |
|---|---|
| **300** (the file's number) | **7 of 7**, reaching 2–8 villagers |
| 8 960 (the harness's number) | 8 of 10 |

## What the test asserts

Not a single seed. The scenario names seed 7 and the condition holds there, but a single-seed assertion lets
the flake back in the moment anything upstream moves. It asserts the scenario's own seed **and** that the
mechanism works at more than half the seeds — so a regression shows up as a number rather than as a mystery
clip.

## Touched

- `tests/village_evening.rs` (new).
- `docs/loop/scenarios.yaml` — both ticks stated, and the villager-only rule.
- `examples/evidence.rs` — `plant_at`'s doc says it returns 8 960 and why that is not the file's 300.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

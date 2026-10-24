# Plan: test the recommended fix before shipping it

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics — escalated. The top of the list was the affinity saturation
that last iteration recommended, and a model change to how every tie forms should be measured before it ships,
not after.

## What was done

1. Searched the corpus for a citation for the saturating shape. **None** — the closest paper is about
   developmental trajectories, not per-exchange diminishing returns. So the shape is a design default.
2. Built it: `affinity + gain · (1 − |affinity|)` in `resolve_interactions`.
3. Measured it **alone** and **with the contact-exchange mechanism that produced the original clique**.
4. Reverted.

## Measured

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| saturation alone | 22 | 38 |
| contact talk at 3 h | **267** | **0** |
| **saturation + contact talk** | **260** | **0** |

## What that establishes

Saturation does not stop the clique. It changes when affinity crosses `0.4`, not whether — and both curves
cross it within 45 exchanges.

## Design

Nothing shipped. The decision file's recommendation is **retracted**, with the measurement and three remaining
options, recommending a **trust gate** next because trust already exists in the crate and already moves far more
slowly than affinity.

## Touched

- `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` — the retraction and the sweep.
- `CHANGELOG.md`, `docs/loop/log.md`.
- **Reverted**: `src/behavior.rs`, `src/core.rs`, `src/relationships.rs`.

## New `Tuning` fields

None.

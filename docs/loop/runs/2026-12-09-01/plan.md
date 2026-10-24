# Plan: find what makes a friendship, and correct a mislabelled citation

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics — escalated. The top of the list was "tie formation is
over-sensitive to contact volume", and it earned the iteration because **two mechanisms in two iterations**
produced a clique from a plausible local rule. A third attempt without knowing why would have failed the same
way.

## What was measured

1. Friend-ties over a full run: **25 → 115**, linear, never saturating, affinity at the clamp.
2. A sweep of `mere_exposure_per_day` (0.05 → 0.005) and `balance_rate_per_day` (0.5 → 0.05): friends
   **22 / 15 / 15 / 16** and **22 / 25**. **Nearly flat** — exposure and balance are not the driver.
3. A single pair: `Friend` after **45** interactions, affinity **0.73** after 57.

## What that establishes

Familiarity from exposure saturates. **Affinity from interaction does not.** Friendship is a monotone function
of how many exchanges happened, so any mechanism that adds contact adds friends in proportion — which is
exactly why both earlier mechanisms produced cliques.

## Design

Nothing in the model changed. Two things shipped:

1. **`mere_exposure_per_day`'s doc corrected.** It cited Zajonc 1968 as though the paper supplied the rate; the
   paper supplies only the mechanism. The doc now states the equilibrium (`0.833`), the three sweeps that show
   exposure is not the driver, and the 45-interaction result.
2. **`tests/exposure_equilibrium.rs`** pins the arithmetic that neither rate reveals alone: exposure and
   relaxation are first-order and opposite, so affinity settles at `exposure / (exposure + relax)`. It asserts
   the fixed point — true at any rate — and *records* rather than asserts what that means for the Friend
   threshold, so the recommended fix is not blocked by a test.

## Why no fix shipped

The fix is a **saturating term on interaction affinity**, and that changes how every tie forms for every host.
`needs-decision/friendship-has-no-diminishing-return.md` records three shapes and recommends it. Not this
iteration's call.

## Touched

- `src/core.rs` — the `mere_exposure_per_day` doc (no behaviour change).
- `tests/exposure_equilibrium.rs` (new).
- `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` (new).
- `CHANGELOG.md`, `docs/research/relationships.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

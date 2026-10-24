# Plan: find the lever for the friendship clique, and prove it

## Why this over the runner-up

`Doing`'s travelling semantics is escalated, and the friendship clique had failed five times — but last
iteration's `Tuning` change made it measurable without touching code, which is exactly the condition for a sixth
attempt to be worth making.

## What was done

1. **Swept the bar.** `--friend-affinity` added to the harness; `friend_affinity` at 0.4 / 0.55 / 0.7 / 0.8 gives
   friends **22 / 18 / 18 / 16**. The bar is not the lever — affinity reaches 1.0, so any bar below it is passed.
2. **Tested length.** A four-times-longer run saturates at **122** friend-ties and then decays. Not a runaway;
   a saturating system with a high asymptote. **This corrected my own prediction.**
3. **Tested the asympote shape.** Clamp affinity to `[-K, K]`, `K = 0.35`, below the bar.
4. **Found and fixed an asymmetry** in the first version of the clamp.
5. **Reverted** the model change; kept the harness flag.

## Measured

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | **22** | **33** | -0.054 |
| clamp only, bar 0.4 | 0 | 18 | -0.078 |
| clamp + trust 0.55 + bar 0.25 | **15** | **18** | -0.078 |
| clamp + trust 0.55 + bar 0.30 | **12** | **18** | -0.078 |

## Why it did not ship

It bounds the network — the first shape in six to do so — and costs mood, traced to the clamp rather than the
friend count. Choosing between those is the decision file's call, and it now has the numbers to choose with.

## Touched

- `examples/evidence.rs` — `--friend-affinity`, kept: it is how a host reads a tighter or looser village.
- `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` — the sweep, the length test, the clamp
  table, the cost attribution, and the asymmetry warning.
- `CHANGELOG.md`, `docs/loop/log.md`.
- **Reverted**: `src/behavior.rs`, `src/relationships.rs`.

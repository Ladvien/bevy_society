# Eval

## Change

**None shipped.** The repair was built, measured, and reverted — because the suite found a coupling the plan did
not anticipate.

## What the measurement establishes

| claim | measured |
|---|---|
| the mood cost is a curve/gate mismatch | `compliment` scores **0.057** at the 0.35 ceiling against **0.673** at 0.79 |
| the cost can be repaired | gate at `Logistic(16.0, 0.25)` → mood **-0.045**, better than baseline |
| the bar must move under the ceiling | 0.4 is unreachable at 0.35; at 0.30 → **16 friends / 10 rivals** |
| the clamp re-times the opinion model | spread falls **25%** where the scenario asserts **33%** |

## The finding

**`affinity` is shared between liking and influence.** `Opinion` convergence reads `Ties`, so the ceiling that
bounds friendship also halves how fast opinions spread — and `opinion-converges-but-not-entirely`'s 12 000 ticks
is silently a statement about that rate.

None of the six earlier attempts found this, because none changed the affinity *scale*: they changed rates,
gates, or the accumulation shape. This one changed the range, and the range is load-bearing in a second model.

## What it says about the decision

The clamp is not a number to pick. It needs either a longer `ticks:`, a compensating `influence_mu`, or a
separate influence weight — and the third is a schema change. That is recorded with the numbers.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **53** binaries (clean after revert) |
| village after revert | friends **22**, rivals **33** |

## Dead-code inventory

Zero.

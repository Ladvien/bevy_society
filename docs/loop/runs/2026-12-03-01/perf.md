# Perf

## The change

Two `trust` fields on three interactions' `on_accept`, applied through the existing `TieDelta` path. No new
system, no new query.

## Behaviour

| | before | after |
|---|---|---|
| `village-evening` mean pleasure | −0.0657 | **−0.0579** |
| accept rate (controlled) | 0.486 | **0.533** |
| mean needs | 0.607 | 0.607 |

Needs unchanged. Trust rises, so more interactions are accepted, so slightly more exchange work runs — the
cost moves in the direction the change intends.

# Eval

## Change

`steering.rs::NUDGE` 0.05 -> 0.30. One constant, with the measurement beside it.

## Before / after, same seed

| | interrupted | friends | rivals |
|---|---|---|---|
| before | **40** | 22 | 33 |
| after | **19** | **26** | 33 |
| no steering at all (floor) | 4 | | |

Interrupted activities **halved**, friends **up 18%**, rivals unchanged.

## What the sweep establishes

The optimum is interior, which is the evidence the constant is doing real work rather than being nudged
monotonically toward a rail:

| nudge | interrupted |
|---|---|
| 0.05 | 40 |
| 0.15 | 32 |
| 0.20 | 29 |
| 0.25 | 24 |
| **0.30** | **19** |
| 0.60 | 27 |

Too small and the symmetric stall survives; too large and the side-step pushes walkers off their line.

## The diagnosis, established rather than guessed

Three independent confirmations:

1. **Steering is the cause**: without it, 4 interruptions; with it, 40.
2. **Crowding scales it**: 6 people -> 6, 12 -> 19, 24 -> 40.
3. **Direct measurement of the stall**: printing remaining distance at every timeout showed walkers sitting
   2.0-4.0 m from their target for the whole 4-hour budget, with `Destination` still present and steering
   never closing.

`steering.rs`'s own comment already named the mechanism — "ORCA has no reason to prefer left over right at an
exactly head-on meeting, so it stalls" — so this is the constant failing to do the job its comment claims.

## What this does not fix

The residual 19 is **geometric**, not kinetic. Twenty-four bodies of 0.35 m radius cannot all occupy a 1.5 m
arrival circle, so the outer shell legitimately cannot get in. The floor of 4 is reachable only by fixing what
the village *means*: `ActivityTarget::Home` ignores `Place::capacity`, so 24 people are told to share a place
that declares room for 6.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **55** binaries |
| `cargo test --no-default-features` | **55** binaries |
| clippy / docs / fmt | 0 errors |

## Coverage, honestly

**No test ships with this change.** Three were written; all three passed at the old nudge as well, meaning none
of them detected the behaviour. They were deleted rather than kept, because a test that passes either way is
worse than no test: it claims protection it does not give. The measurement is reproducible by the command
recorded in `perf.md`, and the constant carries its table.

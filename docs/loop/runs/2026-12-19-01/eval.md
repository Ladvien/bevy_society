# Eval

## Change

`steering.rs::NUDGE` 0.05 -> 0.30. One constant, with the measurement beside it.

## Before / after, whole village, same seed

Counting every `ActivityEnded` for all 24 people (`summary.json`'s `completed_endings` /
`interrupted_endings`), 17 game-days, seed 7:

| NUDGE | completed | interrupted | failed share |
|---|---|---|---|
| 0.05 | 6 948 | 2 872 | **29.2%** |
| **0.30** | **7 571** | **2 237** | **22.8%** |

**A 22% reduction in failed activities** — 635 more activities run to completion.

### The correction that matters

The first version of this iteration quoted **40 → 19** interruptions, a 53% improvement. That was a **24x
sampling error**: the evidence harness's narrative follows a *single tracked person*, and I had been quoting one
villager's evening as if it were the village's. On a whole-village count the effect is real but less than half
as large.

The error is why `examples/evidence.rs` now counts endings for everybody and writes them to `summary.json`. The
metric had to exist before the claim could be made honestly.

## What the sweep establishes

The optimum is interior, which is the evidence the constant is doing real work rather than being nudged
monotonically toward a rail:

The one-person narrative sweep was **not monotonic** and not reproducible in shape — 178, 298, 229, 75, 110 at
0.05 / 0.15 / 0.25 / 0.30 / 0.60 on the post-merge tree — which is itself the evidence that a one-person sample
is the wrong instrument. What survives on the honest measure is the comparison, not the curve: **0.30 fails
22.8% of endings against 0.05's 29.2%**.

## The diagnosis, established rather than guessed

Three independent confirmations:

1. **Steering is the cause**: without it, 4 interruptions; with it, 40.
2. **Crowding scales it**: 6 people -> 6, 12 -> 19, 24 -> 40.
3. **Direct measurement of the stall**: printing remaining distance at every timeout showed walkers sitting
   2.0-4.0 m from their target for the whole 4-hour budget, with `Destination` still present and steering
   never closing. This part stands — it is a property of the solver, not of a sample.

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

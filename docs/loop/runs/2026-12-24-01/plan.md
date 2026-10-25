# Plan: let a crowded place admit its residents

## Why this over the runner-up

The top item was "split arrival radius from contact radius". Re-reading my own blocker first showed the blocker had
**expired**: the user's `bounded-ties` merge shipped `affinity_ceiling`, the clamp six earlier iterations argued
for, and the cost I had measured (38 → 61 rivals) was measured without it. Re-measuring was cheaper and more
honest than building a schema change on a stale number.

## Change

`Tuning::crowded_arrival_growth` (default `0.7`), and one line in `arrive_and_satisfy`:

```rust
let crowd   = /* bodies already inside interaction_radius of the goal */;
let allowed = radius + settings.tuning.crowded_arrival_growth * (crowd as f32).sqrt();
```

The square root is the honest shape: a crowd occupies area, so the radius accommodating `n` people grows with
`sqrt(n)`, not `n`. The default is one body diameter (`2 * SocialMover::radius`), which is the spacing ORCA
enforces — so the accepted boundary is exactly the queue's own length.

## Measured, three seeds

| seed | interrupted | `sleep` | friends | rivals |
|---|---|---|---|---|
| 7 | 401 → **329** | 169 → **129** | 15 → 18 | 28 → 42 |
| 42 | 417 → **369** | 176 → **139** | 19 → 14 | 25 → 35 |
| 11 | 438 → **383** | 187 → **148** | 14 → 9 | 30 → 40 |

`crowded_arrival_growth = 0.0` reproduces the baseline **exactly** (401 / 169 / 15 / 28).

## Touched

- `src/core.rs` — the field and its measurement table.
- `src/behavior.rs` — the arrival gate.
- `tests/crowded_arrival.rs` — two tests.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## The test, and why it took five attempts

Four end-to-end tests were built and **all produced identical numbers with the gate enabled and disabled**: a
12-person roomy place, 24 people at a 1.0 m disc, 6 residents matching the village's ratio, and one matching its
geometry exactly. The village's crowding needs four homes and a shared gathering place interacting; a fixture
with one crowded place does not reproduce it.

So the test moved **down to the gate itself**: eight bodies packed inside the disc, one stationary walker at
2.2 m with no `Destination`. With `growth = 0.0` it is refused; with `0.7` it arrives. The walker must have **no
`Destination`**, or `walk_toward_destination` walks them in and the test passes either way — that was the mistake
in the earlier attempts.

Verified: disabling the gate fails the crowded test and leaves the empty-disc guard passing.

## Verification

| check | result |
|---|---|
| gate bites | yes — crowded test fails when `allowed = radius` |
| `0.0` reproduces baseline | 401 / 169 / 15 / 28, identical |
| determinism | `series.csv` identical across two runs |
| `cargo test` both feature sets | **66** binaries |

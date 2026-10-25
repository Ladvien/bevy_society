# Plan: break the crowd deadlock in ORCA steering

## Why this over the runner-up

The loop's item 2 is "behavior that reads as wrong when you watch it". Reading `village-evening`'s narrative
showed **19 of 27 sleeps interrupted**, and the crate's own steering made it 10x worse than not steering at all.
That is the most visible wrongness in the run, and it is in code this crate ships.

## Change

One constant: `steering.rs::NUDGE`, `0.05 -> 0.30`, with the measurement recorded beside it.

## Touched

- `src/steering.rs` — the constant and its doc comment.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.
- **No test added.** Three attempts were written and all three passed at the old nudge too, so none of them
  measured the change; shipping them would have been false coverage. See `perf.md` for the honest account.

## Measured

| nudge | interrupted | friends | rivals |
|---|---|---|---|
| 0.05 (shipped) | 40 | 22 | 33 |
| 0.15 | 32 | | |
| 0.20 | 29 | | |
| 0.25 | 24 | | |
| **0.30** | **19** | **26** | **33** |
| 0.60 | 27 | | |

Floor with no steering at all: **4**.

## Verification

| check | result |
|---|---|
| `cargo test` | **55** binaries |
| `cargo test --no-default-features` | **55** binaries |
| clippy / docs / fmt | 0 errors, clean |
| reproducibility | two runs at 0.30 give 19 / 26 / 33 both times |

## Deliberately not done

- `Home` still ignores `Place::capacity` (unlike `Place(tag)`). Fixing it moved interruptions 40 -> 27, but it
  changes what the village means, so it is a decision.
- Body radius `0.35 -> 0.20` also helped (40 -> 19) but changes a physical property for a numerical reason.
- The travel timeout is `travel-time-in-two-clocks.md`, still open.

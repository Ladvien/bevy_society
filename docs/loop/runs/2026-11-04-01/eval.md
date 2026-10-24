# Eval

## The defect, before and after

One person, one place 50 m away, no steering plugin, 1 200 ticks. Its target is unreachable, so every
activity times out.

| | starts | ends | unpaired ends |
|---|---|---|---|
| before | **2** | **55** | **53** |
| after | 150 | 149 | 0 (the one in flight) |

Same at the population the fresh-eyes gate uses (40 people, 4 200 ticks):

| | starts | ends |
|---|---|---|
| before | 23 745 | 26 412 |
| after | **1 419** | **1 411** (all `completed`) |

The smaller absolute numbers after are not a slowdown: with starts paired to decisions, needs are
served differently and the run takes fewer, longer activities.

## Behaviour moved, at the same seed

`village-evening`, seed 7, 24 people, 4 200 ticks. Only the message timing changed; no rate was
touched.

| | before | after |
|---|---|---|
| rumors planted | 28 | **53** |
| rumor known by | 12 | 8 |
| rumor reach | 0.500 | **0.333** |
| friend edges | 91 | **118** |
| rivals | 0 | 0 |

`evidence/before-after.png`, `evidence/{before,after}/summary.json`. The gossip change is expected:
`gossip` is gated on `ctx.tellable`, and the tellable set moves when exchanges happen at slightly
different times.

## What this does *not* fix, measured

| Setting | need2 at 0 | need3 at 0 |
|---|---|---|
| one plaza, zero travel | 9.6% | 28.2% |
| village, 24 people | 46.4% | 78.2% |

Recorded, not fixed: the cause is activity *choice* (`snub` 589 + `gossip` 561 of 1 419 starts, and
neither satisfies a need), not the message surface.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 20 binaries, 17 in `behavior` |
| `cargo test --no-default-features` | 20 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `cargo doc --no-deps --all-features` | clean |

## Dead-code inventory

Zero, unchanged. The fix *removed* a parameter (`MessageWriter<ActivityStarted>` from
`arrive_and_satisfy`) rather than adding surface.

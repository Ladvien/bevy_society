# Eval

## Change

1. `grudge` is the **maximum** of `max(0, −valence) · recency` over a person's memories, not the sum. A
   design default, documented as one.
2. A refused interaction's memory is sized by the refusal (`0.3`), not by the deed's `harm` — which only
   applies when the deed actually fired.

## Before / after

| | seed 11 | seed 42 |
|---|---|---|
| `snub` share of activity starts, sum | **50.3%** | **48.4%** |
| `snub` share, max | **26.0%** | **32.6%** |
| longest single-activity run | 10 → **6** | 11 → **8** |
| top activity | `snub` → **`compliment`** | `snub` → **`gossip`** |
| refused-insult memory | 0.700 → **0.300** | — |

Village (`village-evening`, seed 7): travelling 58.9% → **59.0%**, need mean 0.545 → **0.542**. Seed 42:
59.3% / 0.550. No scenario slowed or degraded.

## What is *not* claimed

Seed 11 showed friends:rivals **4:9 → 6:5** and rumor reach **0.167 → 0.333**. Seed 42 shows reach
**0.167 → 0.167** and friends **5 → 6**. The reach doubling is seed-specific and is not part of the claim.
The snub reduction holds at both seeds.

## Why `sum` was wrong, stated without a paper

`sum` + `clamp` makes the value a function of `memory_capacity`: four retained trivial slights reached 1.0
exactly as one betrayal did, so a person's opinion of another was decided by how many small entries the
buffer happened to hold. That is not a psychological quantity. `max` makes the grudge the worst thing that
person did to you, which is what a grudge is. **No paper settles this** — the corpus search for
peak-versus-accumulate returned nothing usable — so it is recorded as a design default, not as a cited
model.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **40** binaries, 0 failures |
| `cargo test --no-default-features` | 40 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.

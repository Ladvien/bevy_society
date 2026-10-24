# Eval

## Change

No model change. The `mere_exposure_per_day` doc corrected, a test pinning the exposure equilibrium's
arithmetic, and a decision file recording the real finding.

## The finding

**Affinity from interaction has no diminishing return**, so friendship is a monotone function of exchange
count.

| | measured |
|---|---|
| friend-ties over 17 game-days | **25 → 115**, linear, never saturating |
| max affinity | **0.999 by tick 9 599** — the clamp, not an equilibrium |
| interactions to `Friend`, one pair | **45** |
| affinity after 57 interactions | **0.73** |

## Three hypotheses, all disproved by sweep

| knob | friends |
|---|---|
| `mere_exposure_per_day` 0.05 / 0.02 / 0.01 / 0.005 | **22 / 15 / 15 / 16** |
| `balance_rate_per_day` 0.5 / 0.05 | **22 / 25** |

My first reading was that exposure's equilibrium (0.833 against a 0.4 threshold) made proximity a friendship
pump. The sweep says exposure barely matters. What matters is that *each conversation adds affinity and nothing
subtracts proportionally* — so tripling contact triples friends, which is exactly how the last two iterations
broke the village.

## The citation error

`mere_exposure_per_day` cited **Zajonc 1968** as though the paper supplied the rate. Zajonc supplies the
mechanism — repeated exposure increases liking — and no number. Corrected, with the equilibrium and the
sweeps recorded in the doc.

## What I refuse to do

Change a rate to make a number look better. Sweeping exposure from 0.05 to 0.005 moves friends 22 → 16, and I
could have shipped 0.01 as "a fix" with no understanding of why it moved. The honest statement is that it
barely moves, and the cause is elsewhere.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **51** binaries |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| village after the change | friends **22**, rivals **33** — unchanged |

## Dead-code inventory

Zero.

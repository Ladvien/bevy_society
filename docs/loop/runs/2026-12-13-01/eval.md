# Eval

## Change

One harness flag (`--friend-affinity`) and a decision file with the measured answer. **No model change shipped.**

## What the measurement establishes

| claim | measured |
|---|---|
| the friendship bar is not the lever | 0.4 → 0.8 moves friends 22 → 16 |
| the clique is not a runaway | saturates at **122** friend-ties by day 24, then decays to 109 by day 57 |
| the asymptote is the lever | clamp ±0.35 turns 22/33 into **15/18** |
| the mood cost is the clamp, not the count | two different bars give identical mood **-0.078** |
| the first clamp formula was wrong | scaling by `1 − \|a\|/K` is asymmetric; rivals **rose 33 → 48** |

## The result

**First shape in six attempts to bound the network without inverting it:** friends **22 → 15**, rivals
**33 → 18**, both falling. At the strictest setting 4–6 friends, holding at three seeds.

**Cost:** mood **-0.054 → -0.078**, attributed to the clamp — affinity-gated activities see lower affinity, so
fewer contacts, so fewer `Liking` exchanges.

## What I got wrong

- Predicted a longer run would give a **bigger** clique. It saturates at 122 and decays. The measurement
  corrected a confident extrapolation.
- Wrote an **asymmetric** clamp (`delta · (1 − |a|/K)`), which let negative affinity reach −1 while positive
  decayed. Rivals rose 33 → 48 rather than falling. A plain clamp to `[-K, K]` fixed it.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **52** binaries (unchanged) |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| village after revert | friends **22**, rivals **33** |

## Dead-code inventory

Zero.

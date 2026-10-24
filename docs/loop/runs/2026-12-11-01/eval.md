# Eval

## Change

**None shipped.** Two hypotheses tested, both negative, reverted.

## Measured

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| `Friend` also needs trust ≥ 0.6 | 22 | 33 |
| contact talk 3 h + trust ≥ 0.35 | **267** | **0** |
| saturate affinity, trust and respect | 23 | 43 |
| saturate all three + contact talk | **268** | **0** |

## Why the trust gate failed

Measured at the end of a run:

| | affinity | trust | familiarity |
|---|---|---|---|
| friend-ties (115) | **0.792** | **0.480** | 0.907 |
| other ties (53) | −0.223 | 0.068 | 0.908 |

Trust *does* discriminate, sevenfold. But `chat` pays `trust: 0.05` per accepted exchange, so contact talk at
~8 a day adds **+0.4 trust/day** and trust saturates exactly as affinity does. A gate on trust is a gate on a
quantity the same mechanism supplies.

## Why saturation failed twice

It bounds at **1.0**, and every `classify` threshold is ≤ 0.4. Saturation changes how fast 1.0 is approached,
never whether 0.4 is crossed.

## The finding

**No tie variable can sit below a threshold for a pair that has had contact.** That is why five mechanisms have
failed — a `help` activity, a contact exchange, saturating affinity, a trust gate, and saturating everything —
and it is the first statement that explains all five rather than one.

## What is left

One untested shape: an asymptote **below** the threshold (`K < 0.4`), so no amount of contact with one person
suffices and friendship needs a second input. It is the only property none of the five failures had.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **51** binaries (unchanged) |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| village after revert | friends **22**, rivals **33** |

## Dead-code inventory

Zero.

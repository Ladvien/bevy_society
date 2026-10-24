# Plan: test the threshold and its inputs, and find the real cause

## Why this over the runner-up

`Doing`'s travelling semantics is escalated, so the top actionable item was last iteration's conclusion: the
threshold and its inputs. It was tested twice and both failed, which produced something better than a fix — a
structural account of why five mechanisms have now failed.

## What was done

1. **Measured what separates a friend from a non-friend.** Thresholds in `classify` gate on affinity (0.4) and
   familiarity (0.3). Measured: friends have affinity **0.792**, others **−0.223**; familiarity is **0.907 vs
   0.908** — no discrimination at all. Trust is **0.480 vs 0.068**, sevenfold.
2. **Gated `Friend` on trust** (0.6, then 0.35 with contact talk). Both negative: unchanged at 22, and 267 with
   contact.
3. **Found why**: `chat` pays `trust: 0.05` per exchange, so contact talk adds `+0.4 trust/day`. Trust saturates
   like affinity.
4. **Saturated all three accumulators.** 23/43 alone; **268/0 with contact**.
5. **Reverted.**

## Measured

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| trust gate 0.6 | 22 | 33 |
| contact talk + trust gate 0.35 | **267** | **0** |
| saturate all three | 23 | 43 |
| saturate all three + contact talk | **268** | **0** |

## The conclusion that ships

**No tie variable can sit below a threshold for a pair that has had contact.** Saturation bounds at 1.0, above
every threshold; every exchange supplies every tie variable; so any level-based gate is passed by contact
volume.

The untested lever is an asymptote **below** the threshold (`K < 0.4`), which is the property none of the five
failures had.

## Touched

- `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` — the failure table, the measured
  distributions, and the retraction of the trust-gate recommendation.
- `CHANGELOG.md`, `docs/loop/log.md`.
- **Reverted**: `src/behavior.rs`, `src/core.rs`, `src/relationships.rs`.

## New `Tuning` fields

None.

# Eval

## Change

**None shipped.** The recommended fix was built, measured in two configurations, and reverted.

## The disproof

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| saturation alone | **22** | 38 |
| contact talk at 3 h, no saturation | **267** | **0** |
| **contact talk at 3 h, with saturation** | **260** | **0** |

Saturation alone looks harmless (22 → 22). It does not survive contact: **260 against 267** is the same clique.
Rivals 0 in both.

## Why

Both curves pass the `0.4` Friend threshold, and both pass it within 45 exchanges. A saturating gain changes
**when** the threshold is crossed, not **whether**.

## The citation question

The saturating shape has **no source in the corpus**. The closest paper — Reiter et al. 2023 on attenuated gains
of trust — is about developmental trajectories, not diminishing returns per exchange. So the shape was a design
default, and testing it before shipping was the right order.

## What this iteration is worth

It prevents the next iteration from shipping saturation as a fix and finding out in a village. Three mechanisms
have now failed on this problem — a `help` activity, a contact exchange, and saturation — and the decision file
now says which shapes are untested rather than recommending the cheapest.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **51** binaries (unchanged) |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| village after revert | friends **22**, rivals **33** |

## Dead-code inventory

Zero.

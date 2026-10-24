# Eval

## Change

**None shipped.** The change was written, measured at four cooldowns, and reverted.

## What the measurement establishes

| claim | measured |
|---|---|
| the exchange is arrival-driven | yes — 30 exchanges in 20 000 ticks for two co-located people |
| contact exchanges can be added | yes — 89, with the cooldown verified tick by tick |
| at a rate that bridges the cluster | **no** — the village becomes 267 friend pairs, 97% of all pairs |
| a rate exists that does both | **no** — 48 h still triples the baseline |

## The finding

The mechanism conflates **meeting** with **befriending**. Two people who share a tavern are aware of each
other; they do not become friends by standing. The crate has one verb — `Interact` — that does both.

In a village everyone is always in contact, so a contact-triggered exchange is a firehose by construction. The
same mechanism that would let two isolated people meet makes twenty-four people a clique.

## Why reverting is the right outcome

A shipped version at any tested cooldown is a **worse** society: friends 267 / rivals 0 is not "more
sociable", it is a village with no disagreements at all — the same overshoot that made the `help` experiment
fail two iterations ago. Shipping it would trade a scenario that fails honestly for a model that fails
silently.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **50** binaries (unchanged) |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| village after revert | friends **22**, rivals **33** — the shipped numbers |

## Dead-code inventory

Zero. Nothing was left behind: the system, the field and the tuning entry are all reverted.

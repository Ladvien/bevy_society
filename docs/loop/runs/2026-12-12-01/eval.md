# Eval

## Change

Six `Tuning` fields replacing literals, and `TieLabel::classify` reading them. Defaults equal the old values.

## Before / after

| | before | after |
|---|---|---|
| `village-evening` series rows | 67 199 | **67 199 identical** |
| friends / rivals | 22 / 33 | **22 / 33** |
| literals in non-test code | **6** | **0** |

Bit-identical, which is the point: this is a *surface* change.

## Why it matters to a host

`friend_affinity` is now adjustable, and the test proves it is read: raising it to `0.9` demotes a tie from
`Friend` to `Acquaintance`.

Before, a host that wanted a colder village had to fork the crate. It also could not discover that
`friend_familiarity = 0.3` is satisfied by **0.907** for a friend and **0.908** for everyone else — the term
discriminates nothing, and that fact was only visible in the source.

## The README

Its claim — *"every rate is a named field on `Tuning`"* — is now true. It was false in two places.

## What I did not do

Change any value. Every default is the literal it replaced, and the sweep from previous iterations showed none of
them moves the village much (exposure 0.05→0.005 moved friends 22→16; balance 0.5→0.05 moved it 22→25). Making
them visible is the fix; choosing new values is the friendship decision's business.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **52** binaries |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |

## Dead-code inventory

Zero.

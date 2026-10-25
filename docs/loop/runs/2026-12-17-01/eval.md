# Eval

## Change

Contact gossip: two people who share a place talk when the speaker has news, gated on
`Knowledge::best_telling` and paced at `contact_gossip_cooldown_days = 2.0`.

## Before / after, same seed (`village-evening`)

| | friends | rivals | mood (P) | rumor reach |
|---|---|---|---|---|
| before | 22 | 33 | -0.0536 | 0.583 |
| **after** | **25** | **28** | **-0.0523** | **0.875** |

Reach **+50%**, rivals **down 15%**, mood **slightly better**. This is the opposite of the contact-`chat`
attempt, which took the village to 267 friend pairs.

## What the sweep establishes

The cooldown is not a taste knob. Reach is **non-monotonic** in it, because every telling to an informed
listener stifles the teller:

| cooldown | reach |
|---|---|
| 0.5 d | **0.375** (below baseline) |
| 2.0 d | **0.875** |
| 6.0 d | 0.583 (no effect) |

## The bug this nearly shipped

Omitting `theirs.knows(rumor)` — plausible, since `best_telling` already filters stifled spreaders — dropped
reach to **0.375**. That is a 35% regression in a scenario the change did not target, invisible in the friend
and mood summaries and visible only in the reach panel of `evidence/before-after.png`.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **54** binaries |
| `cargo test --no-default-features` | **54** binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 errors |
| `RUSTDOCFLAGS="-D warnings" cargo doc` | 0 errors |
| new tests bite (removal check) | 2 of 4 fail when the gate is disabled |

## Dead-code inventory

Zero: the new system is registered and exercised by four tests.

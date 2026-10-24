# Eval

## Crate change

`Obstacle` — a component a host attaches to anything round and solid. `steer_movers` feeds each to ORCA as a
hard wall half-plane, and `push_out_of_obstacles` resolves contact (which `bevy_orca` leaves to the caller).

Proven by a control:

| | closest approach to a block whose face is 1.5 m from its centre |
|---|---|
| no `Obstacle` | **0.01 m** — straight through |
| `Obstacle { radius: 2.0 }` | **2.35 m** — stopped at radius + body |

`tests/sleep.rs` also pins the second crate-level finding: 25 interrupted sleeps against 4 completed when the
whole village shared one `Home`.

## The six clips

| clip | before (as reviewed) | after |
|---|---|---|
| `village` | clusters standing still; HUD said 10/24 with no halos | rumor halos matching the counter, tie lines, people walking |
| `mood` | 24 identical dull rings | green / amber / red halos of differing size |
| `opinions` | all-green bars, wrong topic | red and green radial bars, differing length, on **honesty** |
| `needs` | 24 identical red halos | large green halos shrinking to red across the clip, on **food** |
| `ties` | two short segments, a line running off-frame | a readable graph, in frame |
| `factions` | two colours visible | orange, green and violet all in frame throughout |

Seven distinct faults were found and fixed; the table in `research.md` lists each with its cause.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **42** binaries |
| `cargo test --no-default-features` | **42** binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | 0 errors |

## Dead-code inventory

Zero. `scenes::draw_newest_rumor_rings` was **dead** — defined, correct, never called, while the buggy
`RumorId(0)` version ran. It is now the implementation.

## What I would not claim

The clips are 3.7–5.2 MiB each, up from ~2.6. That is the price of 240 full frames at 256 colours with a
burned caption. Halving with `stride=2` is available in `tools/gif.py` and would cost half the frame rate;
not taken, because a reader scrolling a README sees stills, and the stills are what this iteration fixed.

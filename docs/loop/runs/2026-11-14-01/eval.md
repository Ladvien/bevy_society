# Eval

## The claim, and the measurement

| | claimed since 2026-11-06 | measured 2026-11-14 |
|---|---|---|
| per-tick cost growth over a run | **4×** | **none** — 5.76 → 4.80 ms/tick across 1 000 → 5 000 ticks |
| rumors untellable | 93% | **87%** (301 of 2 250 tellable) |
| what the book costs | (implied: ticks) | **memory**, ~48 B/rumor: 105 KiB at 3 000 ticks, 228 KiB at 6 000 |

200 people, four places, a four-second game-day; five samples of 100 ticks each, once with no
`RumorReach` trigger and once with one. The trigger makes no difference, which is the point: 4 299
pointer-chases are nothing beside 200 people scored against their neighbours.

## What was decided instead of shipped

The retirement rule (`alive` cleared when no living spreader holds the rumor) is **not** shipped. It is
still the right fix for memory, and it is still a decision: `alive` is documented as "the subject is
still in the world", three places read it, and a host sees it. A 4× tick blowup would justify a breaking
semantic change; 5 MiB over a long large run does not.

## Tests

| test | result |
|---|---|
| `the_book_accumulates_rumors_nobody_can_tell` | `2250 rumors, 2250 alive, 301 still tellable` |
| `the_book_costs_memory_not_ticks` | `3000 ticks: 2250 (~105 KiB); 6000: 4865 (~228 KiB)` |

Both are slow — about 36 s together, because a village has to live twelve and twenty-two game-days to
have a book at all. That is the price of the measurement being real.

## A first draft that measured nothing

The helper started with `common::settings()`, whose day is 240 seconds. 3 000 ticks is then **half a
game-day**, and the first run reported `0 rumors, 0 alive, 0 still tellable` — a test that would have
passed on nothing. The helper now sets `seconds_per_day = 4.0` with a comment saying why.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **31** binaries (30 + `rumor_book`) |
| `cargo test --no-default-features` | 31 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. No library surface changed.

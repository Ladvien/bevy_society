# Eval

## The warning

| setting | metres a timeout buys | warned? |
|---|---|---|
| default: 2 h at a 240 s day | **28.0 m** | no |
| village: 40 h at a 4 s day | **9.3 m** | **yes** |
| village fixed: 55 h at a 4 s day | 12.8 m | no |

The startup line, verbatim from `tests/unreachable.rs`:

```
bevy_society: a travel_timeout_hours of 40 at seconds_per_day 4 allows 427 ticks, which at 1.4 m/s
is only 9.3 metres of walking — less than 10 m. People whose activity is further away will always
time out, and the needs those activities satisfy will never be met. Raise travel_timeout_hours or
shorten the walks.
```

That is a host-readable answer to a silent failure the crate had been producing.

## The measurement behind it

Walks timed directly, at `seconds_per_day = 4`, `tick_secs = 1/64`:

| walk | ticks measured | timeout allows at 40 h |
|---|---|---|
| 8.2 m | **308** | 427 ✓ |
| 13 m | **527** | 427 ✗ |

The village layout's diagonal is 13 m, so its far corner could never be reached — and the completion
counts say so: **78 of 108 `eat`s** and **44 of 108 `sleep`s** were interrupted, while **0 of 152**
social exchanges were.

## The harness change

| | before | after |
|---|---|---|
| `travel_timeout_hours` in `examples/evidence.rs` | 40 (427 ticks) | **55** (587 ticks) |
| `fun` at zero | 55% of ticks | **35%** |
| `fun` mean | 0.049 | 0.057 |
| `food` at zero | 28% | 22% |
| `rest` mean | 0.167 | 0.167 |
| `social` at zero | 4% | **11%** |

`evidence/timeout.png`. The plot is honest about the size of the win: `fun`'s floor rises and `rest` is
visibly better, but all four needs still hug zero, and **`social` got worse** — with fewer people stuck
walking, more of them reach places and fewer choose to talk. That is a real trade-off, not a defect, and
it is the strongest evidence that travel was one cause and not the cause.

## The blocker note

`need-cycle`'s note claimed a back-to-back sleep loop "with the shipped rates". Re-measured against the
starter pack, `food` and `fun` **do** cycle ([0.000, 1.000], both crossing 0.35 and recovering past 0.7);
`social` and `rest` never fall below 0.364 and 0.495 because they are served too well to starve. The old
note is kept as `superseded:` so the record of the mistake survives.

The scenario stays `pending`: its observable demands all four needs cycle, and two do not.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **27** binaries (26 + `unreachable`) |
| `cargo test --no-default-features` | 27 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. One private function and one private constant added, both reached from `finish` and asserted by
three tests.

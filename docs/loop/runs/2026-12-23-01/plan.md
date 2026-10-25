# Plan: name the home that cannot seat its residents

## Why this over the runner-up

Last iteration's blocker instruction was to instrument the solver. Doing that found the mechanism (median six
bodies already in the goal disc, which is exactly the declared capacity) and simultaneously showed that **every**
fix trades interruptions for rivalries. So the runner-up — another lever — was measured and rejected three more
times; the shippable thing is the diagnosis.

## The instrument

At every timed-out `sleep`, count bodies within `interaction_radius` of the goal:

| in the disc | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|
| timed-out sleeps | 1 | 1 | 8 | 40 | **55** | 36 | 4 | 2 | 3 |

Median **6**, and the homes declare `capacity: 6`. The disc is full at its declared capacity; the failures are the
queue behind it. `(1.5 / 0.7)² ≈ 4.6` bodies is what a 1.5 m disc actually seats, so six residents already do not
fit.

## Change

`warn_unreachable_places` — already a `Startup` system that measures the layout — now also counts residents per
home and warns when a home cannot seat them, naming the place, the residents, the seats, and the radius that would
work.

## Touched

- `src/behavior.rs` — the resident count and the warning.
- `tests/crowded_home.rs` — one test, both halves.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## The three levers that were tried and rejected

| lever | interrupted | rivals |
|---|---|---|
| crowd allowance 0.35 m/person | 349 | **59** |
| crowd allowance 0.70 m/person | 290 | **51** |
| physical gate `r + d·√n` | 316 | **61** |

All reduce interruptions 21-28% and raise rivalries 13-23%. The mechanism is **time in contact**, not contact
radius (4.0 m, versus a ~2 m arrival disc): arriving earlier means the activity starts earlier and runs its full
duration standing still, instead of walking. A real behaviour change, and a cost the host should choose.

## Verification

| check | result |
|---|---|
| warning fires on the project's own village | names `'well'` and `'tavern'`, 8 residents, 4 seats |
| warning stays quiet when the home fits | asserted in the same test |
| bite check (condition disabled) | the test **fails**, printing an empty capture |
| `cargo test` both feature sets | **62** binaries |

One test, not two: `tracing` allows a single global subscriber per process, and a second test in the same binary
would read an empty string.

# Eval

## The gap

`Signal::Distance` returned `1.0` — the maximum, "as far as it can see" — for **every** `Place` and `Point`
target, because `PlaceView` carried no position.

`Linear(-1, 1)` at 1.0 is exactly 0.0, and a zero consideration zeroes the whole option. So the natural line
does not price travel; it disables the activity:

| | `food` mean | at zero |
|---|---|---|
| no `Distance` on `eat` | **0.402** | 0% |
| `Distance Linear(-1, 1)` | **0.034** | **93%** |

## The fix, measured

Both taverns in one run — 1 m and 7 m, 6 000 decisions:

| | near | far |
|---|---|---|
| before | **0** | **0** |
| after | **5 999** | **0** |

and `eat` with `Distance` in the village: `food` mean **0.034 → 0.176**, at zero **93% → 0%**.

## Behaviour with the shipped pack: unchanged

All **67 199** rows of `village-evening`'s series are identical, because nothing in the pack asks for
`Distance` on a `Place`. Travelling stays 58.9%, needs 0.545. A capability fix.

## What it hands on

`food` at 0.176 is still below its 0.402 baseline, because the pack's curve is `Linear(-1.0, 1.0)` and a
tavern 6.5 m away inside an 8 m radius scores **0.19**. The signal is now correct; the curve is
`needs-decision/travel-cost-is-not-scored.md`'s question.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **39** binaries, 0 failures |
| `cargo test --no-default-features` | 39 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `PlaceView::position` and `Decision::place_positions` are read by the signal and asserted by the test.

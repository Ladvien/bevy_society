# Eval

## Change

`need-cycle`'s observable restated from measurement, and its status moved from `pending` to **`measured`** for
the first time since November. The test's name corrected, and the restated property asserted.

No crate code changed.

## Before / after

| | before | after |
|---|---|---|
| status | `pending` (since November) | **`measured`** |
| observable | "`rest` and `food` cross below 0.35 and above 0.7" | "every need is met and re-met; none left falling with no source" |
| `rest` crosses 0.35? | **never, at any seed** | not claimed |
| test name | `only_food_cycles_...` | `food_and_fun_cycle_and_rest_never_does` |
| `fun` cycles | printed, unasserted, **8 of 8** | printed and explained |

## Why `rest` cannot cycle — the arithmetic

```
sleep  0.15/game-hour × 8 h = 1.20 restored
rest   decay_per_day        = 0.70 lost
                              +0.50 net per sleep
```

`rest` ratchets up until sleep stops being chosen. The observable assumed a need that starves; this one is
over-supplied by 1.7×. `food` is exactly break-even (`1.2 × 0.75 h = 0.90` against `0.9/day`), which is why it
oscillates; `fun` is close behind.

## What the restated observable checks

Every need must reach above 0.7 for every person. That catches the defect the original was reaching for — a
need drained with no activity that satisfies it — without asserting a starvation a generous rate legitimately
prevents.

## What I refuse to do

Retune `rest`'s decay to make the old claim true. The decay is cited (Maslow 1943), the rates are **data** a
host sets, and five iterations have changed the pack since the claim was written. Fitting the model to a stale
assertion is the opposite of keeping the crate honest about where its numbers come from.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **49** binaries |
| `cargo test --no-default-features` | 49 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.

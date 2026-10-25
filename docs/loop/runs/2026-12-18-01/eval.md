# Eval

## Change

`Doing::is_travelling()` and `Doing::label(&SocietyDefs)`, so the component answers "what is this person doing
**now**" without every consumer re-deriving it from `arrived_day`.

## Behaviour is unchanged, and that is the claim

| artifact | before vs after |
|---|---|
| `village-evening` `series.csv` | **byte-identical** (`cmp`) |
| `village-evening` `summary.json` | identical (sorted-JSON diff) |

The helper is a faithful extraction of a workaround the evidence harness already contained. Nothing about the
simulation moved; what changed is that a host now has a correct reading available on the component instead of
having to know to check a second field.

## What the old reading was getting wrong

Tallying `Doing.activity` raw over the same run:

| state | share of person-ticks |
|---|---|
| sleep | 36.8% |
| work | 22.9% |
| eat | 13.3% |
| compliment | 8.7% |
| gossip | 5.1% |
| snub | 5.0% |
| relax | 4.5% |

One 8-hour sleep a day is 33.3%, and the raw figure is 36.8% — the 3.5-point gap is people walking to bed while
labelled asleep. With `label`, that part reads `travelling:sleep` and the tally is honest.

The distribution itself is believable, which was worth checking: nothing is saturated or degenerate, and travel
is only 1.0% of person-ticks in the village.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **55** binaries |
| `cargo test --no-default-features` | **55** binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 errors |
| `RUSTDOCFLAGS="-D warnings" cargo doc` | 0 errors |
| new test bites (prefix deleted) | **1 of 3 fails**, the walking one |

## Dead-code inventory

Zero.

## Not established

The scale curve in `docs/guide/concepts.md` could not be verified: the machine ran at load 10.2 on 10 cores
throughout, and even the 100-person case — which does no perception work — reads 1.6x above the documented value.
Before/after pairs agree to within 1%, so the *change* costs nothing, but the guide's absolute numbers need a
re-measurement on an idle machine.

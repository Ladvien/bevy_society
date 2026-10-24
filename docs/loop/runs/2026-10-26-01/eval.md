# Eval

## Before / after, same seed, same scenario

Measured over 4200 ticks of `village-evening` at seed 7, definitions differing only in the
two `satisfies` lines this iteration added
(`evidence/village-evening-before/` vs `evidence/village-evening/`):

| need | before | after |
|---|---|---|
| social | **0.105** | **0.225** |
| rest | 0.218 | 0.200 |
| food | 0.048 | 0.056 |
| fun | 0.041 | 0.046 |

## The measurement that chose the change

Rise and fall events per need, counted tick by tick over the same run before the change:

```
social: rises=420  falls=20365  ratio 0.021
```

One rise per ~80 ticks per person where the 1.0/day decay needs one per ~51 to break even:
the need was 1.57x short of steady state, which is why it sat at the floor.

## Dead-code inventory

Unchanged at zero: `cargo clippy --all-targets --all-features -- -D warnings` passes with
nothing unused.

## Test counts

| Command | Result |
|---|---|
| `cargo test` | 16 binaries ok |
| `cargo test --no-default-features` | 16 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |

## The capture

Unchanged within noise: `{"rumor_reach": 0.640, "friends": 252, "rivals": 0, "barks": 3376}`
against 0.800 / 256 / 0 / 3076 last iteration. The scenario's social dynamics changed, so a
different draw path is expected; all thresholds except `rivals` hold.

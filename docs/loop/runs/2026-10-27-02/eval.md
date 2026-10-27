# Eval — a rivalry held through a band

Iteration `2026-10-27-02`. One clause in `TieLabel::classify_from`, one new `Tuning` field. The clause is in,
the default came from a sweep, and the numbers below are before and after at the same seed and the same pack.

## The defect, measured before it was fixed

A `Rival` was released the instant its affinity rose above `-rival_affinity` (0.30). Pair-level measurement at
`village-evening` settings, three seeds, 12 000 ticks (2.93 game-days), `tests/zz_scratch_rival_spells.rs`
(deleted after this run):

| seed | rival spells | median spell | **margin above the bar at release** |
|---|---|---|---|
| 7 | 41 | **0.289** game-days | median **0.0002**, p90 0.0862 |
| 42 | 92 | **0.375** game-days | median **0.0001**, p90 0.0003 |
| 11 | 16 | **0.141** game-days | median **0.0001**, p90 0.0004 |

Released with `familiarity_peak < 0.1`: **0 of 149**. The affinity at release was one arithmetic step above the
bar, so the label was being made and unmade by relaxation and by the next accepted `compliment`, not by anything
that happened between the two people.

## The sweep that chose the default

`rival_band` in {0.0, 0.05, 0.10, 0.15}, `village-evening`, three seeds, 17.62 game-days, same pack:

| band | seed | friends | rivals | became rival | left rival | ticks with no rivalry |
|---|---|---|---|---|---|---|
| 0.00 | 7 | 25 | 2 | 47 | 42 | 46.3% |
| 0.00 | 11 | 21 | 0 | 19 | 19 | 79.3% |
| 0.00 | 42 | 19 | 2 | 43 | 38 | 59.1% |
| 0.05 | 7 | **25** | 4 | 34 | 25 | 29.0% |
| 0.05 | 11 | **21** | 1 | 12 | 9 | 39.8% |
| 0.05 | 42 | **19** | 7 | 32 | 18 | 35.5% |
| **0.10** | 7 | **25** | **7** | 27 | 13 | **13.3%** |
| **0.10** | 11 | **21** | **1** | 11 | 8 | **20.6%** |
| **0.10** | 42 | **19** | **8** | 28 | 11 | **18.3%** |
| 0.15 | 7 | **25** | 8 | 25 | 9 | 13.3% |
| 0.15 | 11 | **21** | 3 | 11 | 5 | 3.8% |
| 0.15 | 42 | **19** | 8 | 27 | 10 | 7.5% |

**Friend pairs are identical (25 / 21 / 19) at every band.** The clause guards the negative category and touches
nothing else, and that is the strongest available check that the change did not move the network — the village is
friend-dominated, and a change that moved both ends would have moved the network rather than the asymmetry.

**0.10 is the chosen default**, and the reason it is not 0.15 is arithmetic from the sweep: at 0.15 a rivalry
needs affinity to climb from −0.30 to −0.15, which is 1.5× `affinity_ceiling` (0.35) of travel, and the
released-count falls to **9 / 5 / 10** — a village where grudges mostly never end. 0.10 halves the releases while
keeping reconciliation at every seed, and takes "no rivalry anywhere in the village" from **46 / 79 / 59%** of
ticks to **13 / 21 / 18%**.

## Before → after, at the default

Same pack, same seeds, 17.62 game-days, `evidence/before/` (band 0.0) and `evidence/after/` (band 0.10):

| | seed 7 | seed 11 | seed 42 |
|---|---|---|---|
| friends | 25 → 25 | 21 → 21 | 19 → 19 |
| rivals at the end | 2 → **7** | 0 → **1** | 2 → **8** |
| rivals averaged over the run | 1.13 → **2.98** | 0.21 → **1.04** | 0.66 → **2.57** |
| peak rival pairs | 4 → **7** | 1 → **2** | 4 → **8** |
| ticks with no rivalry | 46.3% → **13.3%** | 79.3% → **20.6%** | 59.1% → **18.3%** |
| `became_rival` | 47 → 27 | 19 → 11 | 43 → 28 |
| `left_rival` | 42 → **13** | 19 → **8** | 38 → **11** |
| label messages | 1 085 → 1 036 | 1 053 → 1 034 | 1 090 → 1 048 |

`became_rival` falls as well as `left_rival`, which is the signature of the mechanism: fewer are *re-made*,
because the ones that form are still there. `left_rival` stays above zero at every seed, so the band did not make
a grudge permanent.

Pair-level, seed 7 at band 0.10, same 12 000-tick window as the defect table:

| | before | after |
|---|---|---|
| rival spells | 41 | **18** |
| median spell | 0.289 game-days | **2.637 game-days** |
| margin above the bar at release | median **0.0002** | median **0.1002** |

The release margin is the number that matters: it moved from "one arithmetic step" to "half the band", which is a
real change between two people rather than a rounding.

## The instrument was wrong, and finding it was not optional

While reading the run's narrative, every interaction line was missing. The harness drained `InteractionResolved`
and `ActivityEnded` into its **counters** before `drain_narrative` read them, so the file showed

```
d1 21h30: started snub
```

with no target and no outcome — on the iteration whose entire subject is which pairs are rivals. Both messages
are now drained once into a `Vec` and read by both consumers, and the same run reads

```
d1 21h30: started snub
d1 21h30: insult with Garet (accepted)
```

The before/after numbers were regenerated after the fix and are **identical** to the pre-fix run, so the counters
were never wrong — only the narrative was, and the narrative is what a host developer reads first.

## Gates

| Command | Result |
|---|---|
| `cargo test` (both feature sets) | see `perf.md` / log entry |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -D warnings` | clean, dead-code inventory still zero |
| `cargo doc --no-deps --all-features` with `-D warnings` | clean |

## Deliberately not changed

1. **`friend_band` (0.05)** keeps its measured default. The rival band is wider because the shelf a rival can
   occupy is wider: `friend_affinity` 0.30 against `affinity_ceiling` 0.35 is 0.05 of headroom, while a rival has
   the whole span from −0.30 to 0 before it is not a rival at all.
2. **`label_hysteresis` (0.02)** still governs only the `Stranger` boundary. It is consulted after the
   `promoted != Stranger` early return, so it cannot affect a `Rival`; widening it was one of the three rejected
   shapes in `plan.md`.
3. **The affinity value itself.** No relaxation rate, no exchange weight, no clamp moved. Every one of those is
   read by the opinion model as an influence weight, and run `2026-12-28-01` measured that changing them re-times
   opinion convergence. A band changes a label and nothing else.

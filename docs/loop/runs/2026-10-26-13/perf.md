# Performance — `DefsFile::merge` carrying two more lists

The change is two `check_disjoint` scans and two `Vec::extend` calls **at compile time**, on a path that already
does the same for nine other lists. It runs once, before the app exists, and touches no per-tick code. The
benchmark cannot distinguish it, and the measurement below is recorded to say so rather than to claim anything.

`examples/benchmark --people N --ticks 200 --reps 3`, same machine, both builds interleaved. Before build from a
worktree at `d825bdd`, after build from the working tree.

| persons | before (ms/tick median) | after (ms/tick median) | before spread | after spread |
|---|---|---|---|---|
| 100 | 2.255 | 2.171 | 4.9% | 37.1% |
| 1 000 | 74.823 | 63.733 | **56.9%** | 49.3% |

**The 1 000-person spread (56.9% before, 49.3% after) is larger than the difference between the builds**, which is
the machine, not the change: this laptop was at load average 21–39 from other work for the whole iteration, and an
earlier pass on it reported 55% and 126% spreads at the same sizes. The after build is nominally faster at both
sizes and **no speedup is claimed** — two compile-time `Vec::extend`s cannot be worth 15%. The 100-person figures
(2.255 vs 2.171, a 4.9% before spread) are the closest to a clean comparison and differ by 3.7%, which is still
inside the noise of a change that runs before the first tick.

For reference, the numbers in `docs/loop/scenarios.yaml` (`scale-1000`: 13.5 ms at 1 000, 0.59 at 100) were taken
on an unloaded machine and are not comparable to these — they are quoted here only so a reader does not mistake
the 63.7 above for a regression in this change. That run and this one differ by the machine's load, not by code.

**A defect this measurement surfaced and did not fix.** `examples/benchmark` builds its society with one `Home`
place seating an arrival disc of radius 1.5 — about four people — for *every* population, so at 100 and 1 000
people the harness warns `'place0' is home to 1000 people, but an arrival disc ... seats about 4`, and every
sleep in the benchmark is interrupted. The per-tick cost is then the cost of a society whose people cannot go to
bed, which is a different workload from the one `scenarios.yaml` describes. It is recorded rather than fixed
because changing the benchmark's fixture changes every number in this file and in `concepts.md`, and that is a
measurement programme of its own — the same trap `--people` scaling has already set three times.

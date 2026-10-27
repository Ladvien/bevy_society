# Performance — one comparison inside a label pass

The change is one extra comparison inside `TieLabel::classify_from`, which already runs two clauses before it
and is called once per tie per tick. No allocation, no draw, no iteration order. The number below is recorded to
say the change is not visible at this scale, not to claim a speedup.

`examples/benchmark --people N --ticks T --reps 3`, both builds interleaved on the same machine, three rounds.
Before build is a worktree at `cd9b10e` (the research commit, `src/` untouched by this iteration).

| persons | round | before median | after median | before min | after min | spread (before / after) |
|---|---|---|---|---|---|---|
| 100 | 1 | 0.951 | 0.987 | 0.893 | 0.879 | 22.2% / 24.8% |
| 100 | 2 | 0.974 | 1.029 | 0.900 | 0.926 | 26.6% / 23.3% |
| 1 000 | 1 | 39.452 | 42.169 | 29.841 | 30.355 | 35.4% / 35.6% |

**The spread is larger than the difference between the builds at both sizes** (22–27% at 100 people, 35% at
1 000), so neither figure distinguishes them. The `min` columns are the closest comparison available — a fast
window is evidence the machine was momentarily quiet — and they differ by **1.6%** at 100 people and **1.7%** at
1 000, in opposite directions across rounds.

The machine was under load from other work for the whole iteration (load average 20–40 on 24 cores), which is why
the spreads are this wide; `docs/loop/runs/2026-12-26-01/eval.md` measured **100%** spreads on the same laptop at
1 000 people and concluded that only a fast window is quotable.

**A defect this measurement walked past, recorded and not fixed.** `examples/benchmark` builds one `Home` place
seating an arrival disc of radius 1.5 — about four people — for *every* population, so at 100 and 1 000 people the
harness warns `'place0' is home to 1000 people, but an arrival disc ... seats about 4` and every sleep is
interrupted. The workload is therefore a society whose people cannot go to bed, which is not the workload
`scenarios.yaml` describes. Changing the fixture changes every number in this file, `concepts.md` and the log, so
it is a measurement programme of its own — the same trap `--people` scaling has set three times in this
repository. It is recorded at `docs/loop/runs/2026-10-26-13/perf.md` as well, by the pass that found it.

## What would have moved if the change were expensive

`refresh_label` is called from `relabel` for every tie of every person each tick, so an expensive clause would
show at 1 000 people first: 1 000 people × ~150 ties is where a per-tie cost multiplies. It does not — the 1 000
figure differs by less than the spread.

## Not measured, and why

- **5 000 people.** The 2026-12-26 run found 5 000 does not complete inside a sensible deadline on this machine
  under its load, and a timeout is not a measurement.
- **The band's own value.** A wider band costs nothing extra: the clause is the same comparison whatever the
  number is.

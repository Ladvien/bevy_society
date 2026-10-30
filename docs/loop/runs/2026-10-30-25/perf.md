# Perf: `Tuning::forgiveness`

Command: `cargo build --release --example benchmark` at HEAD `c95eac7` (src only, checked out into the clone, built, restored) and at this branch; then for n in 100 1000 5000, before then after: `benchmark --people $n --ticks 50 --settle 100 --reps 3`. Host: 24 threads, load average 39 at the start and 82 at the end.

| people | before median ms/tick (min-max) | after median ms/tick (min-max) | spread % before / after |
|---|---|---|---|
| 100 | 1.578 (1.465-1.670) | 1.120 (0.947-1.208) | 13.0 / 23.3 |
| 1000 | 39.156 (33.812-60.594) | 30.468 (29.469-49.125) | 68.4 / 64.5 |
| 5000 | 828.222 (799.215-1460.517) | 883.829 (843.429-1373.549) | 79.8 / 60.0 |

Every spread but one exceeds 20%, so no difference here is a measurement: the 100 and 1000 rows are faster after and the 5000 row 7% slower, within the run-to-run swing. The code adds one tie binary search and a multiply to each `ctx.grudge` call (one per `snub` candidate scored), and the society it produces is different (fewer snubs, more compliments), so a change of cost would come from the behaviour, not the lookup. No scenario this change did not target is slower beyond the noise.

# Perf: `ActivityDef::urgent`

**Verdict: no measurable cost.** On a pack without `urgent` (the starter pack, and the benchmark) the only code added to the decision
is `ActivitySpec::rank_for`, one `match` on an `Option` per option that scored, per decision (`None` returns `spec.rank`); the
default path is bit-identical to HEAD (`eval.md`, `cmp` over a 17-game-day run). It allocates nothing. The decision trace, when a
host turns it on, carries one more `u8` per option.

Command (release, `examples/benchmark`, the starter pack, 24 threads, host load average 34-48 throughout, other loops running):
`bench-before` is `6e2947e` (the merge of pass 28), `bench-after` is this branch, both built into the clone's own `target/` from
a worktree at `6e2947e` that was removed after the runs; pairs alternate, and the order is swapped between pairs to cancel any
first-runs-second effect.

```sh
./bench-{before,after} --people N --ticks 200      # five reps each; the min is the fastest window
taskset -c 5 ./bench-{before,after} --people 100 --ticks 400
```

| people | threads | before: fastest window per run (ms/tick) | after: fastest window per run (ms/tick) | host spread |
|---|---|---|---|---|
| 100 | 24 | 1.35 1.48 1.51 1.82 2.25 2.45 (6 runs, 400 ticks) | 1.61 1.84 1.88 1.98 2.09 2.16 | 45-110 % |
| 100 | 1 core (`taskset`) | 0.91 0.98 0.98 1.03 1.07 1.32 | 0.96 0.98 1.03 1.03 1.10 1.13 | 87-121 % |
| 1 000 | 24 | 57.1 54.8 52.7 44.8 (4 runs, 200 ticks) | 48.8 51.7 44.0 53.7 | 56-77 % |

At 24 threads on a loaded host the 100-person windows overlap (before 1.35-2.45 ms, after 1.61-2.16 ms) and the 1 000-person windows
favour the branch in three of four pairs: the spread is larger than any effect. Pinned to one core, where wall time is close to CPU time,
the two have the same median (1.03 ms/tick) and the same range. No 5 000-person run: the path is identical and the host's other jobs
would dominate it (pass 28 reached the same conclusion).

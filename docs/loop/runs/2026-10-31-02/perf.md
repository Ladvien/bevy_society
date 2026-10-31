# Perf: body clocks

**Verdict: no measurable cost; the host cannot resolve less than about 10 % at 5 000 people.**

What the change adds to a society tick, for a pack with an `unavailable` activity (the starter pack's `sleep`, so the benchmark has it):

- once per tick, one pass over `PersonIndex.sorted`: a `Query<&Doing>` lookup and a `bool` per person (`unavailable_slots`), so 5 000 lookups, about 0.2 ms of a ~530 ms tick at 5 000 people (0.04 %); nothing for a pack with no `unavailable` activity (the `Vec` is empty and never read);
- per decision: a `Chronotype` read and one subtraction (`body_hour`), and one index read per neighbour in place of nothing.

It allocates one `Vec<bool>` per tick. The first implementation queried each neighbour's `Doing` inside the per-person loop (up to the population times its neighbourhood in a society where everyone re-decides every tick, which the benchmark's four-second day does); the slot lookup replaced it before the final measurement (`848b9df`; the harness's outputs are byte-identical across the two).

Command (release, `examples/benchmark`, the starter pack at `seconds_per_day` 4, `before` = `773ba21` built from a worktree into the clone's own `target/` and removed afterwards, `after` = this branch). Host load average 9-50 throughout, other jobs running; the benchmark is effectively single-threaded (CPU time equals wall time), so CPU seconds are quoted beside the ms/tick medians. Orders alternate (ABBA) so first-runs-second and load drift cancel as far as they can.

```sh
taskset -c 5 ./bench-{before,after} --people 100 --ticks 400      # six pairs
./bench-{before,after} --people 1000 --ticks 200                  # three pairs, then CPU seconds, 8 runs of 100 ticks
./bench-{before,after} --people 5000 --ticks 20                   # ABBA
```

| people | before | after | note |
|---|---|---|---|
| 100, one core | median windows 1.80 1.85 2.13 1.76 1.93 1.79 ms/tick; fastest 0.90-1.34 | 1.98 1.80 2.04 1.80 1.87 2.08; fastest 0.98-1.39 | host spread 67-131 %; the ranges overlap |
| 1 000, 24 threads (first build) | median 68.1 70.3 69.8; fastest 44.1 46.8 49.4 | 94.3 77.7 64.9; fastest 50.2 48.4 35.8 | load 45-50, spread 60-98 % |
| 1 000, CPU seconds for 100 ticks × 5 reps (final build) | 30.2 37.1 37.6 31.4 | 31.1 30.2 35.3 33.2 | after is the lower mean (32.5 against 34.1); load 9-27 |
| 5 000, load 8-10 (first build, ABBA) | median 526.0 530.3 | 547.2 523.1 | spread 13 %; +1.3 % on the means |
| 5 000, CPU seconds for 20 ticks × 5 reps (final build, ABBA) | 290.6 (load 12.6), 316.8 (load 22.1) | 320.5 (load 28.2), 325.2 (load 19.8) | the two `before` runs differ by 9 % with the load; the `after` run at load 19.8 (682.6 ms/tick) sits beside the `before` run at load 22.1 (706.0) |

Two runs at 5 000 people that looked like +28 % (800 against 624 ms/tick, and 929 / 770 against 608 / 610) were taken while the host's load average moved from 12 to 45 between the runs of a pair; the pairs at matching load are level. A cleaner figure needs a quiet host: the loop's table in `docs/guide/concepts.md` is unchanged.

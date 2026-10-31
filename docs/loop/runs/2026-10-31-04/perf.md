# Perf: a week

**Verdict: no measurable cost.** The host resolves about 5 % between identical binaries at best; both arms read the same.

What the change adds to a society tick, for a pack that never writes `Week` (the default):

- **Once per tick** in `decide_activities`: `clock.week_position(settings.start_weekday)` (one modulo of the game-day count and a division) and one `f32` field in each person's `Decision`. In `snapshot`: one `i64` field.
- **Per consideration:** nothing. `signal_value` gains a match arm, and the compiled `Signal` for a pack without `Week` never reaches it.
- **Load time:** one pattern test on each `Days` curve.

A pack that does use `Week` pays what `Hour` costs: one division per consideration that reads it.

Command (release, `examples/benchmark`, the starter pack at the benchmark's own `seconds_per_day` of 4; `A` = a binary built from `main` (`8bfdb25`), `B` = this branch; user CPU seconds of the whole process, the three reps' median ms a tick, host load 8-12 at the start of each run):

```sh
./bench --people 1000 --ticks 100 --reps 3        # order A B B A
```

| arm | median ms/tick (min-max of 3 reps) | process user s | load |
|---|---|---|---|
| A (main) | 35.6 (27.9-41.9) | 19.15 | 11.8 |
| B (this branch) | 36.4 (28.9-42.5) | 19.61 | 10.3 |
| B | 35.2 (27.7-41.4) | 19.03 | 9.1 |
| A | 35.3 (27.6-41.8) | 19.00 | 8.5 |

A: 35.6 and 35.3 ms, 19.15 and 19.00 s. B: 36.4 and 35.2 ms, 19.61 and 19.03 s. The arms overlap completely (within-arm spread 39 % of the median, between-arm difference under 2 %). The harness runs above (`evidence/`) are the other evidence that the default society is untouched: byte-identical output at seed 7 (`eval.md`, section 1), so the work is the same work.

Not measured: 5 000 people (the change is per tick and not per person; nothing in it scales with the population).

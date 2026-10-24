# Perf

The walker is one loop over the people who have a `Destination`, once per fixed step. Its work per
walker is a `Vec2` subtraction, a length, a scalar divide and a write.

Measured in release, `--ticks 200`, before this iteration's change against after:

| Persons | ms per tick, before | after | delta |
|---|---|---|---|
| 100 | 0.556 | **0.564** | +1.4% |
| 1 000 | 15.16 | **15.02** | −0.9% |
| 5 000 | 277.0 | **275.5** | −0.5% |

The two larger figures are *faster*, which is noise, not a speed-up: run-to-run variation on this
machine is about ±1%. The honest reading is that the walker is **not measurable** against the
perception pass it runs beside.

That is what to expect: the walker touches only the people who are travelling, and its per-person work
is a handful of floating-point operations, while perception compares every person against every
neighbour within 8 m. It runs on **every fixed step** rather than only on society ticks, which is the
reason to measure rather than assume — at `seconds_per_day = 4` that is one walk per tick, and it
still does not register.

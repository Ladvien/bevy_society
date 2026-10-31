# Perf: rebuff memory

**Verdict: no measurable cost; switched on it is not slower, and the host cannot resolve less than about 10 %.**

What the change adds to a society tick:

- **Off (`None`, the default):** one `Option::is_some` test per activity per person in `decide_activities` and one extra empty `Query` in `DecisionQueries`. The memory is never read or written, and the society is the previous build's byte for byte (`eval.md`, section 1), so the work is identical.
- **On (`Some(h)`):** per person per non-harmful `Person` activity, one `Query<&Rebuffed>` lookup and, when the person has been refused in the last `h` hours, a `retain` over the targets against a list of about two entries (1 569 refusals over 24 people and 17.6 game-days is 3.7 a person-day, so about two in twelve hours). A write per refused exchange: one clone of that list, a prune and a push. No allocation per tick for a person who has no memory.

Command (release, `examples/benchmark` with the new `BENCH_REBUFF=<hours>` switch, the starter pack at `seconds_per_day` 4, **the same binary** for both arms, so there is no baseline build to differ; `A` = off, `B` = `BENCH_REBUFF=12`). CPU seconds (user time of the whole process, the benchmark being effectively single-threaded), ABBA order, host load average 15-48 throughout (other jobs and my own test gate running):

```sh
BENCH_REBUFF=12 ./bench --people 1000 --ticks 100 --reps 5      # B;  without the variable: A
./bench --people 5000 --ticks 10 --reps 3                        # A, with BENCH_REBUFF=12: B
```

| people | A (off) | B (on, 12 h) | note |
|---|---|---|---|
| 1 000, 100 ticks × 5 reps, CPU s | 44.0 43.0 42.3 42.2 (mean 42.9) | 47.1 47.1 36.6 33.4 (mean 41.1) | order A B B A, A B B A; load 26-48; the arms' ranges overlap (33-47 against 42-44) |
| 5 000, 10 ticks × 3 reps, CPU s | 294.5 282.8 (mean 288.7) | 273.6 273.4 (mean 273.5) | order A B B A; load 16-20 |

**Reading.** The on arm is 4-5 % lower on the means, which is not a speed-up the code can claim: with the memory on people approach 5-11 % fewer times a day (`eval.md`), and an exchange costs more than a decision does, so the saved work plausibly outweighs the filter. The ranges overlap at 1 000 people and the host's spread is 29-64 % per window (`ms_per_tick_median` 43.2-75.6 for the same arm), so the figure says only that there is no cost to see. The loop's scale table in `docs/guide/concepts.md` is unchanged.

# Perf: a starting larder

**Verdict: no cost the host can see; the measurement cannot resolve less than ±25 %.** The code change adds one `Res`
and one closure to a system that already walks every person, and allocates only on the tick a person is created. The
benchmark measures a *society*, and the larder changes what the society is doing in the measured window, so it moves
the figure for reasons that are not the code.

Command (release, one binary per side built from the same dependency set; before = `3ebb18b`, whose `src/` is `48e2260`'s,
after = this branch; the host ran at load 29-54 throughout, 24 threads, with other jobs of mine and others'):

```sh
./bench-{before,after} --people N --ticks T --settle S --reps R   # examples/benchmark, the starter pack
```

Two rounds, before and after interleaved. `median` is the median over reps of ms per society tick; `spread` is
(max − min) / median.

| people | T / S / R | before median (min), round 1 / 2 | after median (min), round 1 / 2 | spread % (before; after) |
|---|---|---|---|---|
| 100 | 100 / 300 / 5 | 1.175 (0.918) / 1.213 (1.003) | 1.884 (1.252) / 4.327 (2.033) | 40, 34; 48, 96 |
| 1 000 | 100 / 300 / 5 | 72.7 (51.4) / 64.7 (50.8) | 54.0 (42.1) / 58.6 (43.8) | 46, 28; 52, 39 |
| 5 000 | 30 / 100 / 3 | 1 124 (601) / 1 105 (629) | 1 184 (844) / 1 023 (734) | 47, 48; 30, 70 |

Every spread exceeds the 20 % line, so these are upper bounds and the sign of any difference is not established. The
100-person row read slower after, so it was chased:

- **A longer settle (S = 3 000, three pairs, 100 people):** before 4.65 / 3.55 / 3.93 ms, after 5.20 / 5.71 / 4.18 ms
  (spreads 18-49 %). Still slower after in every pair.
- **A control that removes the code from the comparison:** one binary (this branch) run with the starter pack's bread
  `initial` at its default 1.0 and at 0.0 (a throwaway `BENCH_NO_LARDER` switch in `examples/benchmark.rs`, reverted
  before the commit), three pairs: **4.97 / 4.69 / 4.20 ms with the larder, 5.61 / 4.29 / 3.32 ms without**. Same code, same
  cost, and the difference of two runs of one binary (up to 1.3 ms) is as large as the before/after gap.
- So the larder's effect on the 100-person benchmark is not distinguishable from the host's noise. A fed society is
  busier (the evidence runs show 4-10 % more accepted compliments and 9-21 % more insults by day 17,
  `evidence/summary.md`), and `[INFERENCE]` the cost per tick follows how many people are deciding and how many ties
  they hold, so a small real shift in the measured window is plausible and is not a regression of the code.
- 1 000 and 5 000 people read equal or faster after, inside the same noise.

Not re-measured: the guide's scale table (`docs/guide/concepts.md`, v0.6.0), which needs a quiet host (spread under 20 %).

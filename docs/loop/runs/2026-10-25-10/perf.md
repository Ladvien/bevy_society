# Performance: one multiply per scored target

The change is one line in the per-target scoring loop (`score *= crowding_price(...)`) plus an occupancy count that
already existed before last iteration removed it, so the hot path is the same shape it was two iterations ago.

**Measured, interleaved, on the laptop** (`--people 100` and `--people 5000`, fastest window of five reps, alternating
before/after builds):

| persons | before (ms/tick, min / median) | after (ms/tick, min / median) |
|---|---|---|
| 100 | 0.79 / 1.60 | 0.80 / 1.60 |
| 5 000 | 433 / 534 | 434 / 535 |

Inside the noise at both sizes: the within-run spread at 5 000 is 27% and at 100 it is 90%.

**The price itself is O(places) per target**, not O(people): a place lookup in a `Vec` of four to a few dozen views.
At 5 000 people the benchmark's village has the same handful of places, so the cost does not grow with the
population.

**`big` could not run this iteration's suite.** Its load average was **21-30** with 4 GB free (other users), and two
`cargo test` runs died with `Connection to 192.168.1.110 closed by remote host` mid-compile. The suite was run on the
laptop instead; `tools/big-cargo.sh` works and is documented, it was simply not available when it was needed.

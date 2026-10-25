# Perf

## The change

One constant. No new pass, no allocation, no change to how many neighbours ORCA considers.

## Cost

None measurable: `NUDGE` is a multiplier applied once per member per step inside the existing solve.

## Reproducing the measurement

```sh
cargo run --release --example evidence -- --scenario village-evening --out /tmp/x
grep -c "finished .* (interrupted)" /tmp/x/narrative.txt
```

The counter is the harness's narrative, which renders every `ActivityEnded` for one villager. It is a
one-person sample of the village, so the *ratios* are the finding and the absolute counts are that sample.

## Scale curve, still unverified

I tried again to settle the standing item from last iteration — the guide's `0.6 / 13.5 / 245.1 ms` table. The
machine would not hold still: load average ranged 1.8 to 10.2 across the session, and consecutive runs of the
same binary at 1 000 people gave **36.8 / 58.1 / 46.2 ms**. No measurement taken under that noise is worth
recording as a fact, and quoting one would be worse than leaving the doubt open.

What can be said: before/after pairs taken back to back at 100 / 1 000 / 2 000 / 5 000 agreed within 1%, so this
change costs nothing. The *absolute* curve needs an idle machine.

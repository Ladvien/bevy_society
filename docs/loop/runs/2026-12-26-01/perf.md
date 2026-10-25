# Perf

This iteration's subject **is** perf, so the usual "scale curve unverified" section is replaced by the numbers.

## Measured, fastest window out of 5-9 repetitions

| persons | ms per tick | amortised per 60 fps frame | pairs |
|---|---|---|---|
| 100 | **0.74** | 0.04 ms (0.3%) | 4 950 |
| 1 000 | **28.4** | 1.8 ms (11%) | 499 500 |
| 2 000 | **100.6** | 6.3 ms (38%) | 1 999 000 |

## Cost of this iteration's change

The benchmark runs `--reps` windows instead of one, so it takes N times longer to answer. That is a deliberate
trade: the single-window answer was not worth having. Default is 5; `--reps 1` still works and still emits the
spread fields.

## What a host should take from this

1 000 people costs about **a tenth of a 60 fps budget** when the machine is quiet, not the 5% the guide used to
claim. 2 000 costs more than a third of a frame. Measure on the host's own machine with `--reps`, and treat
`spread_pct` above ~20 as "this machine was too busy for this figure to mean anything".

## Not measured

5 000 people: the run exceeded a 30-minute deadline at `--reps 7 --ticks 300` on this machine, which is itself
information — the guide's `245.1 ms` figure for that population is unreachable here and is no longer quoted.

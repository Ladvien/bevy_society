# Perf

## The change

None shipped. Every experiment reverted; `git diff src/ assets/` is empty.

## Cost of what was tried

All three levers were constant changes — a division in `relax_toward_zero`, a rate, and an authored delta. None
would have been measurably slower, and none shipped, so the point is moot beyond the record.

## Scale

Unchanged. The tool is now trustworthy — `examples/benchmark --reps N` reports min/median/max/spread, and the guide
quotes the fastest window (0.74 / 28.4 / 100.6 ms at 100 / 1 000 / 2 000), which reproduces to under 1%.

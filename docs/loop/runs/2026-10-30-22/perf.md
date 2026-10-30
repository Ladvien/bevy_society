# Scale re-measure, v0.6.0

Command: `cargo +1.98.1 build --release --example benchmark`, then for n in 100 1000 2000 5000: `./target/release/examples/benchmark --people $n --ticks 100 --settle 300 --reps 5`. Host: 24 threads, load average 11-38 during the runs.

| people | min ms/tick | median | max | spread % | ms per million pairs (median) |
|---|---|---|---|---|---|
| 100 | 0.862 | 1.101 | 1.174 | 28.3 | 222.3 |
| 1000 | 43.514 | 57.428 | 74.843 | 54.6 | 115.0 |
| 2000 | 155.778 | 206.109 | 431.760 | 133.9 | 103.1 |
| 5000 | 534.831 | 702.835 | 812.343 | 39.5 | 56.2 |

All spreads exceed the 20% line, so the figures are upper bounds. No code changed.

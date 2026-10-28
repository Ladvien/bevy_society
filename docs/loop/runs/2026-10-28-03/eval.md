# 2026-10-28-03 — benchmark homes

Open issues: #24, #27 (release; skipped). No bug issue. Item: `examples/benchmark` gave every person `Home(places[0])`, a place of capacity `people / places`, so all but a capacity's worth had no bed (the `people-cannot-reach-their-beds` crowd, built into the harness). Homes are now dealt round-robin: `places[i % places.len()]`.

Command: `cargo +1.98.1 build --release --example benchmark && ./target/release/examples/benchmark --people N --reps 3` (before = binary built from the parent commit).

| people | before median / min ms | after median / min ms |
|---|---|---|
| 100 | 1.377 / 1.218 | 1.320 / 1.251 |
| 1000 | 50.35 / 37.95 | 49.87 / 38.66 |
| 5000 | 610.4 / 600.3 | 659.3 / 599.3 |

Minima agree within 2%; medians differ by host load (load average 23-32, spreads up to 42%). No speed-up or slowdown is claimed. No simulation code changed; no plots or narrative apply, so no GREAT reviews are claimed.

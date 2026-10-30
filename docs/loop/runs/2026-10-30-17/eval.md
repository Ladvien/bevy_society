# Eval - decision-path list searches removed

Change: `decide_activities` reads the actor id from its `PersonIndex.sorted` slot, finds a neighbour's id by binary search over a sorted copy of `near`, and `score_option` reads the target's tie and position once per option (was once per consideration).

**Bit-identical.** `examples/evidence --scenario village-evening --seed S --days 6` on HEAD and on this tree: `summary.json` and `series.csv` equal (`cmp`) for S = 7 and 42. Full `cargo test --release` green; `tests/determinism.rs` green with `--no-default-features`.

**Speed (host load 20+, 24 cores, benchmark is single-threaded).** `cargo run --release --example benchmark -- --people 3000 --ticks 40 --reps 1`, before/after pairs, `ms_per_tick_min`: 277.4/254.9, 340.1/341.7, 363.4/331.3 (-8%, 0%, -9%). Too noisy for a figure; the gain is small. Where the rest goes (temporary timers, removed): at 1 000 people about 40% of `decide_activities` is `score_option` (starter pack's `Script` considerations run Rhai per target), 5% building the `Decision`, under 1% `targets_for`; the remainder was not attributed. `perf` and `samply` are unavailable here (`perf_event_paranoid=2`, ptrace scope 1).

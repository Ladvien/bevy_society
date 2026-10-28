# 2026-10-28-07 — benchmark output is readable again

Open issues: #24, #27 (release; skipped). No bug issue. Item: the side finding of pass 06 — `examples/benchmark` printed one `WARN` per home plus travel-timeout warnings (250 lines at 5 000 people) ahead of its JSON line, because its compressed setup (`seconds_per_day 4`, 20 people per bed) is exactly what the host-facing validators warn about.

Change: `examples/benchmark.rs` builds its `LogPlugin` with `filter: "warn,bevy_society::behavior=error"`. Setup and workload are untouched, so timings are comparable with earlier runs.

Check: `./target/release/examples/benchmark --people 100 --reps 3` prints exactly one line, the JSON (median 1.270 ms, spread 21.9%; the spread is host noise, load average 9, so it is not quoted anywhere). Before the change the same command printed 9 WARN lines first. `cargo fmt --check` and `cargo clippy --example benchmark -D warnings` clean. No crate code changed, so no behaviour review applies.

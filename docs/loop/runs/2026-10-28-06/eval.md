# 2026-10-28-06 — scale table re-measure (not completed)

Open issues: #24, #27 (release; skipped). No bug issue. Item: re-measure the guide's scale table (`docs/guide/concepts.md`, M1 Max figures) with the fixed benchmark homes.

Command: `cargo +1.98.1 build --release --example benchmark && ./target/release/examples/benchmark --people N --reps 5`.

Host load average 30-45 on arrival. 100 people: min 1.514 / median 1.741 ms, spread 35.6%. The 1 000 and 5 000 runs did not finish inside 900 s. By the guide's own rule (spread above 20% means the figures say nothing), no number from this pass is quotable. The table is unchanged; it is an M1 Max table and this host is not one.

Side finding: the benchmark emits one `WARN` per home plus the travel-timeout warnings (250 lines at 5 000 people) because its fixed `seconds_per_day 4` setup trips the host-facing validators; this buries the JSON line. Not changed this pass.

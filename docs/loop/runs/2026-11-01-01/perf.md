# Perf

Not measured: `warn_unscored_fields` runs once in `finish` over `defs.activities.len()`, and
`activity_signals` allocates one Vec per call. No system was added or altered, so per-tick cost is
unchanged by construction.

The numbers from the previous iteration stand: 14.1 ms per society tick at 1 000 persons, 0.55 ms at
100, 351.6 ms at 5 000 (`docs/guide/concepts.md`).

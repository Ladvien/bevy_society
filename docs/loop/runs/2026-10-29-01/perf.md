# Perf

Not measured: `warn_short_activities` runs once in `finish`, over `defs.activities.len()` entries,
and `activity_durations` allocates one small vector when a host calls it. Per-tick cost is unchanged
by construction — no system was added or altered.

`scale-1000` remains `pending`, and is now fifth of five.

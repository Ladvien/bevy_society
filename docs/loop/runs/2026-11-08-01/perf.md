# Perf

Release build, `--ticks 200`, 1 000 people, two runs each way.

| | ms/tick |
|---|---|
| before this iteration (no trace exists) | 14.96 |
| after, no trace resource | **15.28** |
| after, `DecisionTrace::with_capacity(64, 8)` | **15.16** |

The three figures are within run-to-run noise of one another. The important comparison is the second
and third: **adding the recorder costs nothing measurable**, which is the constraint Game AI Pro 3
ch. 6 puts on a recorder and the reason it is `Option<ResMut<DecisionTrace>>` rather than a resource the
plugin always inserts.

Why it is free: recording reads `all_scored`, a list the decision loop was already building in the
`trace.is_some()` branch only — so with no trace the branch is not taken and the loop is the one it was.
With a trace, appending a handful of `(String, f32, bool)` per decision is small next to the perception
and scoring work that produced them, and the ring caps the storage.

Tick cost at the other populations is unchanged by construction and not re-measured:

| Persons | ms per society tick |
|---|---|
| 100 | 0.556 |
| 1 000 | 15.16 |
| 5 000 | 277.0 |

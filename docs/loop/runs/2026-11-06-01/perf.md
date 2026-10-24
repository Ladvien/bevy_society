# Perf

Release build, two runs each. Two changes: the `Knowledge` cap, and the harness column.

| Persons | ticks | before (ms/tick) | after (ms/tick) | delta |
|---|---|---|---|---|
| 100 | 200 | 0.556 | 0.556 | 0 |
| 1 000 | 200 | 14.96 | 14.96 | 0 |
| 1 000 | 3 000 settle + 2 000 | **63.36** | **61.91** | **−2.3%** |

## The number that matters

Short runs show nothing, because the cap cannot bind until a person has heard 64 rumors. The long run
is where the defect lives:

| | before | after |
|---|---|---|
| 1 000 people, short run | 15.0 ms | 15.0 ms |
| 1 000 people, after 3 000 settle ticks | **61.9 ms** | **61.9 ms** |

4× the cost with the same population. This iteration removed 2.3% of it. **The rest is the rumor
book**, which grows 848 → 8 357 over 5 000 ticks and is walked in full by the `RumorReach` scan in
`rules.rs` on every tick. That fix is filed as
`needs-decision/retiring-untellable-rumors.md`: 93% of the book's rumors have no living teller and can
never spread again.

No scenario got slower.

## Why the cap is worth 2.3% and not more

`best_telling` scans a person's `Knowledge` once per gossip. Capping at 64 bounds that scan, but the
scan was never the dominant term: at 64 entries it is small next to the O(rumors) walk that runs for
every person every tick whether or not anyone gossips.

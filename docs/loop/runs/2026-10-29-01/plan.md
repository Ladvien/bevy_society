# Plan: warn about what I could not fix

## The order things happened in

1. Measured the log's #1 (`chat` unreachable). Found the village is fine (1:1.7 chat:gossip) and
   only the cluster starves (1:24).
2. Tried lowering gossip's `satisfies`. **Worse** — 128 → 106.
3. Tried lowering gossip's interaction effect. **+20%**, still starved.
4. Computed why: 17 gossip instances per person per day means any supply above 0.06 balances the
   decay, and less than that makes gossip worthless. Not a tunable rate.
5. Reverted both, restored the green baseline.
6. Picked the log's #2 instead, reframed by what steps 1–5 taught: the recurring, silent defect is
   an activity shorter than one tick.

## Design

`BehaviorPlugin::finish` warns when any activity (except the reserved `idle` fallback) is shorter
than two society ticks:

```
tick_hours = tick_secs / seconds_per_day * 24
degenerate = activities where duration_hours / tick_hours < 2.0
```

Two rather than one, because a one-tick activity is the failure mode and a two-tick one is already
suspicious: a person who re-decides every other tick cannot do anything varied.

The warning is a `warn!`, not a panic: a host may have a legitimate reason, and this crate's rule
is that misconfiguration gets a loud log line, never a panic outside the documented plugin-order
checks.

`SocietyDefs::activity_durations() -> Vec<(&str, f32)>` is added so a host — or a test — can make
the same check without reaching into `pub(crate)` internals. That was needed for
`tests/degenerate.rs` and it is the accessor a host would want anyway.

## Touched

- `src/behavior.rs` (`warn_short_activities`, called from `finish`),
- `src/defs.rs` (`activity_durations`),
- `tests/degenerate.rs` (new, 2 tests),
- `CHANGELOG.md`, `docs/loop/log.md`, this run's directory.

## Not touched

`chat`'s gate and gossip's rates — reverted, with the arithmetic recorded.

# Should walking speed be in game time or real time?

## The problem, measured

`SocialMover::speed` is metres **per real second** and steering runs on every fixed step
(`1/64 s`), so a mover advances `1.4 / 64 ≈ 0.022 m` per tick.

`Tuning::travel_timeout_hours` is **game hours**, converted to ticks with
`dt_days · 24`. At `seconds_per_day = 4.0` one tick is 5.6 game-minutes, so the 2-hour
timeout is 21 ticks.

```
a 20 m walk needs      914 ticks
the timeout allows      21 ticks
```

Nobody in the village ever reaches anything. Every activity ends `interrupted`, so
`satisfies` never pays out and all four needs sit at 0 — which the evidence run
`docs/loop/runs/2026-10-25-01/evidence/` shows directly: 37 interrupted endings against
11 completed, and needs at 0.000 for hundreds of ticks.

## Why it is a decision and not a bug fix

Both numbers are individually defensible. The mismatch is that they measure different
clocks, and the SPEC never says which one the walk should live in. Three options:

### 1. Make the timeout real-time too

`travel_timeout_hours` becomes `travel_timeout_secs`, converted with the fixed-step delta.
Walking and its timeout then share a clock, and a slow machine does not strand people.

- **For:** internally consistent; a host's `speed` means what it says; determinism is
  unaffected because the fixed step is deterministic.
- **Against:** a change to a `Tuning` field a host may already set, so it is a breaking
  change to the settings surface; and "2 game-hours of walking" is a more natural thing for
  a designer to say than "2 seconds".

### 2. Make walking speed game-time

`SocialMover::speed` becomes metres per **game-hour**, and steering converts with
`dt_days`. A `seconds_per_day` change then rescales every journey with the day.

- **For:** one clock everywhere, which is what every other rate in the crate uses.
- **Against:** the steering plugin needs `SocietyClock` and is currently independent of it;
  and a host that wants "1.4 m/s like a person" loses the ability to say so.

### 3. Leave both, and document the coupling

Hosts set `travel_timeout_hours` from `seconds_per_day` and the walking distance.

- **For:** no API change.
- **Against:** the failure is silent and catastrophic — a village that cannot walk — and
  the crate's own capture shipped with it for two iterations.

## Recommendation

**Option 1**, in a later iteration, as a deliberate breaking change with a `CHANGELOG`
migration note. It is the only one where a host's two settings cannot silently contradict
each other, and it keeps `speed` meaning metres per second, which is what a game engine
means by speed.

Until then, this iteration records the coupling in `docs/guide/concepts.md` and sets the
demonstration's `seconds_per_day` such that the numbers do correspond.

# People cannot reach their beds, and seven levers do not fix it

> **Investigated 2026-12-22. Nothing shipped; the blocker is real and is recorded with its ruled-out list.**

## The state

On `village-evening` (24 people, 17 game-days, seed 7) **150 `sleep`s are interrupted**, against 400 interrupted
endings of 6 768. Sleep is 37.5% of all remaining failures and the largest single activity.

## Established, by measurement

| fact | how |
|---|---|
| all 150 are `timed_out` | every exit path instrumented; none fired `gone`, `suspended` or `lost` |
| they stall just outside the arrival radius | **min 1.52 m, p25 1.83, median 2.06, p75 2.33**, against `interaction_radius` 1.5 |
| they hold a `Destination` | 150 of 150 |
| they hold a `SocialMover` | 150 of 150 |
| most are still pressing | **84 of 150 have velocity > 0.01 m/s** |

So: a crowd pressing toward targets it cannot reach, for the whole four-hour budget.

## The seven levers, all dead

| lever | interrupted | sleep | friends | rivals |
|---|---|---|---|---|
| baseline | 400 | 150 | 31 | 38 |
| scatter destinations (RNG) | 456 | 173 | 25 | 42 |
| scatter destinations (deterministic) | **484** | 174 | 25 | 36 |
| steering `stop_radius` 0.75 → 1.5 | **400** | **150** | 31 | 38 |
| `interaction_radius` → 2.2 | 378 | 160 | 24 | **90** |
| `interaction_radius` → 2.6 | 364 | 136 | 19 | **82** |
| `Home` honours `Place::capacity` | **440** | 155 | 22 | **69** |

Evidence: `docs/loop/runs/2026-12-22-01/evidence/levers.png`.

## The key negative result

**`stop_radius` produced a byte-identical simulation.** A change to how far walkers push toward their goal that
changes nothing means the push is not what limits them. Every "tune the walker" fix is therefore ruled out, and the
question becomes what the **solver is handed**, not how hard the walker wants to go.

## Why this is a decision and not a bug fix

The two levers that did reduce sleep both work by enlarging `interaction_radius`, which is also what decides contact
— and rivals go **38 → 90** at 2.2 m. That is the same coupling six earlier clamp iterations mapped: proximity
changes ties. So the fix has to either

1. **separate arrival radius from contact radius** — a `Tuning`/`SocietySettings` schema change, and the cleanest
   option, since the two currently share one number for no stated reason;
2. **fix the crowding itself** — make the solver not deadlock, which is a change to `steering.rs` and its ORCA
   parameters, and needs the instrumentation described below first; or
3. **accept it** and restate what a full village day looks like, which is honest but leaves 37.5% of failures in.

## Recommendation

**Option 1, after instrumenting the solver.** Splitting arrival from contact is the one change that cannot damage
the tie network, because it leaves `contact_radius` alone. But it should not be done before the next attempt
records `pref_vel`, neighbour count and member radius for the stuck walkers — the `stop_radius` no-op says those
numbers, not the geometry, are where the answer is.

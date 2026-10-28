# 2026-10-28-11 — plan

- `Tuning::grudge_half_life_hours` (default 12.0, design default, sweep in `eval.md`).
- `recall::halving(hours, half_life)`; `Memories::grudge(other, now, half_life_hours)`; `ctx.grudge` reads it from the snapshot (`SocietySnapshot.grudge_half_life_hours`).
- Harness: `examples/evidence --grudge-half-life H`.
- Tests: `halving` unit test, `a_grudge_cools_on_its_own_half_life`, plus a behavior-level test that a day-old insult no longer drives a snub at the default (see eval).
- Docs: `docs/research/memory.md`, CHANGELOG, `docs/guide` if `ctx.grudge` is described.
- Not a structural change: no message or schema change; `ctx.grudge` keeps its signature.

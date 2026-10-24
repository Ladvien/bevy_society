# Plan: evaluate `Signal::Script` through the engine

## The defect, as found

`src/behavior.rs::signal_value` returned `0.0` for `Signal::Script`. `DefsFile` accepts a
`Script("…")` consideration, `ScriptCompiler` compiles it, `ScriptEngine::eval_f32` exists to score it
— and the decision never called it. An activity whose only consideration was a script therefore scored
0 and was never chosen.

Evidence, before the fix (`tests/dbg_s.rs`, a throwaway):

```
script-only activity chosen: Some(0) (idle is 0, read is 1)
```

Activity `read` declared `considerations: [(input: Script("1.0"), curve: Linear(1.0, 0.0))]`. A
constant 1.0 through a linear curve is a certain 1.0, the highest a consideration can score. The
person idled anyway.

Why this is the most valuable item this iteration could pick:

1. It is a **silent** failure. Nothing errored, nothing logged; a host author who wrote a script
   consideration got a village where that activity never happened and no clue why.
2. It is **already promised** — by the definition format, by `eval_f32`'s existence, and by
   `docs/guide/authoring.md`, which documents `Script` as a valid input.
3. It is the one dead-code item in the inventory that is a *bug* rather than an unused convenience.

## Design

`signal_value` gains a `scripts: Option<&ScriptContext<'_>>`. The context carries the defs, the
tick-start snapshot, the engine and the two bound person ids, and evaluates through the existing
`ScriptEngine::eval_f32` — the same sandboxed engine, the same 10 000-operation limit, the same
`Bindings`, no second path.

- A `Script` with no context (no `ScriptEngine` resource, or no snapshot yet on the first tick) scores
  0, and the activity is skipped that tick.
- A script that fails scores 0 and logs, matching the crate's rule for every other failed
  consideration.
- The result is clamped to `[0, 1]` like every other input, so a script cannot outrank the scale.
- `Field` keeps scoring 0: it needs the stigmergy grids, which only the `features` feature builds, and
  that is a separate gap recorded below rather than silently fixed here.

## Touched

- `src/behavior.rs`: `ScriptContext`, `score_option`, `signal_value`, and `decide_activities` gaining
  `engine` and `cell` parameters.
- `tests/behavior.rs`: two tests — a script that returns 1.0 must win, a script that returns 0.0 must
  not.

## Not touched

`Field` scoring, the remaining four dead accessors, and the capture's thresholds — all recorded in
`review.md` as the next iteration's candidates.

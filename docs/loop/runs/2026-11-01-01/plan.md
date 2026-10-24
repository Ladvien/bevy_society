# Plan: check the premise, then say what is true

## Why this and not the log's #1

#1 is a second need in the definitions schema — a change this loop escalates rather than decides. #3
was `Signal::Field` / `Gradient`, listed with "Moussaïd 2011 needs re-converting" as the blocker.

The first thing this iteration did was test that blocker. **It was wrong.** The field grids are built,
deposited, evaporated and queried correctly; `field_value` returns 1.0 at a crowded cell and
`gradient_target` returns a point on the slope. Moussaïd 2011 is a crowd-*steering* paper and has
nothing to do with a `Gradient` target that walks a steeper-is-better field. The real gap is that
`signal_value` and `targets_for` do not call either function.

So the work became: wire them, and if that fails, make the failure loud.

## Design, and what changed after the attempt failed

1. Thread `Option<Res<SocialFields>>` through `decide_activities` → `score_option` → `signal_value`,
   and through `targets_for` with the actor's factions. **Attempted, tested, failed, reverted.**
2. Ship a `warn!` naming every definition that relies on a `Field` consideration or a `Gradient`
   target, plus the accessors a host needs to make the same check. **Shipped.**

The warning is the right shape regardless of the wiring: a definitions file that names a feature the
decision does not read should not be accepted in silence, and the `Script`-consideration warning two
iterations ago was the same defect with a different signal.

## Not a panic

A host may legitimately keep a `Field` definition while the wiring is unfinished — the rest of its
society works. `warn!`, not `panic!`, per the crate's rule that misconfiguration is loud but never
fatal outside the documented plugin-order checks.

## Touched

- `src/defs.rs` (`activity_signals`, `has_gradient_activity`, `Signal` made public),
- `src/behavior.rs` (`warn_unscored_fields`, called from `finish`),
- `src/lib.rs` (`Signal` re-export),
- `tests/fields_unscored.rs` (new, 4 tests),
- `docs/loop/needs-decision/field-inputs-are-unscored.md`,
- `CHANGELOG.md`, `docs/loop/log.md`, this run's directory.

# Eval

## What is now visible that was silent

| | before | after |
|---|---|---|
| A definition with `Field(Crowd)` or `Gradient(…)` | accepted, activity never chosen, no signal | accepted, activity never chosen, **`warn!` at startup naming it** |
| A host's own check | impossible (the accessors were private) | `SocietyDefs::activity_signals()`, `SocietyDefs::has_gradient_activity()`, public `Signal` |

## The measurements behind the warning

| claim | value |
|---|---|
| `field_value(Crowd)` at a crowded cell | 1.0 (sample == peak 0.069) |
| `gradient_target(Crowd, …)` on a crowd | `Some(Vec2(4.0, -4.2e-7))` |
| `Gradient` activity, 60 ticks | **0 person-ticks** |
| `Field(Crowd)` activity, 60 ticks | 0 person-ticks |

Evidence: `evidence/gradient-never-chosen.json`.

## What did not ship

The wiring. Two tests were written, both failed, and the cause was not found: the `Field` arm runs
with the right position, `field_value` at that cell returns 1.0, and the score is still 0. Reverted.
Recorded in `needs-decision/field-inputs-are-unscored.md` with the one ordering fact established
(`Decide` precedes `Relate`, so a decision reads the previous tick's grid).

## Gates

| Command | Result |
|---|---|
| `cargo test` | **18** binaries ok (17 + `fields_unscored`) |
| `cargo test --no-default-features` | 18 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## Dead-code inventory

Unchanged at zero. `Signal` became public, which is the opposite of dead code: `activity_signals`
returns it, and `warn_unscored_fields` reads it.

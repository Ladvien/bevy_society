# Eval

## Change

`Signal::Field` now reads `SocialFields::field_value` at the actor's cell with its faction memberships, and
`ActivityTarget::Gradient` reads `gradient_target`. Both were constants before: `0.0` and no targets.

## Before / after

| | before | after |
|---|---|---|
| `field_value(Crowd)` read by the decision | **0.0** (constant) | **0.120** — the real cell |
| `Gradient` target | **none** | `Some(Vec2(-2.0, 0.0))` |
| a `Field`-only activity chosen, 2 400 ticks | **0** | **6** |

Behaviour with the starter pack is **bit-identical** — 67 199 of 67 199 `village-evening` series rows —
because the pack uses no `Field` input.

## Three causes, one of them the wiring

The decision file had recorded that wiring was tried and failed. It was three things:

1. **The test's crowd had no slope.** Nine people in one cell is a plateau: `field_value` divides by the
   channel's *peak*, so every cell in the hottest region reads 1.0 and `gradient_target` correctly returns
   `None`.
2. **The fallback outlasted the test.** `idle` is one game-hour and a tick is 0.0016 game-hours, so a single
   fallback holds **640 ticks**. The original ran 60; two of mine ran 400.
3. **The wiring**, which needed `Decision::own_factions` as well.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **43** binaries |
| `cargo test --no-default-features` | **43** binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | 0 errors |

## Dead-code inventory

Zero. The `Field` and `Gradient` arms are reachable and asserted; with the feature off the `NoFields`
stand-in answers both and the warning covers the host.

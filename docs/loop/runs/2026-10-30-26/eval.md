# 2026-10-30-26 — eval

Item: bugs #62 (NeedBelow arming), #64 (restore by trigger index), #63 (`fast_forward` steps), #65 (stale
save docs). Fix commits `6ee6f2b` (#63) and `da098d8` (#62, #64, #65); baseline `300c52b`.

## Failing before, passing after

| test | on `300c52b` | on `da098d8` |
|---|---|---|
| `tests/need_below.rs` (5) | 5 fail: 10 firings in 10 ticks for person 0; a `persons: [1]` trigger fires 0 times; two triggers on one line fire 4 times a tick; a hovering level re-fires; an edit re-arms | 5 pass |
| `tests/fast_forward.rs::fast_forward_after_a_live_frame_runs_exactly_the_ticks_asked` | fails | passes |
| `tests/fast_forward.rs::fast_forward_refuses_a_count_the_clock_cannot_hold_and_a_world_without_time` | never returns (`f32::MAX` days loops for ever; killed after 20 min) | passes |
| `tests/save_triggers.rs` (3) | does not compile against `300c52b` (`TriggersSave::ids`, `need_disarmed`); the issue's reproduction, a v1 save restored into a v2 defs, ran `welcome` again | passes |

## Gate

- `cargo fmt --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`: clean.
- `cargo test --release --all-features --no-fail-fast`: 96 test binaries ok, 0 failures.
- `cargo test --release --no-default-features --no-fail-fast`: 96 test binaries ok, 0 failures. The 5 library
  dead-code warnings are the same as `300c52b`.
- One protected pin raised by exactly what the change adds: `tests/economy.rs` asserted the save `VERSION`
  literally (`4`); it is `5` (commit `assumed owner exception: tests/economy.rs save version pin 4 -> 5`).
  Nothing was removed or loosened.

## Scenario evidence (the village, not a unit fixture)

`evidence/need_below_probe.rs` is a throwaway test: a village of N people, the starter pack plus the
village additions plus one `NeedBelow(food, X)` trigger, run for D game days, counting `TriggerFired` per
person against an independent oracle that arms and disarms each person on their own.

```sh
cp docs/loop/runs/2026-10-30-26/evidence/need_below_probe.rs tests/zz_probe.rs
PROBE_SEED=7  PROBE_PEOPLE=12 PROBE_BELOW=0.7 PROBE_DAYS=3 cargo test --release --features serde --test zz_probe -- --nocapture
PROBE_SEED=99 PROBE_PEOPLE=24 PROBE_BELOW=0.5 PROBE_DAYS=6 cargo test --release --features serde --test zz_probe -- --nocapture
rm tests/zz_probe.rs
# before: git checkout 300c52b -- src/rules.rs src/save.rs   (restore with: git checkout HEAD -- src/rules.rs src/save.rs)
```

| run | firings | independent crossings | people who fired |
|---|---|---|---|
| seed 7, 12 people, 3 days, line 0.7, **before** | 1 | 12 | 1 of 12 |
| seed 7, 12 people, 3 days, line 0.7, **after** | 12 | 12 | 12 of 12 |
| seed 99, 24 people, 6 days, line 0.5, **after** | 29 | 29 | 24 of 24 (5 twice) |

Tables per person: `evidence/before/firings.md`, `evidence/after/firings.md`.

Behaviour of the shipped society is unchanged: neither the starter pack nor the village additions use
`NeedBelow`, so no other scenario's output moves (the full suite, which pins several hashes and rates, is
green).

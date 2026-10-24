# Eval

## Before / after, same seed, same definitions

The defect and its fix, measured by the two tests that bracket it:

| Case | Before | After |
|---|---|---|
| `Script("1.0")` consideration, one person | `Some(0)` — idle | activity `1` chosen |
| `Script("0.0")` consideration, one person | `Some(0)` — idle | idle (correct: a 0 must lose) |
| `ctx.need(actor, "rest")` script, two people at 0.9 and 0.1 | both idle, indistinguishable | tired acts, rested idles |

## Test counts

| File | Tests | Result |
|---|---|---|
| `tests/behavior.rs` | 11 | ok |
| whole suite, default features | 16 binaries | ok |
| whole suite, `--no-default-features` | 16 binaries | ok |

`cargo fmt --check`: clean.

## Dead-code inventory delta

`cargo clippy --all-targets --all-features -- -D warnings`, unused items:

| Before | After | Item |
|---|---|---|
| 6 | 5 | `ScriptEngine::eval_f32` is now called by `signal_value` |

The remaining five are accessor-only fields (`TriggerSpec::name`, `ActivitySpec::name`,
`ActivitySpec::barks`, `ActivitySpec::target`, `Names::{deed,activity,interaction}`, `TieView::target`)
that nothing reads. `ActivitySpec::barks` is a real gap — the SPEC says an activity's barks are thought
lines on start — and is recorded as the third candidate below.

## The capture, run during this iteration

```
{"frames": 480, "rumor_reach": 0.040, "friends": 0, "rivals": 0, "barks": 0}
```

against `docs/DEMOS.md`'s thresholds of `≥0.5`, `≥3`, `≥1`, `≥6`. Not met, and not caused by this
change: the capture pins `seconds_per_day 64.0` with `tick_secs 0.1`, so 480 frames is six game-hours,
and a village needs more social contact than six hours of a 64-second day provides at the crate's
default rates. This is the strongest evidence for the next iteration's pick.

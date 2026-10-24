# Eval

## The headline number

The README's quick start, run exactly as written, before and after. Same code, same 600 updates, same
seed; the only difference is the documentation.

| | society ticks in 600 updates |
|---|---|
| before (`git show HEAD:README.md`) | **0** |
| after (`README.md`) | **37** |

`evidence/before-tick-count.txt` and `evidence/after-tick-count.txt`. Zero is not a slow society; the
clock never advances, `now_days` stays at its start hour, and nothing in the crate runs.

## The guide

| | result on `app.finish()` |
|---|---|
| before | **panic:** `bevy_society: DialoguePlugin needs bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera>` |
| after | no panic; dialogue left out until the host has a camera |

`evidence/before-guide-panic.txt`.

## Documentation tests

| Check | before | after |
|---|---|---|
| a quick start in `src/lib.rs` doctests | none | compiles **and runs**, asserts `tick > 0` |
| README block vs canonical snippet | unverified, and wrong | byte-equal, asserted |
| a hand-fix with a missing import | — | **caught**: `TimeUpdateStrategy` is not in Bevy 0.20's prelude |
| guide warns about the dialogue panic | no | yes, asserted |

The third row is the iteration's own near-miss: my first repair passed every text assertion and did not
compile. Only a doctest caught it.

## Behaviour measured while reading the evidence (not fixed here)

| Observation | Value |
|---|---|
| `need3` pinned at 0.0 | 3 283 / 4 200 ticks (**78.2%**) — never satisfied on four days in five |
| `need2` pinned at 0.0 | 1 950 / 4 200 ticks (46.4%) |
| rumor reach after planting at tick 200 | last changes at tick **62**, final 0.500 — spreads instantly, then stops dead |
| message symmetry, one tracked person | **19** `ActivityEnded` vs **9** `ActivityStarted` |

The last one is a crate defect with a precise cause: `ActivityStarted` is written **on arrival**
(`behavior.rs:340`) and when an activity needs no travel (`:832`), while `ActivityEnded` is written
whenever a `Doing` is removed, including a travel timeout (`:449`). A person whose target is
unreachable therefore ends an activity nobody was told started.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 20 binaries ok (19 + `readme`) |
| `cargo test --no-default-features` | 20 binaries ok |
| `cargo test --doc` | 2 ok — one of them **runs** the quick start |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |

## Dead-code inventory

Zero, unchanged. This iteration added no library surface; it removed a class of wrong documentation.

# Research: what a host copies, and why an unrun snippet is a lie

## The problem this iteration picks

The brief's candidate #3, host ergonomics: *"what a host must know to use it"*. The cheapest possible
test of that is to be the host: take the crate's own documentation, paste it, run it. Three of the
four entry points failed.

## Sources

Not a behaviour model this time, so no new paper is claimed. The relevant literature is on
documentation that is *executable*, which is a well-known practice rather than a citable effect:

| Claim | Source |
|---|---|
| A documentation example should be compiled and run by the test suite, not by hand | Rust's `rustdoc` book, "Documentation tests": docs are extracted and run as a test binary |
| A code sample that cannot fail loudly is not a test | the same, and the crate already follows this for `src/lib.rs` |

The crate's own `AGENTS.md` rule that every rate has a named field and every default a source is the
same principle applied to numbers; this iteration applies it to *code a host is told to write*.

## What was measured, before changing anything

Three entry points, each run exactly as the crate presents it:

| Entry point | As written | Result |
|---|---|---|
| `README.md` quick start | the block the README calls "this code" | **0 society ticks in 600 updates** |
| `docs/guide/getting-started.md` "Add the plugins" | `add_plugins(SocietyPlugins)` over `DefaultPlugins` | **panics:** `DialoguePlugin needs bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera>` |
| `examples/quickstart.rs` | the same code, with the fixed step pinned | runs, 59 exchanges, 30 accepted |
| `src/lib.rs` doctest (before) | — | did not exist for the quick start |

So the *example* was right and the two documents that tell a host to copy it were wrong. The README
even said "which is this code", which was false in the one line that matters: a headless app has no
frame clock, so `Time<Fixed>` never advances and the society is frozen.

## What believable output looks like

Not a model parameter this time; the observable is that **the documented snippet advances the
society**. The check is a tick count, which is unambiguous: 0 is broken, anything above 0 is working.
Measured: 0 before, 37 after, in the same 600 updates.

## The second finding, which the fix exposed

The first hand-written repair named `TimeUpdateStrategy` from `bevy::prelude::*`, and **did not
compile**: Bevy 0.20's `bevy_time` prelude is
`{DelayedCommandsExt, Fixed, Real, Time, Timer, TimerMode, Virtual}` — `TimeUpdateStrategy` is not in
it. Text-level tests passed; only making the snippet a real doctest caught it. That is the argument
for the rest of this iteration: prose about code is not evidence about code.

## Not this iteration's problem, but measured and recorded

Reading the plot the brief requires turned up two behavioural defects worth more than the doc fix:

| Observation | Measurement |
|---|---|
| `need3` never met | at 0.0 on **3 283 of 4 200 ticks (78%)** |
| `need2` rarely met | at 0.0 on **1 950 ticks (46%)** |
| a rumor planted at tick 200 stops spreading immediately | reach changes **last at tick 62**... and never again; final 0.500 |
| `ActivityEnded` without a matching `ActivityStarted` | 19 ends vs 9 starts for one tracked person |

The last one is a crate defect, not a harness one: `end_activities` writes `ActivityEnded` whenever it
removes a `Doing`, but `ActivityStarted` is only written **on arrival**. A person who decides an
activity whose target is out of reach times out after `travel_timeout_hours` and emits an end for a
start nobody saw. Logged as the top item for the next iteration.

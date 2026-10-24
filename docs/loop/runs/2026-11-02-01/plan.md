# Plan: assert the warnings, and find out what that reveals

## Why this

Two iterations shipped a startup `warn!` and neither could be tested: `headless` has no logger, so a
test could only pin the *facts* the warning keys on. Deleting either `warn!` left every test green.
A warning nobody asserts is a comment.

## Design

`tests/common/capture.rs`, adapted from Bevy's `examples/app/log_layers_ecs.rs`:

- `LogPlugin::custom_layer` is `fn(&mut App) -> Option<BoxedLayer>`, so the layer builds an `mpsc`
  channel, keeps the sender, and inserts the receiver as a resource.
- `LogCapture` is `Mutex<Receiver<CapturedLog>>` — the `Mutex` because `mpsc::Receiver` is not `Sync`
  and Bevy 0.20 removed non-send resources entirely (`insert_non_send_resource` does not exist, so
  the upstream example does not compile on this version).
- `drain()` and `containing(needle)` are the test-facing reads.

`tests/warnings.rs` builds one capturing app and asserts both warnings, because the `tracing`
subscriber is global and settable once per process — the first test captures, later ones silently do
not, which is worse than no harness.

## The scope change the harness forced

Asserting the field warning's text showed it was wrong: an activity with both a `Field` consideration
and a `Gradient` target was counted twice, so one offending activity was reported as
`2 definitions … — follow, follow`. Fixed in `warn_unscored_fields` by asking, per activity, whether
it reads a field *or* walks a gradient, and the test now asserts `1 definitions`.

That is the point of the harness: the previous iteration's four tests were green while the sentence
they guarded was wrong.

## Touched

- `tests/common/capture.rs` (new), `tests/common/mod.rs` (one `pub mod`),
- `tests/warnings.rs` (new, 2 tests covering both warnings),
- `src/behavior.rs` (`warn_unscored_fields` de-duplicated),
- `src/defs.rs` (`activity_targets_gradient`),
- `CHANGELOG.md`, `docs/loop/log.md`, this run's directory.

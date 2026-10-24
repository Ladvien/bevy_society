# Research: making a warning testable, and what that exposed

The log's #4: "a log-capture harness — now blocking two separate warnings from being properly
tested". Two iterations had shipped a `warn!` pinned only by the *facts* it keys on, so deleting
either call would have left every test green.

## Why the tests could not see warnings

`tests/common::headless` builds `MinimalPlugins`, which has no `LogPlugin`. Without a `tracing`
subscriber, `warn!` writes nowhere. Adding `LogPlugin` alone is not enough either: it installs a
*global* subscriber, and a global subscriber can be set **once per process**. The observed behaviour:

```
test a_degenerate_duration_is_warned_about ... ok
test a_sane_day_length_is_not_warned_about ...
  ERROR bevy_log: Could not set global logger and tracing subscriber as they are already set.
```

So the first test captures, every later one silently does not — which is worse than no harness,
because it looks like it works.

## Design

`tests/common/capture.rs`, following Bevy's own `examples/app/log_layers_ecs.rs`:

- `LogPlugin { custom_layer, .. }` is a `fn(&mut App) -> Option<BoxedLayer>`, so the layer can build
  an `mpsc` channel, keep the sender in itself and insert the receiver into the `App`.
- A `Layer::on_event` formats the event's `message` field through a `field::Visit` and sends it.
- `LogCapture(Mutex<Receiver<CapturedLog>>)` is the resource; `drain()` and `containing(needle)` read
  it. The `Mutex` is needed because `mpsc::Receiver` is not `Sync` and **Bevy 0.20 has no non-send
  resources** — `insert_non_send_resource` does not exist, so Bevy's own example's approach does not
  compile on this version.
- Because the subscriber is global, a test binary may build **one** capturing app. `tests/warnings.rs`
  therefore asserts both warnings from a single run: a short day, with one activity that is both
  short and relies on a field input.

## What capturing the warnings exposed

**A real defect in the previous iteration's warning.** `warn_unscored_fields` pushed an activity once
for a `Field` consideration and again for a `Gradient` target, so an activity with both was counted
twice. The captured line read:

```
bevy_society: 2 definitions rely on social-field inputs ... — follow, follow
```

...for a definitions file with **one** offending activity. Fixed: the check now asks, per activity,
whether it reads a field *or* walks a gradient, and pushes it once. The message reads
`1 definitions ... — follow`.

That is exactly the class of bug a log assertion catches and an invariant assertion does not. The
previous iteration's four tests were all green while the message was wrong.

## Not done

No filter/level plumbing beyond what a test needs, and no assertion on the *absence* of unrelated
warnings (a test asserting "no other warnings" would fail the moment Bevy adds one).

## Verdict

The harness exists, both shipped warnings are now asserted end to end (level, message, activity name
and arithmetic), and the first thing it caught was a real defect in a warning shipped two iterations
ago.

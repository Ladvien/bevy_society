# Plan: make `Destination` mean something without steering

## Why this over the runner-up

Runner-up was retiring untellable rumors (93% of the book, a 4× cost growth). Lost on one sentence:
**this is a broken contract in a supported configuration, not a paperweight** — `Destination` promises
movement, `--no-default-features` promises a working crate, and the two contradicted each other. It is
also the cause of the log's #1 item.

## Design

```rust
fn walk_toward_destination(
    settings: Res<SocietySettings>,
    time: Res<Time<Fixed>>,
    mut walkers: Query<(&Destination, &mut Transform), Without<crate::SocialMover>>,
)
```

- Runs in `FixedUpdate` after `Rules`, every fixed step — the same slot `steer_movers` uses, so motion
  is smooth while the society ticks slowly.
- `Without<SocialMover>` is the whole isolation story: a host that steers keeps its own movement, and
  a host that does not gets this. One path per person, never both.
- But `SocialMover` **does not exist** without the `steering` feature, so writing that filter
  unconditionally breaks the very build the fix is for — which the no-default-features gate caught.
  A `cfg`-selected alias, `type NotSteered = Without<SocialMover>` or `= ()`, keeps one function body
  for both builds.
- Height is preserved by hand rather than through `SocietyPlane::lift`, which zeroes the other axis.

`Tuning::walk_speed` is added beside `travel_timeout_hours`: both are about the same walk, and both
must exist without the `steering` feature. Default 1.4 m/s, an ordinary human walking speed, declared
as a design default.

## Touched

- `src/behavior.rs` — the walker, registered in `BehaviorPlugin`; `Destination`'s doc corrected to name
  the system that keeps its promise.
- `src/core.rs` — `Tuning::walk_speed`.
- `tests/destination.rs` (new) — two tests.
- `CHANGELOG.md`, `docs/loop/log.md`, `docs/research/needs-and-utility.md`.

## New `Tuning` fields

`walk_speed: f32 = 1.4`, metres per **real** second. Design default: an ordinary human walking speed,
and the same number `SocialMover::speed` already used, so a host that switches to steering sees no
change in pace.

## Tests

| Test | What it proves |
|---|---|
| `a_person_walks_toward_its_destination` | a person with a `Destination` is no longer stationary |
| `and_arrives_so_a_place_activity_can_complete` | the walk finishes, so a `Place` activity can pay its `satisfies` |

Written **red first**: `moved 0.00 m from the origin; arrived false`. After: `moved 6.72 m; arrived
true`.

Both run in the default feature set, where `steer_movers` also exists — so they also prove the
`Without<SocialMover>` guard does not stop the core walker moving a person who has no mover. The
no-default-features build is covered by `cargo test --no-default-features` compiling and passing the
same file.

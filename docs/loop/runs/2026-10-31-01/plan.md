# Plan: replace an adjective with a table

## Why this

`scale-1000` had been `pending` since the loop began, the brief ranks performance fourth, and the
guide asserted the design was "fine" to about a thousand people without a number behind it. A
measurement is cheap, permanent, and it settles a claim the crate makes about itself.

## Design

`examples/benchmark`:

- N autonomous persons in a golden-angle spiral (even spread, no grid artefacts);
- tagged places scaled with the population (`people / 20`, minimum 4) in rings, so distances vary;
- **settle** frames before measuring, because an idle crowd skips the decision and exchange systems;
- time N society ticks, report `ms_per_tick` and `ms_per_million_pairs` as one JSON line.

`tick_secs = 1/64` with `TimeUpdateStrategy::ManualDuration` so each update is exactly one tick, the
same convention the tests and the evidence harness use.

## Deliberately not done

- **No optimisation.** Perception is brute force by design; a spatial index changes `Perception`'s
  shape and belongs in `needs-decision/`.
- **No per-update number.** A society tick is what costs, and a host's frame cost is derived from it.

## Touched

- `examples/benchmark.rs` (new), `Cargo.toml` (one `[[example]]`),
- `docs/guide/concepts.md` (the Scale section),
- `docs/loop/scenarios.yaml` (`scale-1000` → measured),
- this run's directory and `CHANGELOG.md`.

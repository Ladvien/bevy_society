# Guides

Start with [`getting-started.md`](getting-started.md); the rest is reference for one part of the crate each. The README's quick start is the crate's own doctest, kept in sync by `tests/readme.rs`.

- [`getting-started.md`](getting-started.md) — install, the first village, and reading what happens.
- [`concepts.md`](concepts.md) — ticks, game-days, the clock's two sources, snapshots, determinism, scale, ids.
- [`tour.md`](tour.md) — the pieces one at a time: people, needs, ties, opinions, rumors, factions, scripts, dialogue.
- [`features.md`](features.md) — every feature expanded, with the paper and the measured numbers behind it.
- [`authoring.md`](authoring.md) — every definition field, the starter pack, the overlays, the error messages.
- [`decisions.md`](decisions.md) — `DecisionTrace`: why a person chose what they did.
- [`persistence.md`](persistence.md) — `capture` and `restore`: saving the whole society (feature `serde`).
- [`scripting.md`](scripting.md) — the Rhai `ctx` / `fx` API, triggers and the sandbox limits.
- [`dialogue.md`](dialogue.md) — conversations and barks through `bevy_speech_bubbles`.
- [`editor.md`](editor.md) — driving a society from an editor: fast-forward, preview and reset, dry-running a trigger, finding who depends on a building.

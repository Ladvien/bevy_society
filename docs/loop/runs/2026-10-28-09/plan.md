# plan
- `Tuning::revenge_settles` (default chosen by sweep 0 / 0.5 / 1.0).
- `src/behavior.rs::resolve_interactions`, after step (6) `record`: if `accepted && harm < 0`, call `Memories::settle(target, factor)` on the actor, scaling negative-valence memories about `target` by `1 - revenge_settles`. The just-written actor memory is positive so it is untouched. New `Memories::settle` in `src/mind.rs` (engine-free arithmetic stays in the component).
- Tests: `tests/behavior.rs`-style: grievance falls after an accepted insult, is untouched after a refused one, untouched for a different target; determinism unchanged.
- Evidence: `examples/evidence --revenge-settles X`, village-evening 3 seeds and 60-person town; counters: snubs per insult received, rivals, friends.
- Docs: `docs/research/relationships.md` (or memory.md), CHANGELOG, log.

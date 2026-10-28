# Plan

Data-only: in `assets/starter.society.ron` change `snub`'s grudge consideration from `Linear(1.0, 0.0)` to `Power(k)`, so a mild grudge rarely turns into an insult and a deep one still does. Pick k by measuring the compliment:insult ratio at seeds 7/42/11 and 23/16 people. No Rust, no new Tuning field. Docs: `docs/research/rumor-and-reputation.md` + CHANGELOG (starter pack change rule). Tests: existing suite (the pack parses; tests/leaf.rs).

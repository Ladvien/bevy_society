# Evidence harness counts directed ties

Regenerate: `cargo run --release --example evidence -- --scenario village --people 24 --days 6 --out <dir>` (series.csv deleted, ignored).
After: `summary.json` friends 25, rivals 93 (directed ties, as `tests/friendship_bound.rs`). Before, the same ties printed as 12 and 46. Library code untouched; no behaviour change, so no plots/three-review gate.

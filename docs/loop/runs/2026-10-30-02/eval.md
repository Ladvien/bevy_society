# Do people keep going back to the same partner? (measurement, one rejected change)

Regenerate: `cargo run --release --example evidence -- --scenario village --people 24 --days 6 --out <dir>` (new file `partners.txt`; `series.csv` ignored, not committed). Breakdown by interaction: `--days 3`.

**Observed.** Bram's day (narrative) shows five `compliment`s with Garet between 11h28 and 16h22. Measured village-wide (village, 24 people, seed default, 6 days): of 2 526 exchanges, **833 (33.0%)** are with the initiator's previous partner (uniform choice among ~7 neighbours would be ~14%). By interaction (3 days): chat 30%, compliment 42%, gossip 34%, insult 28%. `evidence/partners-before.txt`, `partners-by-interaction-3d.txt`.

**Tried, rejected.** Gate `compliment` on the social need (`Need("social") Linear(-0.6, 1.0)` in `assets/starter.society.ron`), the hypothesis being that sated people stop complimenting. Repeat share 33.0% -> 32.1% (unchanged); compliments 743 -> 471; friends 26 -> 16, rivals 77 -> 105. Worse on the ledger that matters, no effect on the target. Reverted. `evidence/summary-social-gate.json`, `partners-social-gate.txt`.

**Conclusion.** Repetition is a property of partner choice (affinity-weighted target draw, a place of ~8), not of one activity's gate. A fix would be a per-tie `last_exchange_day` (a `Tie` field, so a save-schema change) read by a new `Input`; not built: a third of exchanges repeating a partner is not yet shown to be implausible, and no paper in the notes gives a target rate. Library unchanged.

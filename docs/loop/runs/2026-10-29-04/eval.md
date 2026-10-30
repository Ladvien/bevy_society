# Eval — 6 h refractory, before/after at the same seeds

Regenerate (release build): `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 [--retaliation-refractory 6] --out DIR`, and `--seed 7 --people 60 --days 30` for the town. Host load was 25-45, so no timing is claimed.

| run | insults b→a | rivals b→a | friends b→a | became_rival b→a | reply share of snubs b→a |
|---|---|---|---|---|---|
| village s7 | 390→343 | 35→27 | 21→23 | 108→105 | |
| village s42 | 369→382 | 28→28 | 22→15 | 130→128 | |
| village s11 | 415→348 | 36→29 | 20→20 | 130→114 | |
| town 60 s7 | 1875→1740 | 170→138 | 29→24 | 1089→819 | 55.5%→41.2% (snubs 3781→3197) |

Files: `evidence/{before,after}/{7,42,11,town}/summary.json`, `grudge_sources.txt`, `narrative.txt`; plot `evidence/plots/rivals-friends.png`.

Default `None` changes nothing (the `before` runs are the default configuration; the full test suite is unchanged).

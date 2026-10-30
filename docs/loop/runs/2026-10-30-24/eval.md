# Eval
Regenerate (release): `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --out DIR` (S in 7 42 11 99); town: add `--people 60 --days 30`. HEAD = a detached worktree of `main` at pass start. "brake+Power" = `--friend-capacity 10` plus `snub-power-1.15.patch`. Host load high; no timings.

| run | insults HEAD -> brake+Power | accepted compliments | rivals | friends |
|---|---|---|---|---|
| village s7 | 343 -> 202 | 1005 -> 1334 | 54 -> 39 | 46 -> 66 |
| village s42 | 382 -> 159 | 1082 -> 1473 | 56 -> 25 | 30 -> 90 |
| village s11 | 348 -> 186 | 1101 -> 1325 | 59 -> 37 | 40 -> 72 |
| village s99 (fresh) | 361 -> 167 | 1151 -> 1467 | 66 -> 30 | 35 -> 80 |
| town60 s7 | 1740 -> 781 | 5746 -> 6956 | 277 -> 49 | 49 -> 249 |
| town60 s99 (fresh, brake+Power only) | 729 | 6856 | 30 | 255 |

Brake alone on `Linear` (shipped pack): village s7 336/35/55, s99 371/34/68 (insults/friends/rivals), town s7 1729/42/331: no gain, town rivals worse. Exponent sweep, town s7, brake on: 1.1 -> 52 friends / 140 rivals / 1558 insults; 1.15 -> 249 / 49 / 781; 1.2 -> 651 / 0 / 7; 1.3 -> 576 / 5 / 67.

Default off is bit-identical to HEAD: `summary.json` of village s7 equal byte for byte. Gates: `label_hysteresis` 9/9 unmodified and `friendship_bound` with brake+Power; with `Power(1.15)` in the pack `tests/rumor_book.rs::the_book_accumulates_rumors_nobody_can_tell` fails (735 rumors, 289 tellable), which is why the pack is unchanged. Final default state: full `cargo test --release --all-features`, `--no-default-features`, clippy, fmt, doc (see log). Plot: `evidence/plots/before-after.png`.

# 2026-10-28-09 — does a delivered retaliation settle the grievance and calm the town? No (measured, not shipped)

Open issues: #24, #27 (release; skipped). No bug issue. Item: last pass's top remaining item, the retaliation loop. Research: `research.md` (Gollwitzer, Meder & Schmitt 2011, DOI 10.1002/ejsp.782).

**Trial.** `Tuning::revenge_settles` f: on an accepted `harm < 0` exchange the actor's bad memories about the target are scaled by `1 - f` (`Memories::settle`). Kept as `revenge-settles.patch` (crate, `examples/evidence --revenge-settles`); the tree was reverted, nothing ships. f = 0 is bit-identical to HEAD `90fd543` (its rows reproduce `2026-10-28-08`'s 35/28/36 rivals).

Regenerate: `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --revenge-settles F --out DIR` (and `--people 60 --days 30` for the town). Summaries: `evidence/r<F>/{s7,s42,s11,town60}/summary.json`.

| f | friends s7/s42/s11 | rivals s7/s42/s11 | insults | victims who dislike insulter | town60 friends / rivals / insults |
|---|---|---|---|---|---|
| 0.0 | 21 / 22 / 20 | 35 / 28 / 36 | 390 / 369 / 415 | 347 / 325 / 369 | 29 / 170 / 1875 |
| 0.5 | 21 / 18 / 18 | 21 / 37 / 35 | 369 / 375 / 386 | 312 / 332 / 324 | 33 / 219 / 1843 |
| 1.0 | 16 / 15 / 12 | 39 / 46 / 49 | 389 / 427 / 445 | 335 / 367 / 381 | 33 / 181 / 1808 |

## Finding

The hypothesis is **falsified**. Settling the actor's grievance after a delivered insult does not lower the insult count (390 → 369 → 389 at seed 7; 1875 → 1843 → 1808 at 60 people, within noise) and does not lower the share of insults from someone the victim dislikes (89% → 85% → 86%). Rivals go **up** at f = 1.0 (mean 33.0 → 44.7) and friends go down (21.0 → 14.3): clearing the insulter's grudge frees it to move on, while the victim's fresh grievance is untouched, so a quarrel is no shorter and the tie damage is the same.

Why: a snub is scored on the grudge of the **snubber**, and most grudges are not an answer to a single insult. The 85% "victim already disliked the insulter" share counts grudges held for any prior memory, including refused approaches and witnessed deeds, none of which this settles. The loop is not insult-for-insult retaliation; it is a standing dislike that is re-fed by every contact.

## Next lead

Measure what writes the grudge before changing how it is spent: tally the negative memories that sit behind each snub at decision time by kind (accepted insult received / refused approach / witnessed deed / rumor), at 24 and 60 people.

Verdicts: not run (nothing shipped). Dead-code delta: zero.

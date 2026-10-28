# 2026-10-28-08 — is the rival-saturated town the refusal penalty? No (measurement only)

Open issues: #24, #27 (release; skipped). No bug issue. Item: top remaining item of the last pass, "why the default town rivals itself". Hypothesis tested: `rejection_affinity_penalty` (0.05, paid by the approacher on each refusal; 38% of approaches are refused) feeds rivalry.

Harness change only: `examples/evidence --rejection-penalty P` overrides `Tuning::rejection_affinity_penalty`. No crate code or default changed.

## Sweep (HEAD `51d8813`, `village-evening`, 24 people, 17 game-days)

Regenerate: `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --rejection-penalty P --out DIR`. Summaries: `evidence/pen<P>/s<S>/summary.json`.

| penalty | friends (s7/42/11) | rivals (s7/42/11) | became_rival | left_rival | insults |
|---|---|---|---|---|---|
| 0.05 (default) | 21 / 22 / 20 | 35 / 28 / 36 | 108 / 130 / 130 | 38 / 73 / 57 | 390 / 369 / 415 |
| 0.02 | 15 / 24 / 23 | 33 / 33 / 30 | 120 / 126 / 113 | 53 / 59 / 53 | 446 / 442 / 424 |
| 0.00 | 20 / 22 / 23 | 28 / 25 / 29 | 115 / 110 / 124 | 58 / 60 / 66 | 488 / 455 / 461 |

Mean rivals 33.0 → 31.9 → 27.3; friends flat. Removing the penalty entirely lowers rivals about 17% and raises insults ~20%, so **refusal is a minor contributor, not the cause**; refusals stay ~1600 either way (they come from acceptance odds, not the penalty).

## 60 people, 30 days, seed 7 (`evidence/town60/`)

2 466 interrupted vs 27 438 completed endings; 1 089 `became_rival` vs 748 `left_rival`; 1 875 insults, of which **1 588 (85%) came from someone the victim already disliked** (`victims_who_dislike_their_insulter`), against 5 798 accepted compliments. The harness reports 170 rival ties here against `tests/friendship_bound`'s 1157; the two counts were not reconciled this pass (different homes/place layout is the likely but unchecked reason).

## Finding

The rivalry engine is retaliation, not refusal: an insult writes a grudge into the victim (negativity ratio 5), `snub` is scored on `ctx.grudge(actor, target)` alone, so the victim snubs back. Next lead: a decay or reciprocity cap on that loop (Bondü & Richter 2016, DOI 10.3389/fpsyg.2016.00795, already cited by `snub`), measured as snubs-from-a-victim per insult received.

Verdicts: not run (no behaviour changed). Dead-code delta: zero.

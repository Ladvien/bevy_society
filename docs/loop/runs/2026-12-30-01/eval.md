# 2026-12-30-01 — a friend-capacity brake, measured and not shipped

Open issues: #24, #27 (release; skipped). No bug issue. Item: the top remaining item of passes 04-07, "a friendship brake independent of rivalry, then retry the snub rate" (`needs-decision/snub-rate-vs-scarce-friendship.md`, option 2).

## Trial

`Tuning::friend_capacity` (10): a tie that is not yet a friend gains positive affinity, by interaction and by mere exposure, at `1 - friends/capacity` of its rate. Ten is the layer above a 3.3-person support clique if layers scale by about three (Zhou, Sornette, Hill & Dunbar 2005, DOI 10.1098/rspb.2004.2970); a design default derived from that ratio. The full diff is `friend-capacity.patch` (reapply with `git apply`). Not shipped: see below.

## Measurements (HEAD `308c85a` plus the patch; `village-evening`, 17 game-days, 24 people)

Regenerate: `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --out DIR` (summaries committed under `evidence/<cell>/sS/summary.json`).

| cell | seed | compliments : insults | friends | rivals |
|---|---|---|---|---|
| before (HEAD) | 7 / 42 / 11 | 1116:390 / 1093:369 / 1092:415 | 21 / 22 / 20 | 35 / 28 / 36 |
| snub `Power(1.2)`, no brake | 7 / 42 / 11 | 1489:73 / 1867:0 / 1543:41 | 80 / 106 / 86 | 7 / 0 / 5 |
| snub `Power(1.2)` + brake | 7 / 42 / 11 | 1458:71 / 1456:56 / 1913:2 | 61 / 65 / 85 | 11 / 8 / 0 |
| `Linear` + brake | 7 / 42 / 11 | 975:347 / 1072:415 / 1077:402 | 18 / 11 / 14 | 30 / 40 / 37 |

## `tests/friendship_bound.rs` (60 people, seed 7, 30 game-days)

`cargo test --release --test friendship_bound -- --nocapture` prints the line quoted here.

| snub curve | brake | friend ties of 3540 | rival ties |
|---|---|---|---|
| `Linear` (HEAD) | off | 62 (1.8%) | **1157 (32.7%)** |
| `Linear` | 10 | 44 (1.2%) | 1590 (44.9%) |
| `Power(1.1)` | 10 | 60 (1.7%) | 1267 (35.8%) |
| `Power(1.15)` | 10 | 87 (2.5%) | 426 (12.0%) |
| `Power(1.2)` | off | 3216 (90.8%) | 0 |
| `Power(1.2)` | 10 | **776 (21.9%)** | 0 (**fails `rivals >= 1`**) |

## Findings

1. **The brake does what it was built for.** At `Power(1.2)` it takes friendship from 90.8% to 21.9% of ordered pairs, under the 25% bar.
2. **The shipped town is not friend-saturated; it is rival-saturated.** At 60 people HEAD has 1.8% friend ties and 32.7% rival ties. The needs-decision file framed the 60-person bar as "friendship too plentiful"; with HEAD's `Linear` snub the opposite holds, and the brake removes friends (62 → 44) so rivals rise (1157 → 1590).
3. **The snub exponent has no stable middle at 60 people.** Rivals go 1157 (1.0), 1267 (1.1), 426 (1.15), 0 (1.2): non-monotonic on one seed, so no exponent reads as a value to ship, and `rivals >= 1` fails at 1.2 with the brake on.
4. **At 24 people the brake barely binds** (3-4 friends a person against a capacity of 10): it lowered friends 80 → 61 and 106 → 65, and left one seed (11) with 0 rivals.

## Decision

Nothing shipped: the patch trades one failing bar for another (`Power(1.2)` + brake passes 25% and fails `rivals >= 1`; HEAD passes both while 33% of a 60-person town are rivals). Tree reverted; `friend-capacity.patch` kept. Next lead: why a 60-person town rivals itself at the default snub (`became_rival` vs `left_rival` over 30 days at 60 people), which is a larger defect than the compliment:insult ratio.

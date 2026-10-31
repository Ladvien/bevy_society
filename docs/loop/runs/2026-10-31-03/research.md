# Research - a person does not go back to someone who has just turned them down

## The defect, measured

Village (24 people, seed 7, 17 game-days at 64 s/day, the shipped pack, no chronotype offsets; `evidence/before/v7/reapproach.txt`, produced by the new `ReapproachLedger` in `examples/evidence.rs`):

| after an approach that was... | refused (1 569) | accepted (2 890) |
|---|---|---|
| followed within 1 h by another approach to the same person, not an insult | **29.0 %** | 41.9 % |
| ...of the same interaction | 17.9 % | 27.2 % |
| followed within 24 h, not an insult | **49.6 %** | 67.9 % |
| median gap to the next approach (any) | 3.1 h | 1.1 h |

A refusal lowers the chance of going back to that person by a quarter (29.0 against 41.9 %), not by most of it. Narrated (`docs/loop/runs/2026-10-31-02/evidence/narratives/owl-quill-s7-6days.txt`, `--track 16`, six game-days): `gossip with Garet (refused)` at 20:38, 21:27, 22:43, 23:01, 23:19 and 23:37 on day 2, six refusals in three hours by the same person to the same person for the same thing; `compliment with Rhea / Sable / Ulric / Tomas (refused)` in the first hour of day 1.

## Why the crate does this

`decide_activities` scores every neighbour afresh each decision and remembers nothing about who turned the person down:

- `gossip`'s only per-target term is `Distance` (`assets/starter.society.ron`): the nearest neighbour wins every time, whatever the last answer was. A person's nearest neighbour who keeps refusing is chosen again and again.
- `compliment` reads `Affinity` only, through `Logistic(16, 0.25)`.
- The one consequence of a refusal is `Tuning::rejection_affinity_penalty` (0.05 on the refused person's tie to the refuser) and a small `Distress`. 0.05 is a nudge on a logistic centred at 0.25; `chat` reads affinity through `Linear(0.8, 0.4)`, so a refusal costs a chat 0.04 of its score, and `play`'s affinity term is flat (`Linear(0.0, 0.8)`), so a refusal costs it nothing. `docs/loop/log.md` 2026-10-28-08 measured that term as not what drives the town's rivalries; this note does not revisit it.
- `Tie` has no "last exchange" stamp that a refusal could set (`last_contact_day` is written by mere exposure as well), the gap `docs/research/relationships.md` already records.

`FailedPlaces` (a walk that ended before arrival) and `RecentHarm` (the retaliation refractory: one blow per grievance per period) are the crate's two per-person avoid-lists. Neither covers an approach that was turned down.

## Sources

- **Maner, DeWall, Baumeister & Schaller 2007**, *Does social exclusion motivate interpersonal reconnection? Resolving the "porcupine problem"*, Journal of Personality and Social Psychology 92(1), DOI `10.1037/0022-3514.92.1.42`. Abstract (read through `paper_search`; the full text is not in the corpus, `paper_download` found no open copy): six experiments; threat of exclusion led participants to express greater interest in making **new** friends, to want to work with others, to form more positive impressions of novel targets and to reward new partners; "**Excluded individuals did not seem to seek reconnection with the specific perpetrators of exclusion**" or with new partners they did not expect to meet; fear of negative evaluation moderated it (high-FNE participants did not respond affiliatively). This is the mechanism: after being turned down a person turns toward other people and not back toward the one who turned them down. Reconnection is with *new* sources of affiliation.
- **Downey & Feldman 1996**, *Implications of Rejection Sensitivity for Intimate Relationships*, JPSP 70(6), DOI `10.1037/0022-3514.70.6.1327` (in the corpus, `doc_id 10.1037_0022-3514.70.6.1327`, page 1): "Responses to perceived rejection include hostility, dejection, emotional withdrawal, and jealousy", and people who anxiously expect rejection "readily perceive intentional rejection" in ambiguous behaviour. The direction is withdrawal, plus hostility; the reference list of that passage includes **Baumeister & Leary 1995**, *The need to belong*, Psychological Bulletin 117(3), DOI `10.1037/0033-2909.117.3.497`, which this crate already cites for the need `social`.
- **Bondü & Richter 2016**, DOI `10.3389/fpsyg.2016.00795` (cited in the pack at `snub`): rejection sensitivity predicts *reactive aggression*. Hostility after a rejection is therefore a separate channel from withdrawal, and it already has its own gate here (`Tuning::retaliation_refractory_hours`). **The memory in this note covers affiliative approaches only**; a harmful one (an `insult`) is left to that refractory.
- **Stenseng, Belsky, Skalicka & Wichstrom 2014**, DOI `10.1177/0146167214554591` (abstract read): exclusion in preschool predicted more aggression and less cooperation two years on, supporting the need-to-belong reading against a blanket "reconnection" one. A boundary on the claim: what Maner et al. find is specifically *not approaching the perpetrator*; it is not that every excluded person turns warm.
- **Not found, and said so:** no paper in the corpus or in the search gives an interval ("how many hours before a person tries the same one again"), and none gives a rate of re-approach after a refusal in a population. The interval is a **design default**, chosen by measurement below.

## The model

A person who is **refused** (by the target's acceptance draw; not by a `requires` rule) in an approach that carries no harm remembers the one who refused for `Tuning::rebuff_memory_hours` game-hours. While it lasts, that person is not a candidate target of any *affiliative* `Person` activity (`chat`, `gossip`, `compliment`, `play`, `share`, anything a pack adds whose deed is not harmful). It is the same shape as `FailedPlaces` and `RecentHarm`: a per-person `Vec<(Entity, until_day)>`, filtered *after* `targets_for` so the RNG draws are the same whether or not anything is remembered, pruned on write, saved, and `None` turns it off and is exactly the old curve.

What it is deliberately not:

- **Not a tie change.** The acceptance model, `rejection_affinity_penalty` and the labels are untouched.
- **Not a ban on the pair.** The memory expires; after it, the person is a candidate again at whatever the tie scores.
- **Not applied to harm.** A refused `insult` already starts the retaliation refractory; Bondü & Richter says the reaction to a rejection can be hostile, so hostility is not suppressed here.
- **Not a rule for the host.** A host's own `Interact` is never refused by it; it only steers the crate's choices.

## What believable output looks like (predictions, to be struck if wrong)

1. Within the memory window almost no affiliative approach goes to someone who has just refused: the row "not an insult within 1 h" of the table above falls from 29 % toward the share that is a *different* person's approach (zero for this ledger) plus any approach by the target to the actor (a different ordered pair, not counted).
2. Beyond the window the share returns toward the accepted row: people do go back, later.
3. The refusal rate (35.2 % of approaches in this pack, `docs/loop/log.md`) falls, because the pairs that refuse most are the ones no longer retried; refusal distress falls with it, so mood rises a little.
4. The number of distinct people a person approaches in a game-day rises (the approach goes elsewhere), with no change in the number of approaches.
5. Insults are not suppressed: the count stays within noise.
6. Nothing that is not an affiliative approach changes: sleep, work, eating, bakes are the same decisions.

Prediction 3 is the one most likely to be wrong; the harness reports it either way.

## Not done in this pass

The other half of the same narrative, `compliment` six times to the same person in forty minutes **after acceptances** (34 % of exchanges repeat the previous partner), is satiation, not rebuff: a different mechanism with a different source (the accepted row above, 41.9 % within one hour). It stays on the list.

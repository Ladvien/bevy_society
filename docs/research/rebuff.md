# Rebuff: a person does not go back to someone who has just turned them down

`Tuning::rebuff_memory_hours` (`src/core.rs`), `Rebuffed` (`src/behavior.rs`, written in `exchange`, read in `decide_activities`), `PersonSave::rebuffed` (`src/save.rs`). Run notes with the evidence: `docs/loop/runs/2026-10-31-03/`.

## The problem

`decide_activities` scores every neighbour afresh at every decision and remembers nothing about who said no. The crate's one consequence of a refusal is `rejection_affinity_penalty` (0.05 off the refuser's own tie to the refused) and a small `Distress`: a nudge on a logistic, and nothing at all on `gossip`, whose only per-target term is `Distance`, or on `play`, whose affinity term is flat. So the nearest neighbour who keeps refusing is chosen again at the next decision.

Measured on the village at seed 7 (24 people, 17 game-days at 64 s/day, the shipped pack; `docs/loop/runs/2026-10-31-03/evidence/rebuff.md`):

- **29.0 %** of refused approaches were followed within an hour by another affiliative approach to the same person (24-29 % in the four villages and the three towns), against 42 % of accepted ones: a refusal cut the chance of going back by a quarter, not by most of it. 49.6 % were followed within a day (39-50 % in all seven runs).
- A narrated person (`--track 16`, `docs/loop/runs/2026-10-31-03/evidence/narratives/owl-quill-s7-6days-before.txt`) asked the same neighbour for gossip at 20:38, 20:56, 21:27, 22:43, 23:01, 23:19 and 23:37 on one evening: six of seven refused.
- 30.5-34.4 % of all exchanges were with the initiator's previous partner.

## Sources

- **Maner, DeWall, Baumeister & Schaller 2007**, *Does social exclusion motivate interpersonal reconnection? Resolving the "porcupine problem"*, JPSP 92(1), DOI `10.1037/0022-3514.92.1.42`. Abstract read through `paper_search`; the full text is not in the corpus and `paper_download` found no open copy. Six experiments: threat of exclusion raised interest in making **new** friends, in working with others, in forming positive impressions of novel targets and in rewarding new partners; "Excluded individuals did not seem to seek reconnection with the specific perpetrators of exclusion." Fear of negative evaluation moderated it. This is the direction of the mechanism: turn away from the person who refused, toward other people.
- **Downey & Feldman 1996**, *Implications of Rejection Sensitivity for Intimate Relationships*, JPSP 70(6), DOI `10.1037/0022-3514.70.6.1327` (in the corpus): "Responses to perceived rejection include hostility, dejection, emotional withdrawal, and jealousy". Withdrawal is the response modelled; hostility is a separate channel.
- **Bondü & Richter 2016**, DOI `10.3389/fpsyg.2016.00795` (already cited at the starter pack's `snub`): rejection sensitivity predicts reactive aggression. Hence a harmful approach is **not** held by this memory; `retaliation_refractory_hours` holds it.
- **Stenseng, Belsky, Skalicka & Wichstrom 2014**, DOI `10.1177/0146167214554591` (abstract read): exclusion in preschool predicted more aggression and less cooperation two years on. A boundary on the claim: what Maner et al. show is not approaching the *perpetrator*; it is not that every excluded person turns warm.
- **No source gives an interval** and none gives a rate of re-approach in a population. The 12 hours is a design default chosen by measurement.

## The model

A person who is refused (the target's acceptance draw says no; a `requires` rule refusing it writes nothing) in an approach that carries no harm (no deed, or `harm >= 0`) records `(refuser, now + h/24)` in `Rebuffed` for `h = rebuff_memory_hours`. While it lasts, `decide_activities` drops that person from the targets of every non-harmful `Person` activity, after `targets_for`, so the RNG draws are the same whether or not anything is remembered. Entries expire; a write prunes the expired and the repeated. A save carries it by `PersonKey` (`rebuffed`, `#[serde(default)]`, `VERSION` unchanged). `None` is exactly the old curve: at seed 7 the harness's `summary.json`, `narrative.txt` and `series.csv` equal the previous build's byte for byte.

Deliberately not:

- **Not a tie change.** The acceptance draw, `rejection_affinity_penalty` and the labels are untouched. The refused share of approaches does not move (35-37 % in the villages, 37-40 % in the towns).
- **Not a ban on the pair.** The memory expires.
- **Not applied to harm.** A refused insult is held by the retaliation refractory.
- **Not a rule for the host.** A host's `Interact` is never refused by it.
- **Per person, not per activity.** Maner et al.'s excluded participants did not seek out the perpetrators at all, not just for the same request.

**Off by default, and why.** `rebuff_memory_hours` is `None` in `Tuning::default()`; `Some(12.0)` is the value measured. At 12 hours it trips `tests/label_hysteresis.rs::the_band_stops_the_low_end_flapping_and_moves_no_relationship` by one event: that test bars the share of `Acquaintance → Stranger` demotions a label band leaves at 5 % of those without a band, and the fixture reads 16 against 311 (5.14 %). The same fixture with the memory off reads 12 against 317 (3.8 %), and across seven memories 2.8-6.0 % (0 / 3 / 6 / 9 / 12 / 18 / 24 h: 3.8 / 3.9 / 5.6 / 6.0 / 5.1 / 2.8 / 3.7 %). The with-band count is 9-16 events, mean about 13, so the bar is a Poisson count at its edge and the memory only changes which trajectory the village takes; it is not evidence that the band works less well. A protected check is not loosened by the loop, so the memory is opt-in; `docs/loop/needs-decision/a-refusal-is-remembered.md` and `docs/loop/runs/2026-10-31-03/rebuff-default.patch` (the one-line default and the bar at 8 %) switch it on.

## What it did (same seeds before and after; `evidence/rebuff.md`, `evidence/plots/rebuff.png`, `sweep.png`)

| | before | after, 12 h |
|---|---|---|
| refused approach followed within 1 h by another to the same person, not an insult (4 villages, 2 towns of 60, 1 of 40) | 24-29 % | **0.3-1.1 %** |
| ...within 24 h | 39-50 % | 13-20 % |
| different people approached per person-day (villages / towns) | 4.1-4.4 / 4.7-5.1 | **5.2-5.4 / 5.4-6.3** |
| exchanges with the initiator's previous partner | 30.5-34.4 % | 21.5-25.3 % |
| refused share of all approaches | 35-40 % | 35-40 % (unchanged) |
| approaches per person-day | 9.3-10.4 / 9.7-12.2 | 9.2-9.4 / 8.9-10.9 |

Sweep at village seed 7 (0 / 3 / 6 / 12 / 24 h): refused approaches followed within 1 h 29.0 / 0.9 / 1.3 / 0.3 / 0.4 %, within 24 h 49.6 / 28.3 / 24.9 / 18.3 / 1.8 %. At 24 h there is no retry at all within the day, which treats a refusal as a standing one; at 12 h a person refused at midday is a candidate again the next morning.

**What did not move, and a prediction that was wrong.** The prediction (run note `research.md`, 3) that the refused share would fall because the pairs that refuse most are no longer retried was struck: it is 35.2 → 36.6 % at seed 7 and flat everywhere, because the acceptance draw reads the *target's* tie to the actor, mood and personality, and a retry changes none of them (an earlier measurement in `docs/loop/log.md`: mean acceptance probability 0.656, mean affinity term +0.058, the refusal rate set by `accept.base`). The memory changes whom a person asks, not how often they are refused.

**What moved that is not claimed as mechanism.** Friend ties rose in all seven pairs (48 → 86, 66 → 69, 51 → 61, 69 → 70, 128 → 168, 106 → 140, 103 → 107) and insults fell in five of seven (301 → 229, 288 → 247, 1 442 → 1 309, 1 425 → 1 302, 527 → 503; 227 → 262 and 252 → 267 rose); rival ties and mean mood moved both ways (`evidence/rebuff.md`). Each is one seed's trajectory of a chaotic society, so these are directions at most.

**The cost.** With fewer retries a person with few neighbours runs out of people to ask: approaches per person-day fall 9-11 % at village seed 7 and in the towns (12.2 → 10.9, 11.8 → 10.8, 9.7 → 8.9) and 5 % at seeds 11 and 99 (one village, seed 42, rises 2 %), and the run's mean `social` need is 0.007-0.025 lower in five of seven pairs (0.786 → 0.773, 0.784 → 0.771, 0.773 → 0.748 at seed 7; two villages are 0.003-0.006 higher) and `fun` 0.005-0.031 lower in five of seven; both stay far above any floor.

## Not modelled

- Satiation after an *accepted* approach (six compliments to one person in forty minutes): a different mechanism, left open.
- Why the person refused (attribution), the refuser's side of the pair (the one who refused does not remember) and rejection sensitivity as a trait (Downey & Feldman's individual difference): the memory is the same for every person.
- A refusal's effect on approach *motivation* toward strangers (Maner's reconnection with new partners): the model removes a candidate; it does not raise the score of the others.

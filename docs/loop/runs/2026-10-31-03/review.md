# Review - `Tuning::rebuff_memory_hours`

Evidence cited is committed: `evidence/rebuff.md`, `evidence/plots/{rebuff,sweep,over-time}.png`, `evidence/{before,after,sweep}/*/reapproach.txt`, `evidence/narratives/*.txt`, `eval.md`.

## 1. Behavior

**Critique.** The source says one thing about the person who has just been turned down: they do not seek reconnection with the one who excluded them (Maner et al. 2007), and the response to rejection is withdrawal (Downey & Feldman 1996). The measured defect was that a refusal changed almost nothing about who is asked next: 29.0 % of refused approaches were followed within an hour by another affiliative approach to the same person against 41.9 % of accepted ones (village seed 7). After, that is 0.3 % at seed 7 and 0.3-1.1 % in all seven runs (`rebuff.md`, column 5), and the share that comes back within a day is 13-20 % where it was 39-50 %; the accepted row does not move (`rebuff.md`, column 6), so the change touches refusals and not conversation in general. The owl's evening of seven asks of one neighbour (`narratives/owl-quill-s7-6days-before.txt`, 20:38-23:37) is one ask in the after file. The reading as a stranger is: she asked, was refused, did something else (`person3-s42-6days-after.txt`, 12:55 refused by Edric, 13:08 chat with Quill accepted).

What the review does not let through: (1) the prediction that the refused share would fall is wrong and is struck in `eval.md` and `docs/research/rebuff.md` (35.2 → 36.6 %); (2) only half of Maner's finding is modelled, the turning away; the other half, more interest in *new* partners, is not, so a person with few neighbours runs out of people and does something else (approaches per person-day -4.5 to -11 % in six of seven, mean `social` and `fun` needs 0.005-0.03 lower in five of seven); (3) 12 hours is a design default, and the sweep shows the friend counts are noise between 3 and 24 h (89 / 58 / 86 / 81), so the default rests on the re-approach profile alone (24 h leaves 1.8 % retrying within a day, 3 h 28 %).

**Verdict: GREAT.** The targeted behaviour is removed in seven of seven runs with the cost named and small, the control row is untouched, and `--no-rebuff` is the old society byte for byte.

## 2. Zoom

**Critique.** `plots/rebuff.png`: the red bars (before) are never under 24 % at 1 h and climb to 39-50 % at 24 h; the blue (after) are flat near zero until 24 h, where they reach 13-20 %, a step at the memory length and not a ramp, which is the shape a hard expiry should draw. `plots/sweep.png`: the 24 h point falls to 1.8 % and no further, the refused share is flat on a 0-50 axis (the first version of this panel clipped the axis to 34.7-36.6 and read as a hill; it was redrawn), friends and insults are jagged across hours at one seed. `plots/over-time.png` (village seeds 7 and 42, towns of 60 seed 7 and 40 seed 3): the two arms are the same for the first day and a half, then the after line for friends sits above the before line for most of each run (village 42 only narrowly, and near level for its last four days) and rivals do not separate consistently (village 42's rivals end above, the 40-person town's below); the daily mood cycle has the same amplitude in both arms, no new oscillation, no saturation at the ceiling, no drift in either, no spike at a tick boundary. One thing the plots cannot say is why friends are higher: the mechanism is not established and the changelog does not claim one.

**Verdict: GREAT.** No artifact found in any panel; the two limits named above (jagged sweep, rivals mixed) are in the changelog and the research note as limits.

## 3. Fresh eyes

**Critique.** Regenerated at seeds and populations the work was not tuned on: villages at seeds 42, 11 and 99 (the sweep was seed 7 alone), towns of 60 at seeds 7 and 99 and a town of 40 at seed 3 (20 game-days), and a narrative for a person at seed 42 (`person3-s42-6days-*.txt`) the earlier pass never read. Every one shows the same thing: refused-then-asked-again within the hour 24-29 % → 0.3-1.1 %, partner repetition 30.5-34.4 % → 21.5-25.3 %. The one re-approach left in person 3's six days is an insult twelve minutes after a refused compliment, which is the case the memory leaves out on purpose (Bondü & Richter 2016); a stranger reading the file sees a person turn nasty after being turned down, and that is a believable and separately handled sequence. Explaining it to a host developer without the code: "if someone says no, you don't ask them again for half a day, and you ask someone else; you can still lash out at them." That is a sentence in the guide (`docs/guide/concepts.md`) and in the field doc.

What a skeptic would still ask: does it generalise past the starter pack's `chat`/`gossip`/`compliment`? The filter reads the interaction's deed and the activity's `Person` target, not any name, so a host's pack with its own affiliative activities is covered; `tests/rebuff.rs` pins it on a pack that names only `chat` and `insult`.

**Verdict: GREAT.**

## What the verdicts do not cover: the default

All three verdicts are about the behaviour at `Some(12.0)`. The memory ships **off** because that value fails one protected check (`label_hysteresis`'s 5 % bar, 16 against 311 events, 5.14 %); the probe in `eval.md` section 6 shows the check is a Poisson count at its edge (2.8-6.0 % across seven memories, 3.8 % with it off), not a finding about the memory. That is a decision for the owner, recorded with a patch that was run (`needs-decision/a-refusal-is-remembered.md`, `rebuff-default.patch`), not one this pass took by loosening a pin or by choosing 18 or 24 hours for passing it.

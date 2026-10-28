# 2026-10-28-09 research — a grievance answered is a grievance settled

Open issues #24, #27: release issues, skipped. No bug issue. Item: last pass's top remaining item, the retaliation loop (85% of insults at 60 people come from someone the victim already dislikes).

**Mechanism today.** An accepted insult writes a negative memory into the victim. `snub`'s only consideration is `ctx.grudge(actor, target)` (max over negative memories about the target, recency weighted, half-life ~138 h). Answering the insult does not touch that memory, so the victim keeps snubbing the same person every few ticks until the memory decays, independent of how many times the grievance was already answered; each snub then seeds the same loop in the other direction.

**Sources.**
- Gollwitzer, Meder & Schmitt 2011, *What gives victims satisfaction when they seek revenge?*, Eur. J. Soc. Psychol. 41(3):364-374, DOI 10.1002/ejsp.782. Revenge satisfies, and justice counts as restored, only when the offender **understands** the act as a response to the earlier one; mere suffering of the offender does not. Avengers felt better than non-avengers only when understanding was signalled. Model reading: a delivered retaliation (the `insult` was **accepted**, so the offender received it) closes the grievance; a refused one delivered no message and the grievance stays open (already how refusal works: it writes no injury and no relief).
- Bondü & Richter 2016, DOI 10.3389/fpsyg.2016.00795 (already cited by `snub`): reactive aggression is a reaction to how one was treated, i.e. it is **about a specific provocation**, which is what makes it answerable.
- Counter-evidence, kept in view: Carlsmith, Wilson & Gilbert 2008, DOI 10.1037/a0012165, find punishing does not by itself end rumination (avengers feel worse than expected). So the settlement is **partial by design** and is a swept design default, not 1.0 by assertion.

**Formula.** On an accepted exchange whose deed has `harm < 0`, each of the actor's memories about the target with `valence < 0` is multiplied by `1 - revenge_settles`. `revenge_settles` in `[0, 1]`; the sources give the mechanism and no number, so the default is a design default chosen by sweep (0 = behaviour before this change).

**Believable output.** Snubs per insult received fall toward one reply per provocation (a turn-taking quarrel, not a continuous barrage); the rival count at 60 people falls without friends being touched; refused insults still deepen nothing; a stranger-to-stranger insult changes nothing because there is no grudge to settle.

# Rumour and reputation

## What bevy_society does

A rumour is a record about one deed by one person. It has an origin, a subject, a deed with a harm value, a truth flag and a count of people who know it. Each person's knowledge of a rumour carries a belief and a spreader flag. A person tells a rumour to a contact who does not yet know it. The listener takes on a belief that scales with the teller's belief and the trust between them. A listener who already knows the rumour is stifled, and the teller stops spreading it. Witnesses to a deed react with approval or disapproval, which feeds reputation through image scoring or standing. A norm violation (approval below −0.5) emits a `NormViolated` message, and a rumour is planted when the harm passes `rumor_threshold` or any witness reacts strongly. The model functions are on HEAD (`src/model/rumor_dk.rs`, `src/rumor.rs`, `src/beliefs.rs`, `src/reputation.rs`); the witnessing and gossip wiring is D5/D7 work.

## Equations as implemented

- Telling weight (`src/model/rumor_dk.rs:28–30`): `w = |valence| · belief · recency(h)`, where `recency` is the function in `memory.md` and `h` is hours since the memory was created. Selection picks the highest-weight rumour whose subject is not the listener (`src/beliefs.rs:175–178`).
- Belief on hearing (`src/model/rumor_dk.rs:38–40`): `belief_listener = teller_belief · (0.5 + 0.5 · clamp(trust, 0, 1))`. Design default (no source paper gives this coefficient); mechanism: Sommerfeld et al. (2007).
- Stifling (`src/model/rumor_dk.rs:17–22`): on contact, if the listener already knows the rumour, the teller stops spreading it. This is the classic Daley–Kendall rule, equivalent to stifling probability α = 1.
- Gossip and reputation thresholds: `rumor_threshold` = 0.3 (`src/core.rs`, design default), `gossip_trust_gain` = 0.3 and `gossip_affinity_gain` = 0.2 (design defaults), `witness_gain` = 0.2 (design default). Mechanism: Sommerfeld et al. (2007) for gossip carrying reputation.
- Norm violation: `NormViolated` is written when a witness's approval `a < −0.5` (specified for D7).
- Reputation assessment (`SocietySettings.assessment`, default `Standing`): either image scoring or standing. The witness approval `a` moves the actor's trust and respect under the chosen assessment.

- **The victim's write, and the ratio.** `patient_reaction` moves affinity by
  `witness_affinity · severity`, where

  ```text
  severity = harm                       if harm >= 0.0
           = harm · negativity_ratio    if harm <  0.0
  ```

  `witness_affinity` = 0.15 (design default; it was the literal inside `witness_reaction` until 2026-10-27-03)
  and `negativity_ratio` = 5.0. **Baumeister, Bratslavsky, Finkenauer & Vohs 2001** (DOI
  `10.1037/1089-2680.5.4.323`), reporting Gottman 1994, p. 33: *"positive and good interactions must outnumber
  the negative and bad ones by at least five to one … Gottman's index suggests that bad events are on average
  five times as powerful as good ones"*. The reciprocity half is the same review at p. 22, citing Wills et al.
  1974: *"negative, displeasurable behaviors were reciprocated to a significant degree, whereas the
  reciprocation of positive, pleasurable behaviors was weaker and not significant"*.

  Before this existed the victim's affinity for the person who had just hurt them was set entirely by mere
  exposure: measured on `village-evening` (seed 7, 12 000 ticks), every accepted `insult` left it at **+0.034**,
  positive, beside the insulter's +0.131. A victim went on liking their insulter, and the more often the two met
  the more they liked them.

  **The patient reciprocates and the witness judges**, which is why only one of them reads an opinion. A
  witness's affinity is `witness_affinity · approval` with `approval = norm · sign(harm)` — the deed weighed
  against the witness's own `Opinions`. A patient's is `witness_affinity · severity`, straight from the harm.
  Reading the patient's opinion instead would make the write silently zero for anyone whose `Opinions` hold no
  entry on that topic, because `Opinions::value` returns 0.0 when unsized.
- **Witness dilution.** A deed's per-witness trust cost is divided by `witnesses ^ Tuning.witness_dilution` (default 0.5). Sommerfeld et al. 2007 (DOI `10.1073/pnas.0704598104`) measured that gossip transmission dampens — *"the more extreme a gossip was, the less impact it had on the reader"* — and cited Hess & Hagen on believability rising with the number of sources: the same news, believed by more people and weighing less in each. Measured, one witnessed insult costs each witness **0.0800** with one bystander and **0.0431** with five; without dilution it was 0.0800 in both and the aggregate grew with the crowd while a trust gain could only reach two people.

## Why these defaults

- **Telling weight.** Park et al. (2023) weight memories for retrieval by importance, relevance and recency. The code's telling weight takes the recency and importance parts; relevance is replaced by the teller's belief. Park's text: "we treat recency as an exponential decay function over the number of sandbox game hours since the memory was last retrieved. Our decay factor is 0.995." (`10.48550_arxiv.2304.03442`, converted; passage checked). Departure: the crate measures from creation, not retrieval (see `memory.md`).
- **Daley–Kendall stifling.** Moreno, Nekovee & Pacheco (2004) describe the original rumour model as "due to Daley and Kendall", with stifling when a spreader meets a spreader or stifler (`10.1103_physreve.69.066130`, converted; passage checked). They generalise the stifling step to probability α. The code is the α = 1 case (see below).
- **Gossip and reputation.** Sommerfeld et al. (2007) show that gossip carries reputation in indirect reciprocity (`10.1073_pnas.0704598104`, converted; passage checked). This is the mechanism behind the trust-scaled belief on hearing.
- **Rumour definition.** Zubiaga et al. (2017) define rumours as "items of information that are unverified at the time of posting" (`10.48550_arxiv.1704.00656`, converted; passage checked). The truth flag in the model is the ground-truth version of this idea.
- **Numeric defaults.** `0.3`, `0.2` and the trust coefficient `0.5` are design defaults (no source paper gives these numbers). Park's recency rate 0.995 and Daley–Kendall's structure are the sourced parts.

## Open contradiction check: Daley–Kendall

`src/model/rumor_dk.rs:17–22` stifles on contact whenever the listener already knows the rumour. The classic Daley–Kendall model (Daley & Kendall 1964, DOI `10.1038/2041118a0`, not converted) has this rule, and Moreno et al. (2004, converted) state it as the original model. **Not a contradiction.** The crate implements the α = 1 case. The generalisation to stifling with probability α is **not modelled**, and is listed as a departure.

## What the model leaves out

- Probabilistic stifling (Moreno's α); stifling is deterministic.
- Rumour decay in time beyond the recency weight on telling.
- Verification or counter-rumours: a rumour's truth flag is fixed, and no person corrects it.
- Multiple deeds per rumour; each rumour is about one deed.
- Rumour propagation across groups or factions beyond normal contact.
- Empirical calibration of trust or witness gains.

## Open contradictions and departures

- **Departure, documented:** recency from creation, not retrieval (Park 2023).
- **Departure, documented:** stifling is the α = 1 case of Moreno et al.'s generalisation; α is not modelled.
- **Design defaults:** `rumor_threshold`, trust coefficient, gossip gains and `witness_affinity`. Not fitted.
- **`negativity_ratio` is a cited magnitude on a design-default weight.** Baumeister et al. 2001 supplies the
  five-to-one ratio and the direction of the asymmetry; the 0.15 it multiplies is not fitted by anything.
- **The pack's `snub` grudge curve is a design default measured against the 5:1 ratio.** `Linear(1.0, 0.0)` gave
  2.9 accepted compliments per accepted insult (`village-evening`, 17 game-days, seeds 7/42/11, then seed 23 with
  16 people: 1116:390, 1093:369, 1092:415, 760:276) and rivals outnumbered friends (35/28/36/23 vs 21/22/20/11).
  `Power(1.2)` gives 1489:73, 1867:0, 1543:41, 1090:3 with rivals 7/0/5/0 against friends 80/106/86/63. The sweep
  (exponent 1.4: 30:1, no insults, 49:1, 18:1; 2.0: nearly none; 3.0: none) shows the response is steep and not
  monotone per seed, so 1.2 is the least restrictive value that clears 5:1 everywhere. Seed 42 now has no
  rivalry at all in 17 days.
- **Wills et al. 1974 and Gottman 1994 are cited through Baumeister et al.**, not converted into the corpus.
  Recorded as such rather than presented as primary sources.
- **No contradiction found** with converted sources.

## Sources

- Takács, K., Flache, A. & Mäs, M. (2016). Discrepancy and disliking do not induce negative opinion shifts. https://doi.org/10.1371/journal.pone.0157948 · `10.1371_journal.pone.0157948` — the negative result this crate's reputation model respects: people do not shift opinions away from those they dislike, so no "hostility damps itself" rule was added. Damping is located in who a person chooses to interact with, which is the decision layer.
- Bondü, R. & Richter, P. (2016). Interrelations of justice, rejection, provocation, and moral disgust sensitivity and their links with the hostile attribution bias, trait anger, and aggression. https://doi.org/10.3389/fpsyg.2016.00795 · `10.3389_fpsyg.2016.00795` — the mechanism behind snubbing: rejection and provocation sensitivity predict reactive aggression, i.e. a reaction to how one has been treated. Cited for the mechanism only; the study reports correlations, so the starter pack's grudge curve slope is a design default.
- Baumeister, R. F., Bratslavsky, E., Finkenauer, C. & Vohs, K. D. (2001). Bad Is Stronger Than Good. *Review of
  General Psychology* 5(4), 323–370. https://doi.org/10.1037/1089-2680.5.4.323 · `10.1037_1089-2680.5.4.323` —
  pp. 22 and 33 for the two findings this note uses (negative behaviours reciprocated more strongly; bad events
  about five times as powerful). Already cited by the pack's `snub`; `Tuning::negativity_ratio` is the magnitude.
- Park, J. S. et al. (2023). Generative Agents. https://doi.org/10.48550/arXiv.2304.03442 · `10.48550_arxiv.2304.03442`
- Moreno, Y., Nekovee, M. & Pacheco, A. F. (2004). Dynamics of rumor spreading in complex networks. https://doi.org/10.1103/PhysRevE.69.066130 · `10.1103_physreve.69.066130`
- Sommerfeld, R. D., Krambeck, H.-J., Semmann, D. & Milinski, M. (2007). Gossip as an alternative for direct observation in games of indirect reciprocity. https://doi.org/10.1073/pnas.0704598104 · `10.1073_pnas.0704598104`
- Zubiaga, A. et al. (2017). Detection and resolution of rumours in social media: a survey. https://doi.org/10.48550/arXiv.1704.00656 · `10.48550_arxiv.1704.00656`
- Daley, D. J. & Kendall, D. G. (1964). Epidemics and rumours. https://doi.org/10.1038/2041118a0 · not converted

# Rumour and reputation

Corpus status: distill_search did not return the canonical papers in a usable form, and catalog_read reports no converted entry for the Daley–Kendall (10.1038_2041118a0), Ohtsuki–Iwasa (10.1016_j.jtbi.2004.06.005) or Nowak–Sigmund (10.1038_31225) stems. The sources here are cited from paper_get (OpenAlex/Crossref) metadata and abstracts, not from converted full text. Daley–Kendall has no abstract in the metadata.

## What bevy_society does

A deed that is harmful, or that most witnesses judge strongly, is witnessed by the people near the actor. Each witness changes its trust in the actor and its affinity and respect toward the actor, and appraises admiration or reproach. The patient feels gratitude or anger. A deed large enough to matter also becomes a rumour, which spreads only through accepted interactions flagged as gossip. When a teller passes a rumour to a listener, the listener's belief is scaled by how much it trusts the teller. The teller picks the most vivid rumour it still believes, weighted by how recently it heard it. If the listener already knows the rumour, the teller stops spreading it. The listener's view of the subject moves with the rumour's valence. Reputation can be computed in one of two ways: image scoring, where every refusal to help counts against the refuser, or standing, where refusing a person the witness distrusts does not count against the refuser.

## Equations as implemented

All constants are the defaults of `Tuning` in `src/core.rs` unless noted.

1. Deed harm h in [-1, 1], from `DeedDef`. A rumour is created if |h| >= rumor_threshold = 0.3, or if any witness has |a| >= 0.5, where a is the witness's opinion on the deed's topic.
2. Witness gain: k = witness_gain * (0.5 + C_W) = 0.2 * (0.5 + C_W), where C_W is the witness's conscientiousness in [0, 1].
3. Image scoring: trust(W->A) += k * h for each witness W of actor A.
4. Standing: the same as image scoring, except that when h < 0 and trust(W->P) < 0.3 (the patient is one the witness distrusts), the trust update is skipped.
5. Witness affinity: affinity(W->A) += 0.15 * a. Witness respect: respect(W->A) += 0.1 * a. The witness appraises Admiration when a > 0 and Reproach when a < 0, with intensity |a|. If a < -0.5, the witness writes NormViolated.
6. Patient P: trust(P->A) += k * h, and P appraises Gratitude (h > 0, intensity h) or Anger (h < 0, intensity |h|) toward A.
7. Telling weight: |valence| * belief * recency(hours), with recency(h) = 0.995^h. The teller picks the rumour with the largest weight. Ties go to the lowest RumorId.
8. Stifling: if the listener already knows the rumour, the teller sets spreader = false for it. Otherwise the listener learns it.
9. Belief on hearing: belief_listener = belief_teller * (0.5 + 0.5 * clamp(trust(listener->teller), 0, 1)). This is the crate's modelling choice, marked as such in `src/model/rumor_dk.rs`.
10. Subject update on hearing: trust(listener->subject) += gossip_trust_gain * valence * belief, with gossip_trust_gain = 0.3, and affinity(listener->subject) += gossip_affinity_gain * valence * belief, with gossip_affinity_gain = 0.2. The tie is created if it is absent.
11. Reach: known_by / living persons.

Belief itself is stored and not decayed. Recency enters only the telling weight, so an old rumour is still believed at full strength and simply loses priority to newer ones.

Discrepancy (citation, not numeric): the tuning comments cite Sommerfeld et al. 2007 for rumor_threshold = 0.3, and Dunbar 2004 for gossip_trust_gain and gossip_affinity_gain. The Sommerfeld abstract concerns gossip composition, information transfer and behaviour in an indirect-reciprocity game with 14 groups of nine students. It gives no threshold. The Dunbar abstract concerns gossip's bonding and free-rider functions. It gives no gain values. The papers support the mechanisms, not the numbers. The numbers are the crate's choices.

Unconfirmed (must-confirm): the Daley–Kendall stifling rule. The code stifles the teller whenever the listener already knows the rumour. Daley and Kendall (1964) stifle a spreader on meeting another spreader or a stifler. This is from memory of the 1964 model, not from the paper text, which is not in the metadata. The code's rule is a variant that also stifles on meeting a stifler, and it is possible that the variant is intended. Confirm before changing the code.

Unconfirmed (must-confirm): the Ohtsuki–Iwasa standing rule. The abstract ("How should we define goodness?—reputation dynamics in indirect reciprocity") frames the question of which actions count as good, but states no rule. The code's exception, for refusing a distrusted bad recipient with trust(W->P) < 0.3, is the crate's reading.

Unconfirmed (must-confirm): the recency constant 0.995 per hour from Park et al. 2023. That paper was not read in this pass. The constant is cited from the plan, and the half-life it implies is about 138 hours.

## Why these defaults

Sommerfeld et al. 2007: "Here we designed a game to test a widespread assumption that gossip functions as a vector for the transmission of social information." This supports modelling gossip as a transmission step between listener and teller, rather than as a global broadcast.

Dunbar 2004: "a key function may be related explicitly to controlling free riders." This supports gossip as a channel for reputation, which is what the subject update in step 10 models.

Nowak and Sigmund 1998: "Cooperation pays because it confers the image of a valuable community member." This supports the witness-based trust update in steps 3 and 4. The abstract does not give the scoring rule, so the harm-weighted, witness-weighted increment k * h is a generalisation, not a copy of the paper's scoring.

Ohtsuki and Iwasa 2004: "Little has been known, however, about the definition of 'goodness' in reputation." This is the reason the code offers two switchable reputation rules rather than one. The standing exception in step 4 is the crate's answer to that open question.

Daley and Kendall 1964, "Epidemics and rumours": the title frames rumour spread as an epidemic with stifling. This supports the spreader and stifler states. The exact stifling rule is the must-confirm item above.

Candidate-pool title only, not read: 10.1098/rspb.2001.1809, "Cooperation through indirect reciprocity: image scoring or standing strategy?" This directly compares the two rules that step 4 switches between.

## What the model leaves out

- Rumour mutation. Rumours are copied verbatim, with the same valence and subject. Sommerfeld et al. report changes in composition, which the model does not reproduce.
- Multiple variants of one deed. Each deed produces at most one rumour.
- Re-spreading by stiflers. A stifler never resumes spreading, so forgetting cannot revive a rumour.
- Hop-based credibility. A listener does not discount a rumour by the number of hops it has travelled, beyond the belief product in step 9.
- Context-specific reputation. Reputation is one trust and one affinity per pair, with no split by activity or deed type.
- Attention limits on witnessing. Every person in perception range witnesses the deed.
- False rumours. A rumour created with truth = false is never checked against anything, and no witness corrects it.
- Costs of punishment. Reproach and refusal cost the witness nothing.
- Other goodness rules. Only the one standing exception from Ohtsuki and Iwasa is modelled.
- Multi-party gossip. Gossip passes from one teller to one listener per interaction.

## Sources

- Daley and Kendall 1964, "Epidemics and rumours": https://doi.org/10.1038/2041118a0 (metadata read via paper_get; no abstract; catalog has no converted entry).
- Sommerfeld, Krambeck, Semmann and Milinski 2007, "Gossip as an alternative for direct observation in games of indirect reciprocity": https://doi.org/10.1073/pnas.0704598104 (abstract read via paper_get; catalog state not checked in this pass).
- Dunbar 2004, "Gossip in Evolutionary Perspective": https://doi.org/10.1037/1089-2680.8.2.100 (abstract read via paper_get; catalog state not checked in this pass).
- Nowak and Sigmund 1998, "Evolution of indirect reciprocity by image scoring": https://doi.org/10.1038/31225 (abstract read via paper_get; the abstract is truncated; catalog has no converted entry).
- Ohtsuki and Iwasa 2004, "How should we define goodness?—reputation dynamics in indirect reciprocity": https://doi.org/10.1016/j.jtbi.2004.06.005 (abstract read via paper_get; the abstract is truncated; catalog has no converted entry).
- Park et al. 2023, generative agents: https://doi.org/10.48550/arXiv.2304.03442 (cited from the plan; not read in this pass).
- Candidate-pool title only, not read: 10.1098/rspb.2001.1809, "Cooperation through indirect reciprocity: image scoring or standing strategy?"
- Home-still doc_ids: none converted at time of writing for the papers above.

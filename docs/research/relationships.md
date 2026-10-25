# Relationships

## What bevy_society does

Each person keeps a `Ties` component: one `Tie` per person they know. A tie holds an affinity in `[-1, 1]`, trust in `[0, 1]` (default 0.5), respect in `[-1, 1]`, a familiarity peak with the day of last contact, and a label (Friend, Acquaintance, Stranger, Rival). Familiarity decays on a power law from the last contact. Contact (exposure) raises familiarity and nudges affinity toward the other person. Affinity relaxes slowly toward zero when no one reinforces it. When two people share a friend, a triadic balance step nudges the pair's affinity. A label changes only when a threshold is crossed, and the change is emitted as a message. The model functions are on HEAD (`src/relationships.rs`, `src/model/recall.rs`, `src/model/balance.rs`).

## Equations as implemented

- Familiarity (`src/model/recall.rs:18–20`, used by `Tie::familiarity`, `src/relationships.rs:99–106`): `f = peak · (1 + elapsed/τ)^(−d)`, with `τ = Tuning.familiarity_tau_days` = 1.0, `d = Tuning.familiarity_decay_exp` = 0.5, and `elapsed` in game days since `last_contact_day`.
- Exposure (`src/relationships.rs:110–117`):
  - `familiarity_peak ← clamp(f + rate · dt · (1 − f), 0, 1)`, with `rate = Tuning.exposure_per_day` = 2.0;
  - `affinity ← clamp(affinity + mere_exposure_per_day · dt · (1 − |affinity|), −1, 1)`, with `mere_exposure_per_day` = 0.05 — **a design default**; Zajonc (DOI `10.1037/h0025848`) supplies the mechanism, not the rate. Against `affinity_relax_per_day` = 0.01 that settles affinity at **0.833**, which is 52% above the `0.4` Friend threshold, so proximity alone is sufficient for friendship. Measured, it is **not** what sets a village's friend count (sweeping it to 0.005 moves friends only 22 → 16): affinity from *interaction* has no saturating term, so a pair becomes a Friend after **45 exchanges** and friendship scales with contact volume. See `docs/loop/needs-decision/friendship-has-no-diminishing-return.md`.
- Relaxation (`src/relationships.rs:120–123`): affinity moves toward zero by `affinity_relax_per_day · dt`, with `affinity_relax_per_day` = 0.01 (`src/model/balance.rs`, `relax_toward_zero`).
- Triadic balance (`src/model/balance.rs:7–9`): `Δaff(A→B) = balance_rate_per_day · dt · aff(A→B) · aff(B→C)`, with `balance_rate_per_day` = 0.5 and `balance_samples_per_tick` = 8 (`src/core.rs`).
- Introduction (`introduction_familiarity` = 0.1 starting peak, design default): a new tie starts at this familiarity.
- Labels (`src/relationships.rs:40–50`, `TieLabel::classify`):
  - Friend if affinity ≥ 0.4 and familiarity ≥ 0.3;
  - Rival if affinity ≤ −0.4 and familiarity ≥ 0.2;
  - Stranger if familiarity < 0.1;
  - otherwise Acquaintance.
  `refresh_label` (`:127–140`) returns the old and new label only when the label changes.

- **The label thresholds are `Tuning` fields.** `friend_affinity` 0.4, `friend_familiarity` 0.3, `rival_affinity` 0.4, `rival_familiarity` 0.2, `stranger_familiarity` 0.1, and `rejection_affinity_penalty` 0.05 — all **design defaults**, none from a paper. They were literals inside `classify` until 2026-12-12. Measured, `friend_familiarity` is satisfied by 0.907 among friend-ties and 0.908 among the rest, so that term discriminates nothing; that is now visible and adjustable in one place.
- **Trust can rise and fall.** `chat` (+0.05 to each party), `gossip` (+0.03) and `compliment` (+0.02) raise it on accept; `witness_reaction` moves it by `trust_gain * harm`, so a witnessed `steal` (`harm -0.7`) is **-0.14 per witness**. The starter pack set **no** trust delta at all until 2026-12-03, so trust could only fall: measured, targets held **0.13-0.27** toward the person approaching them against the 0.5 that `AcceptDef` treats as neutral, and 47% of interactions were refused.
- **The asymmetry that remains is a decision.** A gain reaches two parties; a loss reaches every witness at full magnitude. One chat is `+0.10`; one insult seen by three people is `-0.42`. See `docs/loop/needs-decision/a-deed-costs-what-the-crowd-saw.md`.

## Why these defaults

- **Familiarity decay.** Murre & Dros (2015) fit a power function to forgetting (`10.1371_journal.pone.0120644`, converted; power-function passage checked). Anderson & Schooler (1991) show the same shape for memory (DOI `10.1111/j.1467-9280.1991.tb00174.x`, not converted; cited from DOI only).
- **Support cliques and group size.** Zhou, Sornette, Hill & Dunbar (2005) give a support clique of about 3.3 people and an equivalent group size of about 150 (`10.1098_rspb.2004.2970`, converted; passage checked). Those numbers set the scale of `close_ties` = 15 in `SocietySettings`; the figure 15 itself is a design default (no source paper gives this number).
- **Network dynamics.** Sekara, Stopczynski & Lehmann (2016) show how close-tie networks change over time (`10.1073_pnas.1602803113`, converted; passage checked). This supports letting ties change rather than fixing them.
- **Homophily.** Gillespie et al. (2015) report that "friendship patterns demonstrated gender-based homophily" (`10.1371_journal.pone.0128900`, converted; passage checked). The affinity similarity step in `src/model/opinion.rs` is a design default in the same spirit; the study does not give its size.
- **Mere exposure.** Zajonc (1968, DOI `10.1037/h0025848`, not converted) is the mechanism for the exposure step. Zandstra (2000, `10.18174_197257`, converted) quotes it, which is secondary support only.
- **Balance.** Heider (1946) and Cartwright & Harary (1956) describe triadic balance (DOIs `10.1080/00223980.1946.9917275` and `10.1037/h0046049`, not converted). The rate `0.5` is a design default.
- **Label thresholds.** `0.4`, `0.3`, `0.2` and `0.1` are design defaults (no source paper gives these numbers). The code's doc comment says the thresholds follow the mechanisms of Zajonc (1968) and Byrne (1961, DOI `10.1037/h0044721`, not converted).

## What the model leaves out

- Reciprocity and mutual perception: each person's tie is held separately, so asymmetric views are allowed but not explained.
- Strength of ties from shared activities or time spent together, beyond exposure.
- Dissolution of ties through conflict, except through the rival label.
- Group-based tie formation beyond the faction bias applied in D7.
- Kin, role or structural ties (family, employer); only interaction history counts.

## Open contradictions and departures

- **Design defaults:** the label thresholds, exposure rates and balance rate are design defaults, not fitted to a source.
- **No contradiction** found with converted sources. Gillespie and Zhou support the scale and homophily mechanisms; neither fixes the numeric thresholds.

## Sources

- Zhou, W.-X., Sornette, D., Hill, R. A. & Dunbar, R. I. M. (2005). Discrete hierarchical organization of social group sizes. https://doi.org/10.1098/rspb.2004.2970 · `10.1098_rspb.2004.2970`
- Sekara, V., Stopczynski, A. & Lehmann, S. (2016). Fundamental structures of dynamic social networks. https://doi.org/10.1073/pnas.1602803113 · `10.1073_pnas.1602803113`
- Gillespie, C. et al. (2015). https://doi.org/10.1371/journal.pone.0128900 · `10.1371_journal.pone.0128900`
- Murre, J. M. J. & Dros, J. (2015). Replication and analysis of Ebbinghaus' forgetting curve. https://doi.org/10.1371/journal.pone.0120644 · `10.1371_journal.pone.0120644`
- Zandstra, E. (2000). Mere exposure, thesis. · `10.18174_197257` (secondary; quotes Zajonc)
- Anderson, J. R. & Schooler, L. J. (1991). https://doi.org/10.1111/j.1467-9280.1991.tb00174.x · not converted
- Zajonc, R. B. (1968). https://doi.org/10.1037/h0025848 · not converted
- Heider, F. (1946). https://doi.org/10.1080/00223980.1946.9917275 · not converted
- Cartwright, D. & Harary, F. (1956). https://doi.org/10.1037/h0046049 · not converted
- Byrne, D. (1961). https://doi.org/10.1037/h0044721 · not converted

## Bereavement (issue #6)

- **Prigerson, H. G. et al. (2009).** Prolonged Grief Disorder: psychometric validation of criteria proposed for DSM-V and ICD-11. *PLoS Medicine* 6(8): e1000121. https://doi.org/10.1371/journal.pmed.1000121 · `10.1371_journal.pmed.1000121` (in the corpus).

  The model's mechanism, not its rate. The paper's own criteria are keyed to the loss of a **close relationship** rather than to any death — bereavement is a fact for everyone who knew the person and a wound in proportion to how close they were. `Tuning::bereavement_familiarity` (0.3, a design default) is the crate's line between an acquaintance and someone you knew, and it exists so a **family member** grieves: they are spawned with high familiarity and no time to reach the affinity that a `Friend` label needs.

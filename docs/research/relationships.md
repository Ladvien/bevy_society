# Relationships

## What bevy_society does

Each person keeps a `Ties` component: one `Tie` per person they know. A tie holds an affinity in `[-1, 1]`, trust in `[0, 1]` (default 0.5), respect in `[-1, 1]`, a familiarity peak with the day of last contact, and a label (Friend, Acquaintance, Stranger, Rival). Familiarity decays on a power law from the last contact. Contact (exposure) raises familiarity and nudges affinity toward the other person. Affinity relaxes slowly toward zero when no one reinforces it. When two people share a friend, a triadic balance step nudges the pair's affinity. A label changes only when a threshold is crossed, and the change is emitted as a message. The model functions are on HEAD (`src/relationships.rs`, `src/model/recall.rs`, `src/model/balance.rs`).

## Equations as implemented

- Familiarity (`src/model/recall.rs:18–20`, used by `Tie::familiarity`, `src/relationships.rs:99–106`): `f = peak · (1 + elapsed/τ)^(−d)`, with `τ = Tuning.familiarity_tau_days` = 1.0, `d = Tuning.familiarity_decay_exp` = 0.5, and `elapsed` in game days since `last_contact_day`.
- Exposure (`src/relationships.rs:110–117`):
  - `familiarity_peak ← clamp(f + rate · dt · (1 − f), 0, 1)`, with `rate = Tuning.exposure_per_day` = 2.0;
  - `affinity ← clamp(affinity + mere_exposure_per_day · dt · (1 − |affinity|), −1, 1)`, with `mere_exposure_per_day` = 0.05.
- Relaxation (`src/relationships.rs:120–123`): affinity moves toward zero by `affinity_relax_per_day · dt`, with `affinity_relax_per_day` = 0.01 (`src/model/balance.rs`, `relax_toward_zero`).
- Triadic balance (`src/model/balance.rs:7–9`): `Δaff(A→B) = balance_rate_per_day · dt · aff(A→B) · aff(B→C)`, with `balance_rate_per_day` = 0.5 and `balance_samples_per_tick` = 8 (`src/core.rs`).
- Introduction (`introduction_familiarity` = 0.1 starting peak, design default): a new tie starts at this familiarity.
- Labels (`src/relationships.rs:40–50`, `TieLabel::classify`):
  - Friend if affinity ≥ 0.4 and familiarity ≥ 0.3;
  - Rival if affinity ≤ −0.4 and familiarity ≥ 0.2;
  - Stranger if familiarity < 0.1;
  - otherwise Acquaintance.
  `refresh_label` (`:127–140`) returns the old and new label only when the label changes.

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

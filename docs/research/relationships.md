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
- Labels (`TieLabel::classify_with_trust`), the **promotion** bars:
  - Friend if affinity ≥ `friend_affinity` and familiarity ≥ `friend_familiarity` and trust ≥ `friend_trust`;
  - Rival if affinity ≤ −`rival_affinity` and familiarity ≥ `rival_familiarity`;
  - Stranger if familiarity < `stranger_familiarity`;
  - otherwise Acquaintance.
  `refresh_label` returns the old and new label only when the label changes.
- **Leaving a category is not the same as entering it** (`TieLabel::classify_from`). An `Acquaintance`
  falls back to a `Stranger` only at `stranger_familiarity − label_hysteresis`, so familiarity decaying
  through one line no longer flips the label. See "A label is a latitude" below.

- **The label thresholds are `Tuning` fields.** `friend_affinity` 0.4, `friend_familiarity` 0.3, `rival_affinity` 0.4, `rival_familiarity` 0.2, `stranger_familiarity` 0.1, and `rejection_affinity_penalty` 0.05 — all **design defaults**, none from a paper. They were literals inside `classify` until 2026-12-12. Measured, `friend_familiarity` is satisfied by 0.907 among friend-ties and 0.908 among the rest, so that term discriminates nothing; that is now visible and adjustable in one place.
- **Trust can rise and fall.** `chat` (+0.05 to each party), `gossip` (+0.03) and `compliment` (+0.02) raise it on accept; `witness_reaction` moves it by `trust_gain * harm`, so a witnessed `steal` (`harm -0.7`) is **-0.14 per witness**. The starter pack set **no** trust delta at all until 2026-12-03, so trust could only fall: measured, targets held **0.13-0.27** toward the person approaching them against the 0.5 that `AcceptDef` treats as neutral, and 47% of interactions were refused.
- **The asymmetry that remains is a decision.** A gain reaches two parties; a loss reaches every witness at full magnitude. One chat is `+0.10`; one insult seen by three people is `-0.42`. See `docs/loop/needs-decision/a-deed-costs-what-the-crowd-saw.md`.

## A label is a latitude, not a line (issue #10)

- **Sherif, M. & Hovland, C. I. (1961).** *Social judgment: Assimilation and contrast effects in
  communication and attitude change.* Read via **Fagan, P. J. & Browne, A. (1994: 219–220)**, in
  *Responses of the Battered*, corpus doc `10.1007_0-306-47586-3_8` (in the corpus), which quotes the
  latitude model:

  > "The 'latitude of acceptance' is the range of possibilities with which an individual is willing to
  > agree or to which an individual can adapt. ... According to social judgment theory, if stimuli
  > continue to fall at the end of the continuum or even slightly above the end point, this will produce
  > a shift of the range toward that anchor—or assimilation."

  A categorical judgement about a person is a **band**, not a point. Entering a category and leaving it
  are therefore not the same test, which is hysteresis. **The source supplies the shape and no number**;
  `Tuning::label_hysteresis` is a design default chosen by measurement.

- **Igarashi, T.** *Longitudinal Changes in Face-to-Face and Text Message–Mediated Friendship Networks.*
  Corpus doc `10.1017_cbo9780511894701.022` (in the corpus).

  > "A 'superficial network' is composed of low intimate and weak ties that are **stable but
  > changeable**."

  A weak tie should be the most stable thing in the graph. Measured before this change it was the least:
  `Stranger <-> Acquaintance` was 924 of 2 867 `TieLabelChanged` messages over 16.4 game-days, with a
  median margin from the bar of **0.0027**.

- **Where the band goes, and where it must not.** `village-evening`, seed 7, 16.4 game-days: **2 867**
  label messages, one per 8.2 game-minutes. Of the four transitions measured, the low end is noise (the
  affinity or familiarity in the flipping tick moved by ~0.0002) and the friend and rival ends are events
  (a median one-tick step of 0.013 — one accepted exchange or one witnessed deed). A band on the friend
  and rival bars as well cut messages 19% but moved rivals 42 → **48** and friends 18 → **20**, because
  it held a friendship after the event that ended it. The shipped band is on `stranger_familiarity`
  only, at 0.02: **friends and rivals identical on all three seeds**, `Stranger -> Acquaintance` 828 →
  **552** (= 276 ordered pairs, once each) and back 96 → **1**.

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

## The clamp and the influence weight (issue #10)

- **Deffuant, G., Neau, D., Amblard, F. & Weisbuch, G. (2000).** Mixing beliefs among interacting agents. *Advances in Complex Systems* 3: 87–98. DOI `10.1142/S0219525900000078`.

  The influence term. `Tuning::affinity_weight` scales affinity by `1/affinity_ceiling` so that this model keeps the `[-1, 1]` scale it was defined on: the clamp bounds how much a person can *like* anyone, which is what keeps friendship scarce, but the opinion model reads affinity as *how much weight this person's view carries*, and the raw clamped value would put every weight at a third of what the model expects.
- **Byrne, D. (1961).** Interpersonal attraction and attitude similarity. DOI `10.1037/h0044721`. The affinity-from-similarity mechanism the clamp modifies; the ceiling itself is a **design default**, chosen by the measurement in `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` rather than supplied by a paper.
- **Trust as the discriminator.** Measured on the village: friend-ties average affinity 0.792 against -0.223 for the rest, but familiarity is **0.907 against 0.908** — no separation at all — while trust separates **sevenfold** (0.480 against 0.068). That is why `Tuning::friend_trust` is the second thing a friend is. Design default; the paper supplies the mechanism (Zajonc 1968, above) and none of the numbers.

## Bad is stronger than good in total — and good should win by numbers (issue #10)

- **Baumeister, R. F., Bratslavsky, E., Finkenauer, C. & Vohs, K. D. (2001).** Bad is stronger than good.
  *Review of General Psychology* 5(4): 323–370. DOI `10.1037/1089-2680.5.4.323` · `10.1037_1089-2680.5.4.323`
  (in the corpus).

  The crate cites this already, in `snub`'s consideration, for the grudge mechanism. The review makes **two**
  claims and the crate currently honours one:

  > "bad impressions and bad stereotypes are quicker to form and **more resistant to disconfirmation** than good
  > ones"

  > "**good may prevail over bad by superior force of numbers**: Many good events can overcome the psychological
  > effects of a single bad one."

  The second is conditional, and it is the one the starter pack does not satisfy. Measured on `village-evening`,
  seed 7, 24 people over 17 game-days, summing every affinity write (`examples/evidence.rs` counts the accepted
  compliments; the insult side is the sum of `0.15 * approval` where `approval < 0.0` in `src/reputation.rs`):
  **1 604** accepted `compliment`s moved affinity by **+160.4** in total (**+0.10** each), while an `insult`
  (deed harm `-0.4`) reached witnesses **7 920** times and moved them by **-241.2** in total — a mean of
  **-0.0305** affinity per witness. Bad is ahead **1.5:1** in total weight, and good is outnumbered **4.9:1** by
  events: one compliment weighs about 3.3x a witness's reaction, but there are 4.9x as many reactions. Good loses
  on numbers, not on weight, so friendship cannot accumulate, and the village settles at **18 friend pairs
  against 42 rival pairs**.

  An earlier version of this note said an insult was `witness_gain 0.2 x -0.4 = -0.08` per witness, **-0.48** in
  total, "5:1 per event". That applied `witness_gain` to affinity. `witness_gain` moves **trust**; affinity moves by
  `0.15 x approval`, where `approval` is the witness's opinion of kindness signed by the deed's harm.

  **The asymmetry is a design default the crate does not yet have.** It is recorded rather than shipped, because
  the obvious implementation — making negative affinity decay more slowly — was measured and it **doubles**
  rivalries (42 -> 82): persisting grudges persist while friendship keeps fading at the same rate, so the
  counterweight is unchanged. What the paper actually prescribes is more friendly *events*, not stickier ones.
  See `docs/loop/needs-decision/good-is-outweighed-by-bad.md`.

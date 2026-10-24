# Personality and mood

## What bevy_society does

Each person has five personality traits (`Personality`, the OCEAN dimensions in `[0, 1]`) and a mood, a point in pleasure–arousal–dominance (PAD) space. The trait vector sets the mood's baseline. Each tick, the mood is pulled toward the centre of the space, relaxes back toward its baseline, and drifts toward the mood of nearby people (contagion). Emotions are discrete events (joy, distress, pride, shame and so on) that appraisal produces from game events and that decay with a half-life. The emotion's kind and intensity feed the mood and the bark feeling. The model functions are on HEAD (`src/model/pad.rs`, `src/mind.rs` `Mood`, `Emotions`).

## Equations as implemented

- Baseline from traits (`src/model/pad.rs:10–13`): with `t′ = 2t − 1` for each trait, `S = −N′`, `P = 0.21 E′ + 0.59 A′ + 0.19 S`, `A = 0.15 O′ + 0.30 A′ − 0.57 S`, `D = 0.25 O′ + 0.17 C′ + 0.60 E′ − 0.32 A′`, each clamped. **The coefficients are unverified** (see below).
- Mood step (`src/mind.rs:183–207`), with `dt` in game days:
  - pull toward the centre: `I = min(pull_per_day · dt, 1)`, applied to each axis by `I`, with `pull_per_day = Tuning.mood_pull_per_day` = 4.0;
  - return to baseline **plus the positivity offset**: `min(return_per_day · dt, 1)` toward `baseline.pleasure + Tuning.positivity_offset`, with `return_per_day = Tuning.mood_return_per_day` = 1.0 and `positivity_offset` = 0.1. Arousal and dominance have no offset and return to the bare baseline;
  - contagion, when contacts are present: `pleasure += contagion_per_day · weight · dt · (contacts − pleasure)`, with `contagion_per_day = Tuning.contagion_per_day` = 0.3;
  - then `Mood::from_pad` (`src/mind.rs:163`).
- Emotion decay: `emotion_half_life_days = Tuning.emotion_half_life_days` = 0.1 (about 2.4 game hours).
- Emotion kinds (`src/mind.rs:212–238`): Joy, Distress, Hope, Fear, Pride, Shame, Admiration, Reproach, Gratitude, Anger, Liking, Disliking. Appraisal follows OCC as used by Gratch & Marsella (2004) and Dias & Paiva (2005).

## Why these defaults

- **Mood as PAD.** Kovács (2025) uses the ALMA model, which takes its valence and arousal ("pleasure-displeasure") from Gebhard (2005) (`eth-cgl-digital_chars-Kov25a`, converted; passage checked). Silverman et al. (2006) describe a unified architecture that spans "personality/culture/emotion" (`10.1162_pres.2006.15.2.163`, converted; passage checked), the same layers the mood model combines.
- **The positivity offset and the negativity bias.** Cacioppo, Cacioppo & Gollan 2013 (DOI `10.1017/S0140525X13002537`, converted; passage checked) separate two properties of the affect system: *"the intercept for the positive activation function (i.e., the approach motivation at zero input) is higher than the intercept for the negative activation function (producing the positivity offset), and the gain for the negative activation function is higher than for the positivity activation function (producing the negativity bias)."* The crate had the gain — `snub`'s insult pays the target Anger 0.5 against `compliment`'s Joy 0.3 — and no intercept, so pleasure's resting point was the personality baseline alone, which is zero for a sampled personality: measured on `village-evening`, pleasure's maximum over every person and tick was exactly `+0.000`. Measured, an accepted `chat` or `gossip` also pays mutual `Liking` (0.2 and 0.15): before that, 43% of accepted interactions paid no emotion at all, and 1 216 refusals produced exactly 1 216 distress appraisals against 443 positives from 1 338 acceptances. The **offset's magnitude is a design default**; the paper gives its existence and sign, not a number. Measured at three values (max pleasure **+0.000 → +0.0155 → +0.0476** for 0.0, 0.1, 0.3), the offset moves the *ceiling*; the mean needs the appraisal balance, which the ratio of `on_accept` and `on_reject` emotions controls.
- **Emotional contagion.** Van Kleef & Côté (2022) review how emotions pass between people in social settings (`10.1146_annurev-psych-020821-010855`, converted; mood and contagion passages checked). Hatfield (1993) is the classic source for contagion (DOI `10.1111/1467-8721.ep10770953`, not converted; cited from DOI only). The rate `0.3` is a design default (no source paper gives this number).
- **Pull and return rates.** The pull rate `4.0` and return rate `1.0` are design defaults. The code comment attributes the pull to Mehrabian (1996) and the return to Gebhard (2005); see the conflict below.
- **Emotion half-life 0.1 day.** Design default (no source paper gives this number).
- **PAD baseline coefficients.** Mehrabian (1996) gives the trait-to-PAD regressions the code uses. **Unverified:** Mehrabian 1996 is not converted in the corpus, so the coefficients in `pad.rs` are not checked against the paper. Gebhard (2005) reproduces them but its text is not converted either.

## Conflict: attribution of the mood rates

`src/core.rs` `Tuning` (l.82–86) attributes `mood_pull_per_day` to Mehrabian 1996 and `mood_return_per_day` to Gebhard 2005. `src/mind.rs` `Mood::step` doc (l.181–182) attributes the pull to Gebhard and the return to Mehrabian. The two doc comments disagree. This note takes neither side. The rate values are design defaults. The fix belongs in the source doc comments, not here, and is reported to the integrator.

## What the model leaves out

- Mood-dependent appraisal: emotions are appraised from events, not from the current mood.
- Individual differences in contagion beyond the contact count and weight.
- Physiology: no stress, hormones or circadian effects.
- Cultural display rules; van Kleef & Côté discuss them, the crate does not model them.
- Long-term mood disorders or trait change over time; traits are fixed per person.

## Open contradictions and departures

- **Unverified:** Mehrabian (1996) PAD coefficients (`src/model/pad.rs`). Not converted; the two DOIs also disagree in the crate (`10.1080/00049539608259510` in `core.rs`, `mind.rs:182`, `pad.rs`; `10.1007/BF02686918` at `mind.rs:128`).
- **Conflict (recorded above):** the two src doc comments attribute the mood rates to different sources.
- **No contradiction found** in converted sources for the contagion or PAD structure. The converted sources support the structure only, not the coefficients.

## Sources

- Kovács, G. (2025). Converted document `eth-cgl-digital_chars-Kov25a` (title not checked from the text).
- Silverman, B. G. et al. (2006). Human Behavior Models for Agents in Simulators and Games: Part II. Presence 15(2). https://doi.org/10.1162/pres.2006.15.2.163 · `10.1162_pres.2006.15.2.163`
- van Kleef, G. A. & Côté, S. (2022). The Social Effects of Emotions. Annu. Rev. Psychol. https://doi.org/10.1146/annurev-psych-020821-010855 · `10.1146_annurev-psych-020821-010855`
- Mehrabian, A. (1996). PAD model. https://doi.org/10.1080/00049539608259510 · not converted
- Gebhard, P. (2005). ALMA. https://doi.org/10.1145/1082473.1082478 · not converted; cited in Kovács [24]
- Hatfield, E. et al. (1993). Emotional contagion. https://doi.org/10.1111/1467-8721.ep10770953 · not converted
- Gratch, J. & Marsella, S. (2004). OCC-based appraisal. https://doi.org/10.1016/j.cogsys.2004.02.002 · not converted

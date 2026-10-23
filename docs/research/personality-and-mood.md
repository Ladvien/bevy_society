# Personality and mood

## What bevy_society does

Each person is sampled a five-trait `Personality` (openness, conscientiousness, extraversion, agreeableness, neuroticism) and starts from a baseline mood on three axes, pleasure, arousal and dominance (PAD), computed from those traits. Events add emotions (an OCC-style subset of twelve kinds, each with a fixed PAD vector). Each tick the mood moves toward the intensity-weighted centre of the active emotions, relaxes back toward the personality baseline, and drifts its pleasure toward the pleasure of the people it is in contact with, weighted by affinity and extraversion. Emotions fade with a half-life of 0.1 game-day. The mood feeds activity scoring, bark feelings and the rumour and grudge logic.

## Equations as implemented

**Trait sampling.** Each trait is the mean of three uniform draws, `t = (u₁ + u₂ + u₃)/3`, so the default population is centred on 0.5 (`Personality::sample`, `src/mind.rs`). The trait sign convention is t′ = 2t − 1 ∈ [−1, 1]. Sources: McCrae & John 1992, DOI 10.1111/j.1467-6494.1992.tb00970.x; Goldberg 1990, DOI 10.1037/0022-3514.59.6.1216. Both are metadata-only (see Sources).

**Baseline mood (`src/model/pad.rs::baseline`).** With S = −N′ (emotional stability, the negation of neuroticism):

- P = 0.21·E′ + 0.59·A′ + 0.19·S
- A = 0.15·O′ + 0.30·A′ − 0.57·S
- D = 0.25·O′ + 0.17·C′ + 0.60·E′ − 0.32·A′

Each result is clamped to [−1, 1]. Source: Mehrabian 1996, DOI 10.1080/00049539608259510 and DOI 10.1007/BF02686918 (metadata-only), as reproduced in the ALMA model of Gebhard 2005, DOI 10.1145/1082473.1082478 (metadata-only). **Unverified against the paper.** The coefficients and the N/S sign convention in the code match ALMA's presentation, but the corpus holds no full text of either paper. The Mehrabian 1996 abstract, read through a web search result and not through the corpus, describes five regressions of the Big Five factors on trait PAD scales, which is the reverse direction of the equations above. This must be checked against the paper before the note is treated as settled. No discrepancy is recorded, because the paper is not in the corpus.

**Emotion vectors (`EmotionKind::pad`).** Each emotion has a fixed PAD point (ALMA's table, Gebhard 2005, metadata-only):

| Emotion | P | A | D |
| --- | --- | --- | --- |
| Joy | 0.40 | 0.20 | 0.10 |
| Distress | −0.40 | −0.20 | −0.50 |
| Hope | 0.20 | 0.20 | −0.10 |
| Fear | −0.64 | 0.60 | −0.43 |
| Pride | 0.40 | 0.30 | 0.30 |
| Shame | −0.30 | 0.10 | −0.60 |
| Admiration | 0.50 | 0.30 | −0.20 |
| Reproach | −0.30 | −0.10 | 0.40 |
| Gratitude | 0.40 | 0.20 | −0.30 |
| Anger | −0.51 | 0.59 | 0.25 |
| Liking | 0.40 | 0.16 | −0.24 |
| Disliking | −0.40 | 0.20 | 0.10 |

**Emotion intensity.** Appraisal adds intensity to a matching (kind, target) emotion, capped at 1.0. Each tick intensity decays as I ← I · 0.5^(Δt / t½), with t½ = `emotion_half_life_days` = 0.1 game-day. An emotion is dropped when I < 0.05.

**Mood step (`Mood::step`, `src/mind.rs`).** Let I_k be the intensities, P_k the PAD vectors and I = min(1, ΣI_k). Let V = Σ I_k·P_k / ΣI_k be the intensity-weighted centre, and let B be the baseline. Per tick, with Δt = `dt_days`:

- Pull toward the emotional centre: mood ← mood + min(1, `mood_pull_per_day`·Δt) · I · (V − mood), with `mood_pull_per_day` = 4.0.
- Return to baseline: mood ← mood + min(1, `mood_return_per_day`·Δt) · (B − mood), with `mood_return_per_day` = 1.0.
- Emotional contagion on pleasure only: pleasure ← pleasure + `contagion_per_day` · E · Δt · (P̄_c − pleasure), where E is extraversion in [0, 1], `contagion_per_day` = 0.3, and P̄_c is the mean pleasure of contacts weighted by max(0, affinity to them). The term applies only when at least one weight is positive.
- All components are clamped to [−1, 1].

The decision that E, extraversion, is the contagion factor is a design choice recorded in the plan. The corpus review below supports the affiliation weighting, not the extraversion factor.

**Sources for the mood step.** Hatfield et al. 1993 (contagion), DOI 10.1111/1467-8721.ep10770953, metadata-only. Gebhard 2005 (mood update), metadata-only. The defaults `mood_pull_per_day`, `mood_return_per_day` and `contagion_per_day` are design defaults; no corpus paper verifies them.

## Why these defaults

- **PAD as the mood space.** Valence, arousal and dominance rated as separate dimensions is the lineage that the corpus VAD rating papers continue. `10.18653_v1_p18-1017` (Mohammad 2018, a VAD lexicon) and `10.1145_2901739.2901752` (Adams, Graziotin & Mäntylä 2016, VAD in text mining) are indexed in home-still and document the VAD rating method. They do not verify the Mehrabian coefficients, which remain unverified (see above).
- **Affiliation weights contagion.** `10.1146_annurev-psych-020821-010855` (Annual Review of Psychology 2022, 73:629–658) states: "A complete explanation of contagion effects needs to accommodate the sharing of emotions that is not automatic and nonconscious as well as motivational processes that render individuals more likely to catch the emotional states of those they want to affiliate with and attend to." The review also covers social appraisal and complementary reactions. This supports weighting contagion by affinity (motivation to affiliate). It does not establish extraversion as the factor, so that remains a design choice.
- **Rates are design defaults.** The half-life of 0.1 game-day, the pull of 4.0 per game-day, the return of 1.0 per game-day and the contagion of 0.3 per game-day are design defaults chosen so that moods move within a few game-hours at the default clock. No corpus paper was found that verifies these magnitudes. They should be tuned against the village capture, not against a paper.

## What the model leaves out

- Non-verbal and automatic contagion in a single contact. Contagion here is a pleasure drift, not a per-channel transfer. The review above treats non-automatic sharing as the important part.
- Social appraisal and complementary reactions (Manstead & Fischer 2001; Van Kleef et al. 2008, both discussed in the annurev review). An agent does not react to how another person expects it to feel.
- Appraisal beyond the OCC subset in `EmotionKind`. There are no hope-of-relief or sadness-for-others kinds, and no blended emotions beyond intensity sums.
- Mood-congruent appraisal and memory. A negative mood does not make negative events more likely to be appraised.
- Individual differences in emotion decay. The half-life is shared by every person.
- Dynamic baselines. The baseline is fixed at spawn from the traits and does not drift with life events.
- Facets and the Big Five beyond the three PAD mappings. Each trait enters the baseline only through the listed coefficients.
- Cultural display rules and norm-driven suppression.

## Sources

Corpus papers (full text read in home-still):

- `10.48550_arxiv.2304.03442` — Park et al. 2023, generative agents (memory and recency context; see `memory.md`). Not a source for PAD.
- `10.1146_annurev-psych-020821-010855` — Annual Review of Psychology 2022, contagion and affiliation. <https://doi.org/10.1146/annurev-psych-020821-010855>
- `10.18653_v1_p18-1017` — Mohammad 2018, VAD lexicon. <https://doi.org/10.18653/v1/P18-1017>
- `10.1145_2901739.2901752` — Adams, Graziotin & Mäntylä 2016, VAD in text mining. <https://doi.org/10.1145/2901739.2901752>

Metadata-only seeds (not in the corpus; cited from catalog metadata, full text not verified):

- Mehrabian 1996, PAD framework. <https://doi.org/10.1007/BF02686918>
- Mehrabian 1996, Big Five in the PAD model. <https://doi.org/10.1080/00049539608259510>
- Gebhard 2005, ALMA. <https://doi.org/10.1145/1082473.1082478>
- Hatfield et al. 1993, emotional contagion. <https://doi.org/10.1111/1467-8721.ep10770953>
- McCrae & John 1992, Big Five. <https://doi.org/10.1111/j.1467-6494.1992.tb00970.x>
- Goldberg 1990, Big Five markers. <https://doi.org/10.1037/0022-3514.59.6.1216>
- Russell 1980, circumplex model. <https://doi.org/10.1037/h0077714>
- Russell 2003, core affect. <https://doi.org/10.1037/0033-295X.110.1.145> (downloaded; conversion timed out, so not in the corpus)

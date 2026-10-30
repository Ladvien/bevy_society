# Memory

## What bevy_society does

Each person keeps a bounded list of memories (`Memories`, capacity `SocietySettings.memory_capacity` = 64). Every memory has a valence, an importance and a creation time. When a memory is scored for recall or for rumour telling, its weight is multiplied by a recency factor that decays with the hours elapsed since the memory was created. Familiarity with another person decays on a power law from the last contact. A grudge is the worst negative memory about one person, weighted by recency; the grudge a script reads (`ctx.grudge`) fades faster toward someone its holder still trusts.

## Equations as implemented

- Recency (`src/model/recall.rs:3–11`): `recency(h) = 0.995^max(0, h)`, where `h` is game-hours since the memory was **created**. Half-life ≈ 138 h. There is no tuning field; the 0.995 factor is fixed in code.
- Familiarity (`src/model/recall.rs:18–20`): `familiarity(peak, elapsed, τ, d) = peak · (1 + elapsed/τ)^(−d)`, with `τ = Tuning.familiarity_tau_days` = 1.0 and `d = Tuning.familiarity_decay_exp` = 0.5.
- Exposure raises the peak (`src/model/recall.rs:24–25`): `exposed_peak(f, rate, dt) = clamp(f + rate·dt·(1 − f), 0, 1)`, with `rate = Tuning.exposure_per_day` = 2.0.
- Grudge (`src/model/recall.rs`): `grudge = clamp(max over memories of max(0, −valence) · recency(h), 0, 1)` — the **worst** offence, not the count of them. **A design default, not a cited model.** It used to be a sum, which made the value depend on `memory_capacity`: four retained trivial slights saturated it exactly as one betrayal did. Measured in a cluster of twelve, `snub` (whose only consideration is the grudge) was 50.3% of activity starts when summing and 26.0% when taking the maximum, and at a second seed 48.4% → 32.6%. Recency is exponential (Park et al. 2023, DOI 10.48550/arXiv.2304.03442); the reduction from many events to one number is this crate's choice and no paper in the corpus settles it.
- Forgiveness (`src/model/recall.rs`, `forgiving_pace`; `src/snapshot.rs`, `ctx.grudge`): `ctx.grudge(a, b)` reads each memory at `h · (1 + forgiveness · trust(a → b))` instead of `h`, with `forgiveness = Tuning.forgiveness` = 12.0 and trust the holder's current tie trust in `[0, 1]` (0.5 with no tie). `Memories::grudge` is the same formula at pace 1. See "Forgiveness" below.
- Memory capacity (`src/mind.rs:433–438`): 64 memories per person, oldest dropped first. Design default (no source paper gives this number).
- Telling weight in rumour selection (`src/model/rumor_dk.rs:28–30`): `|valence| · belief · recency(h)`. See `rumor-and-reputation.md`.

## Why these defaults

- **Recency rate 0.995.** Park et al. (2023) use the same factor: "we treat recency as an exponential decay function over the number of sandbox game hours since the memory was last retrieved. Our decay factor is 0.995." (`10.48550_arxiv.2304.03442`, converted; passage checked). Park measures from the last **retrieval**. This crate measures from **creation**. This is a documented departure, recorded in `src/mind.rs` (`Memories` doc and `Memories::score`) and in `src/model/recall.rs`. It is a user decision in the finish plan, kept on purpose: this crate does not simulate retrieval, so a retrieval-based clock would have no event to reset it.
- **Power-law forgetting of familiarity.** Murre & Dros (2015) model forgetting with a power function, the shape used for familiarity here (`10.1371_journal.pone.0120644`, converted; power-function passage checked). Anderson & Schooler (1991) give a power-law shape for memory retention (DOI `10.1111/j.1467-9280.1991.tb00174.x`, not converted; cited from its DOI only).
- **Ebbinghaus-style decay in LLM agent memory.** MemoryBank (Zhong et al. 2023) builds forgetting on an Ebbinghaus curve (`10.48550_arxiv.2305.10250`, converted; checked). That supports time-based decay of stored memories, not the specific exponent used here.
- **Exposure.** Mere exposure (Zajonc 1968, DOI `10.1037/h0025848`, quoted in Zandstra 2000, `10.18174_197257`, converted) is the mechanism behind `exposure_per_day`. The value 2.0 is a design default (no source paper gives this number).

## Forgiveness (2026-10-30)

**Mechanism, not rate.** McCullough, Root Luna, Berry, Tabak & Bono 2010 (*Emotion*, DOI `10.1037/a0019349`): forgiveness rises with time since the offence, at a rate that differs between people, and was "uniquely associated with participants' perceptions that their relationships with their offenders retained value". Burnette, McCullough, Van Tongeren & Davis 2012 (*PSPB*, DOI `10.1177/0146167211424582`): "the combination of low exploitation risk and high relationship value predicted the greatest forgiveness." Strelan, Karremans & Krieg 2017 (*BJSP*, DOI `10.1111/bjso.12173`): the closeness → forgiveness link is carried by post-transgression trust, which "always played the main mediating role". Fehr, Gelfand & Nag 2010 (*Psychological Bulletin*, DOI `10.1037/a0019993`, 175 studies): apology correlates with forgiveness at r = .42. All four read from their abstracts (none is open access or in the corpus); the corpus search found no paper with a forgiveness rate.

This crate's directed trust is value and exploitation risk read together: it rises with each good exchange (an accepted `chat` +0.05 is the nearest thing the pack has to an apology) and falls by `witness_gain · harm` with each offence its holder suffers. So the pace a grievance fades at is `1 + forgiveness · trust`, applied to the age the existing exponential recency reads. McCullough et al.'s curve is logarithmic over months; the crate keeps Park's exponential and changes only its rate per relationship, which is the part of the finding about value.

**The value 12.0 is a design default, chosen by measurement** (`docs/loop/runs/2026-10-30-25/eval.md`). At trust 0.5 a grievance's half-life is 20 hours instead of 138; at trust 0.1 (a serial offender) 63 hours. Measured against HEAD at the same seeds (17-day village of 24, 30-day town of 60):

| run | insults | rival ties (directed) | friend ties (directed) |
|---|---|---|---|
| village seed 7 | 343 → 251 | 54 → 28 | 46 → 62 |
| village seed 42 | 382 → 240 | 56 → 24 | 30 → 65 |
| village seed 11 | 348 → 255 | 59 → 18 | 40 → 47 |
| village seed 99 | 361 → 249 | 66 → 36 | 35 → 54 |
| town seed 7 | 1740 → 1401 | 277 → 111 | 49 → 101 |
| town seed 99 | 1720 → 1491 | 229 → 155 | 52 → 86 |

The town's rivals respond smoothly to the gain (seed 7: 277, 205, 148, 89, 53 at 0, 4, 8, 16, 32), so the default is not on a cliff. **16 measured better and is not the default:** it takes the 200-person rumor book of `tests/rumor_book.rs` from 1 678 rumors to 1 315, of which 264 are still tellable (20.1%), over that test's pinned 20% line (rumors in the starter pack come only from witnessed insults, so fewer insults mean a smaller book with the same few hundred live ones). 12 keeps it at 18.3% (274 of 1 495). A host that wants a warmer society sets `forgiveness: 16.0` or more; `0.0` restores the unforgiving curve exactly.

## What the model leaves out

- Retrieval-based recency: the decay is from creation only, so a rehearsed memory does not refresh (Park's mechanism is not modelled).
- Relevance scoring against a query: memories are not ranked by topic match, only by recency and valence.
- Consolidation or interference between memories; capacity is a hard drop of the oldest memory.
- Memory of individual facts beyond valence, importance and subject. The model stores feelings about events, not their content.
- Forgetting of rumours by recall failure; a rumour dies by stifling or by its subject leaving the simulation (see `rumor-and-reputation.md`).

## Open contradictions and departures

- **Departure, documented:** recency is measured from creation, not from last retrieval (Park 2023). This is a user decision in the finish plan. Recorded in code and here. Park's claim concerns retrieval-time recency; this crate chooses creation time on purpose, so this is not a contradiction.
- **Unverified:** the Ebbinghaus and power-law exponents here are not fitted to any dataset. The exponent `d = 0.5` is a design default.

## Sources

- Park, J. S. et al. (2023). Generative Agents. https://doi.org/10.48550/arXiv.2304.03442 · home-still `10.48550_arxiv.2304.03442`
- Murre, J. M. J. & Dros, J. (2015). Replication and analysis of Ebbinghaus' forgetting curve. https://doi.org/10.1371/journal.pone.0120644 · `10.1371_journal.pone.0120644`
- Zhong, W. et al. (2023). MemoryBank. https://doi.org/10.48550/arXiv.2305.10250 · `10.48550_arxiv.2305.10250`
- Zandstra, E. (2000). Thesis on mere exposure. · `10.18174_197257`
- Anderson, J. R. & Schooler, L. J. (1991). Reflections of the environment in memory. https://doi.org/10.1111/j.1467-9280.1991.tb00174.x · not converted
- Zajonc, R. B. (1968). Attitudinal effects of mere exposure. https://doi.org/10.1037/h0025848 · not converted
- McCullough, M. E., Root Luna, L. M., Berry, J. W., Tabak, B. A. & Bono, G. (2010). On the form and function of forgiving. https://doi.org/10.1037/a0019349 · not in the corpus (abstract only)
- Burnette, J. L., McCullough, M. E., Van Tongeren, D. R. & Davis, D. E. (2012). Forgiveness results from integrating information about relationship value and exploitation risk. https://doi.org/10.1177/0146167211424582 · not in the corpus (abstract only)
- Strelan, P., Karremans, J. C. & Krieg, J. (2017). What determines forgiveness in close relationships? The role of post-transgression trust. https://doi.org/10.1111/bjso.12173 · not in the corpus (abstract only)
- Fehr, R., Gelfand, M. J. & Nag, M. (2010). The road to forgiveness: a meta-analytic synthesis of its situational and dispositional correlates. https://doi.org/10.1037/a0019993 · not in the corpus (abstract only)

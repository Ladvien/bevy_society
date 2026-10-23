# Memory

## What bevy_society does

Each person keeps a bounded list of memories (`Memories`, capacity `SocietySettings.memory_capacity` = 64). Every memory has a valence, an importance and a creation time. When a memory is scored for recall or for rumour telling, its weight is multiplied by a recency factor that decays with the hours elapsed since the memory was created. Familiarity with another person decays on a power law from the last contact. Grudges are a bounded sum over negative memories, each weighted by recency.

## Equations as implemented

- Recency (`src/model/recall.rs:3–11`): `recency(h) = 0.995^max(0, h)`, where `h` is game-hours since the memory was **created**. Half-life ≈ 138 h. There is no tuning field; the 0.995 factor is fixed in code.
- Familiarity (`src/model/recall.rs:18–20`): `familiarity(peak, elapsed, τ, d) = peak · (1 + elapsed/τ)^(−d)`, with `τ = Tuning.familiarity_tau_days` = 1.0 and `d = Tuning.familiarity_decay_exp` = 0.5.
- Exposure raises the peak (`src/model/recall.rs:24–25`): `exposed_peak(f, rate, dt) = clamp(f + rate·dt·(1 − f), 0, 1)`, with `rate = Tuning.exposure_per_day` = 2.0.
- Grudge (`src/model/recall.rs:31–34`): `grudge = clamp(Σ max(0, −valence) · recency(h), 0, 1)`, summed over the person's memories.
- Memory capacity (`src/mind.rs:433–438`): 64 memories per person, oldest dropped first. Design default (no source paper gives this number).
- Telling weight in rumour selection (`src/model/rumor_dk.rs:28–30`): `|valence| · belief · recency(h)`. See `rumor-and-reputation.md`.

## Why these defaults

- **Recency rate 0.995.** Park et al. (2023) use the same factor: "we treat recency as an exponential decay function over the number of sandbox game hours since the memory was last retrieved. Our decay factor is 0.995." (`10.48550_arxiv.2304.03442`, converted; passage checked). Park measures from the last **retrieval**. This crate measures from **creation**. This is a documented departure, recorded in `src/mind.rs` (`Memories` doc and `Memories::score`) and in `src/model/recall.rs`. It is a user decision in the finish plan, kept on purpose: this crate does not simulate retrieval, so a retrieval-based clock would have no event to reset it.
- **Power-law forgetting of familiarity.** Murre & Dros (2015) model forgetting with a power function, the shape used for familiarity here (`10.1371_journal.pone.0120644`, converted; power-function passage checked). Anderson & Schooler (1991) give a power-law shape for memory retention (DOI `10.1111/j.1467-9280.1991.tb00174.x`, not converted; cited from its DOI only).
- **Ebbinghaus-style decay in LLM agent memory.** MemoryBank (Zhong et al. 2023) builds forgetting on an Ebbinghaus curve (`10.48550_arxiv.2305.10250`, converted; checked). That supports time-based decay of stored memories, not the specific exponent used here.
- **Exposure.** Mere exposure (Zajonc 1968, DOI `10.1037/h0025848`, quoted in Zandstra 2000, `10.18174_197257`, converted) is the mechanism behind `exposure_per_day`. The value 2.0 is a design default (no source paper gives this number).

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

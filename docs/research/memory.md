# Memory

## What bevy_society does

Each person keeps a bounded list of memories of interactions, witnessed deeds, heard rumours, conversations and custom events. Each memory records when it happened, what kind it is, who it concerns, its valence (good or bad) and its importance. A memory's pull on recall decays exponentially with the hours since it was formed, so recent and important memories surface first, and the oldest, least important one is evicted when the list is full. Relationships forget the same way: how familiar a person is to another fades as a power law of the time since their last contact, and each contact resets that clock. Rumours fade for teller and listener alike, so a stale story is less likely to be passed on.

## Equations as implemented

**Recency (`src/model/recall.rs::recency`).** r(h) = 0.995^max(0, h), where h is the number of game-hours since the memory was formed. In the code, h = (now_days − memory.day) · 24 (`Memories`, `src/mind.rs`). The factor 0.995 is set in `recall.rs`; no setting exists for it. Source: Park et al. 2023, DOI 10.48550/arXiv.2304.03442 (verified, see Sources).

**Recall score (`Memories::recall`).** For a memory m, with `about` the person the query concerns:

score(m) = r(h_m) + importance_m + [other_m == about]

The recall call returns the top n by this score. The `about` bonus of 1 is a design term, not taken from Park et al.

**Eviction (`Memories`, capacity 64).** When the list holds `memory_capacity` entries and a new one arrives, the entry with the lowest r(h) + importance is removed. The default capacity is `SocietySettings::memory_capacity` = 64. The code comment on `Memories` attributes the 64 to Park et al. 2023. The lines of that paper that were checked do not state a capacity of 64, so the attribution is unsupported in the corpus text (see Discrepancies).

**Grudge (`Memories::grudge`).** grudge(other) = min(1, Σ max(0, −valence_m) · r(h_m)), summed over memories m about `other`.

**Familiarity (`Ties::familiarity`, `src/model/recall.rs::familiarity`).** Given the peak familiarity f_peak, the time since last contact Δt in game-days, τ = `familiarity_tau_days` = 1.0, and d = `familiarity_decay_exp` = 0.5:

f(Δt) = f_peak · (1 + Δt/τ)^(−d)

The power law and the exponent 0.5 are the design. The corpus contains no paper that verifies the exponent or the (1 + t/τ)^(−d) form (see Discrepancies and Sources).

**Exposure (mere exposure, Zajonc 1968).** On each contact, with f = f(now) and Δt_step in game-days, `exposure_per_day` = 2.0:

- f_peak ← clamp(f + 2.0 · Δt_step · (1 − f), 0, 1)
- last_contact ← now
- affinity ← affinity + `mere_exposure_per_day` · Δt_step · (1 − |affinity|), with `mere_exposure_per_day` = 0.05

**Affinity relaxation.** affinity relaxes toward 0 by `affinity_relax_per_day` · Δt_step = 0.01 per game-day, never crossing 0.

**Rumour telling weight (`src/model/rumor_dk.rs::telling_weight`).** Among the rumours a teller spreads, the weight is |valence| · belief · r(h_heard), where h_heard is the number of hours since the teller heard the rumour. This uses the same recency function.

## Why these defaults

- **Exponential recency at 0.995 per hour.** Park et al. 2023 (`10.48550_arxiv.2304.03442`, indexed, full text read) say: "we treat recency as an exponential decay function over the number of sandbox game hours since the memory was last retrieved. Our decay factor is 0.995." The value matches the code, and the verification is the direct quote. Park et al. also state that the three recall components (recency, importance, relevance) are min-max normalised to [0, 1] and summed with equal weights (α = 1). The code keeps the 0.995 and the equal-weight idea but does not min-max normalise (see Differences).
- **Forgetting as a power law.** The code uses a power law for familiarity rather than an exponential. The plan cites Anderson & Schooler 1991, DOI 10.1111/j.1467-9280.1991.tb00174.x, and Murre & Dros 2015, DOI 10.1371/journal.pone.0120644, for this form. Both are metadata-only in the corpus: the Murre & Dros PDF was downloaded but has no catalog entry, and the Anderson & Schooler full text is not in home-still. Neither full text was read, so the power-law claim is not verified, and the exponent 0.5 that the code uses is also not verified.
- **Mere exposure.** Zajonc 1968, DOI 10.1037/h0025848, is metadata-only. A corpus thesis on food, `10.18174_197257` (Zandstra 2000, page 13), quotes it: "The mechanism of 'mere exposure' was originally proposed by Zajonc (1968), who suggested that mere exposure to a stimulus was a sufficient condition for enhancement of one's attitude toward that stimulus." The same thesis also says "High frequency or extended exposure to foods may, however, also lead to a decrease in liking." The code's mere-exposure gain is small and capped by (1 − |affinity|); that cap is a design choice, not taken from a paper. The decrease with extended exposure is not modelled (see Leaves out). Byrne 1961, DOI 10.1037/h0044721, is metadata-only and is cited for the familiarity–liking link.
- **Forgetting-curve framing.** MemoryBank (Zhong et al. 2024, `10.1609_aaai.v38i17.29946`, page 7) frames memory updating on an Ebbinghaus-style forgetting curve: "Taking inspiration from this theory, we integrate a memory updating mechanism." It gives the framing only, not the exponents.

## What the model leaves out

- **Retrieval refresh.** Park et al. decay recency from the last retrieval. The code decays a memory from its creation, so recalling a memory does not refresh it. This is a difference from Park et al. (see Differences below); the corpus does not justify it.
- **Reflection and synthesis.** Park et al. have an agent generate higher-level reflections from memories. This crate has no reflection step.
- **Relevance by embedding.** Park et al. score relevance semantically. The code uses only whether the memory concerns the queried person.
- **Consolidation and distortion.** Memories never change after creation, and rumours do not mutate as they spread (no telephone-game drift).
- **Per-kind capacity.** Eviction is global. A flood of rumours can push out a person's only memory of a kind.
- **Decay of importance.** Importance is fixed at creation. Only recency decays.
- **Forgetting by exposure.** The mere-exposure caveat (extended exposure can reduce liking) is not modelled: exposure only increases familiarity and affinity.

## Differences from Park et al. 2023 (not numeric discrepancies)

- **Decay origin.** Park: since last retrieval. Code: since creation (`memory.day`).
- **Normalisation.** Park: min-max over the candidate set, then an equal-weight sum. Code: raw values summed, so the about-bonus of 1 dominates recency and importance.

## Discrepancies

- **`Memories` capacity attribution** (`src/mind.rs`, `Memories`, comment naming Park et al. 2023 for `memory_capacity` = 64). The corpus full text of Park et al. (`artifact://598`, grep for "capacity" and "64") contains no capacity of 64; the only capacity discussion is about the limits of generative agents. Value in code: 64. Paper value: none found. Action for Main: either cite the source of 64 or make it a design default with no attribution. The number itself is not disputed by a paper.
- **Familiarity exponent** (`src/model/recall.rs::familiarity` doc comment says the exponent 0.5 is "to be confirmed against docs/research/memory.md"). Not verified by any corpus paper. This note records the exponent as a design default, not a paper value. It is not a discrepancy, because no corpus paper contradicts it.

## Sources

Corpus papers (text checked in home-still; Zandstra and the textbook were checked by excerpt, not in full):

- `10.48550_arxiv.2304.03442` — Park et al. 2023, generative agents; recency decay 0.995 per game-hour since last retrieval, equal-weight recall score. <https://doi.org/10.48550/arXiv.2304.03442>
- `10.1609_aaai.v38i17.29946` — Zhong et al. 2024, MemoryBank; forgetting-curve framing. <https://doi.org/10.1609/aaai.v38i17.29946>
- `10.18174_197257` — Zandstra 2000, thesis on food liking; mere exposure (Zajonc 1968) and its decrease with extended exposure. <https://doi.org/10.18174/197257>
- `W1493777802` — Research Methods in Psychology (2016), textbook; mere-exposure definition (page 68).

Metadata-only seeds (not in the corpus; cited from catalog metadata, full text not verified):

- Anderson & Schooler 1991, memory and the environment. <https://doi.org/10.1111/j.1467-9280.1991.tb00174.x>
- Murre & Dros 2015, replication of the forgetting curve (downloaded, no catalog entry). <https://doi.org/10.1371/journal.pone.0120644>
- Zajonc 1968, attitudinal effects of mere exposure. <https://doi.org/10.1037/h0025848>
- Byrne 1961, interpersonal attraction and attitude similarity. <https://doi.org/10.1037/h0044721>

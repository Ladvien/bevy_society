# Opinion dynamics

## What bevy_society does

Each person holds one opinion per topic, a value in `[-1, 1]`, and a per-topic confidence that sets how far apart two opinions can be and still influence each other. An interaction between two people moves both opinions toward each other when they are within the confidence bound (bounded confidence, Deffuant-style). Opinions also drift toward a stubborn anchor, get pulled toward the mean of a confident group (conformity), and feed back into affinity when two people already agree. The model functions are on HEAD (`src/model/opinion.rs`). The wiring that runs them on a live tick is D5/D7 work (`src/beliefs.rs` `conform_opinions`, `beliefs::exchange_opinions`), which lands with the finish plan.

## Equations as implemented

- Confidence bound (`src/model/opinion.rs:6–8`): `bound = base + gain · O`, with `base = Tuning.confidence_base` = 0.2, `gain = Tuning.confidence_openness` = 0.6, and `O` the person's openness in `[0, 1]`.
- Deffuant step (`src/model/opinion.rs:13–21`): for opinions `x_i`, `x_j` and difference `d = x_j − x_i`, with ε the confidence bound and `μ = Tuning.influence_mu` = 0.3: if `|d| < ε`, the change is `μ · weight · d`; otherwise 0. The `weight` is the affinity weight defined below.
- Anchor pull (`src/model/opinion.rs:24–26`): `Δx = rate · stubbornness · (anchor − x)`, with `rate = Tuning.anchor_pull` = 0.1.
- Affinity from agreement (`src/model/opinion.rs:31–33`): `Δaff = similarity_gain · (1 − |d|)`, with `similarity_gain = Tuning.similarity_gain` = 0.05.
- Conformity (`src/model/opinion.rs:42`, gate at `:58–60`): applies when `|m| > 0.3`, where `m` is the group's mean opinion on the topic. Strength follows `n / (n + 3)` for `n` contacts and `Tuning.conformity_per_day` = 0.2 (design default).
- Affinity weighting (`src/model/opinion.rs:65`): `weight_j = max(0.1, aff(i→j) + 1)`, a weighted mean over the group.

## Why these defaults

- **Bounded confidence threshold.** Flache et al. (2017) describe the Deffuant-style rule with a threshold ε and a convergence parameter μ ≤ 0.5 (§2.24–2.26, converted; passage checked, citing Deffuant). They note that small confidence gives fragmented clusters (§2.30, converted; passage checked) and that the cluster count follows from the interval width (§2.42, converted). The confidence bound here is linear in openness so that open people tolerate a wider range, which is a design default (no source paper gives this linear form).
- **Update rule and μ.** Castellano, Fortunato & Loreto (2009) state the bounded-confidence update as `x_i(t+1) = x_i(t) + μ[x_j(t) − x_i(t)]` with `μ ∈ [0, 1/2]`, and no change when `|x_i − x_j| > ε` (eq. 27, converted; passage checked). They also give `n_c ≈ 1/(2ε)` (converted; checked). `μ = 0.3` sits in that range.
- **Consensus and ε.** Castellano et al. state that for ε > 1/2 on complete graphs the population reaches consensus (converted; checked). Flache et al. (§3.13, converted) report that threshold values have "hardly been tested" empirically, so the choice of ε is not empirically anchored here.
- **Opinion networks.** Hansen & Ghrist (`10.48550_arXiv.2005.12798`, converted; passage checked) describe Hegselmann and Krause's bounded confidence as "a threshold *r* for opinion sharing", which is the same mechanism in a different notation.
- **Anchor pull and similarity.** Design defaults (no source paper gives these numbers). The mechanisms are standard stubborn-agent and homophily accounts; the values are not fitted.

## What the model leaves out

- Network structure: every interaction is local to the pair or the group the tick supplies; there is no fixed graph and no rewiring.
- Noise, or the random-pairing dynamics of Hegselmann–Krause beyond what the scheduler gives.
- Opinions are not coupled across topics; each topic moves on its own.
- Repeated-contact memory of disagreement; the bound is applied per interaction.
- Empirical calibration of ε, μ or the conformity strength.

## Open contradictions and departures

- **Departure, documented (affinity weight):** Castellano et al. eq. (27) has no weight. The code multiplies `μ` by `weight = max(0.1, aff + 1)` (`src/model/opinion.rs:13–21`, `:65`). This is the affinity-weighted extension in SPEC, not a contradiction of the base rule: the bounded-confidence structure (threshold ε, step μ, no change outside ε) is confirmed by the converted sources.
- **Not a contradiction, not yet checked:** Deffuant 2000 (DOI `10.1142/s0219525900000078`) is the primary source for the rule. Not converted, so the form above is confirmed only through Castellano and Flache. Listed as wanted.
- **Unverified:** Hegselmann & Krause (2002) primary text is not converted; the threshold claim above rests on Hansen & Ghrist's description.

## Sources

- Flache, A. et al. (2017). Models of Social Influence: Towards the Next Frontiers. JASSS 20(4). https://doi.org/10.18564/jasss.3521 · `10.18564_jasss.3521`
- Castellano, C., Fortunato, S. & Loreto, V. (2009). Statistical physics of social dynamics. Rev. Mod. Phys. https://doi.org/10.1103/RevModPhys.81.591 · `10.1103_revmodphys.81.591`
- Hansen, J. & Ghrist, R. (2020). Opinion dynamics on discourse sheaves. https://doi.org/10.48550/arXiv.2005.12798 · `10.48550_arXiv.2005.12798`
- Deffuant, G. et al. (2000). Mixing beliefs among interacting agents. https://doi.org/10.1142/s0219525900000078 · not converted
- Hegselmann, R. & Krause, U. (2002). Opinion dynamics and bounded confidence. · not converted

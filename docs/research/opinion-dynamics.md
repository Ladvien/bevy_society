# Opinion dynamics

Corpus status: distill_search and abstract_search did not return the canonical papers below in a usable form, and catalog_read reports no converted entry for the Deffuant 2000 stem (10.1142_s0219525900000078). The sources here are cited from paper_get (OpenAlex/Crossref) metadata and abstracts, not from converted full text. They belong in awaiting-conversion until Phase A's report runs.

## What bevy_society does

Each person holds one opinion value per topic in [-1, 1], an anchor (the view it started with), and a stubbornness weight. When two people share an accepted interaction whose topics include one of these topics, each moves toward the other only if they are within a confidence bound that grows with the person's openness, and the size of the move depends on how much each trusts and likes the other. Opinions also drift back toward the anchor. A person who has at least three contacts in range drifts toward the affinity-weighted group mean, but only when that mean is clearly positive or clearly negative. Every disagreement also nudges affinity: similar views attract, and strong disagreement pushes affinity down.

## Equations as implemented

All constants are the defaults of `BeliefRates` in `src/beliefs.rs` and `Tuning` in `src/core.rs`.

1. Confidence bound: epsilon_i = confidence_base + confidence_openness * O_i = 0.2 + 0.6 * O_i, where O_i is the openness trait in [0, 1], so epsilon lies in [0.2, 0.8]. Source: `Opinion::influence` through `confidence_bound` in `src/model/opinion.rs`. Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
2. Pairwise move, applied to each listed topic of an accepted interaction in both directions: d = x_j - x_i. If |d| < epsilon_i, then x_i += influence_mu * w_ij * d, with influence_mu = 0.3 and w_ij = 0.5 * trust(i->j) + 0.5 * max(0, affinity(i->j)). If |d| >= epsilon_i, this term is zero. Deffuant et al. 2000, DOI 10.1142/S0219525900000078. The weight w_ij is this crate's choice; the paper does not define it.
3. Anchor pull, applied in the same call after step 2: x_i += anchor_pull * stubbornness_i * (anchor_i - x_i), with anchor_pull = 0.1 and stubbornness defaulting to 0.2. The anchor is the value the person started with. Friedkin and Johnsen 1990, DOI 10.1080/0022250X.1990.9990069.
4. Similarity attraction: affinity(i->j) += similarity_gain * (1 - |d|), with similarity_gain = 0.05, computed from the d before step 2. Because |d| lies in [0, 2], the term is negative when |d| > 1. Cited for direction only: Byrne 1961, DOI 10.1037/h0044721, and McPherson, Smith-Lovin and Cook 2001, DOI 10.1146/annurev.soc.27.1.415.
5. Conformity: for a person with n >= 3 contacts in range, m = sum_k w_k x_k / sum_k w_k, where w_k = max(0.1, affinity_k + 1) is the person's affinity toward contact k. The move applies only if |m| > 0.3: x_i += conformity_per_day * A_i * (n / (n + 3)) * dt_days * (m - x_i), with conformity_per_day = 0.2 and A_i = agreeableness. Asch 1956, DOI 10.1037/h0093718; Nowak, Szamrej and Latané 1990, DOI 10.1037/0033-295X.97.3.362. The weighting is this crate's reading of Nowak et al.; the weight formula and the gate |m| > 0.3 are not in the abstract.
6. Each value is clamped to [-1, 1] after the moves.

Steps 2 to 4 run once per accepted interaction and are not multiplied by dt_days. Step 5 is per day, scaled by dt_days. The plan's convention is that every rate is per game-day, so steps 2 to 4 are an exception to it in code, and the note records it here rather than in the code.

Discrepancy: the Deffuant 2000 abstract confirms the threshold mechanism but gives no numeric ranges for mu (influence_mu = 0.3) or for epsilon (0.2 to 0.8 through openness). The must-confirm item for mu and epsilon is therefore still open. Nothing in the abstract contradicts the defaults, and nothing confirms them. Likewise, the gate |m| > 0.3, the factor n/(n+3) and the weight max(0.1, affinity + 1) are not in the Nowak et al. 1990 abstract, so they are the crate's choices.

## Why these defaults

Deffuant et al. 2000 (doc_id not converted): "High thresholds yield convergence of opinions towards an average opinion, whereas low thresholds result in several opinion clusters." This is why epsilon rises with openness, so open people converge and closed people form clusters.

Friedkin and Johnsen 1990: the paper describes "a network of interpersonal influences and the content of individuals' opinions." The anchor term follows this framing of influence plus an initial opinion. The abstract gives no formula for the stubbornness weighting, so 0.2 and 0.1 are the crate's values.

Nowak, Szamrej and Latané 1990: "an incomplete polarization of opinions reaching a stable equilibrium, with coherent minority subgroups managing to exist near the margins." This supports gating conformity behind a threshold on the group mean, so that mixed groups do not collapse. The value 0.3 is the crate's.

Latané 1981: "impact should be a multiplicative function of the strength, immediacy, and number of other people," and "the marginal effect of the Nth other person being less than that of the (N-1)th." This motivates a diminishing-returns factor in group size. The n/(n+3) term is a saturating choice that resembles the paper's diminishing returns. It is not derived from Latané's power law.

Discrepancy (citation): Latané 1981 gives a power law in strength, immediacy and number, which the crate does not implement. The n/(n+3) factor only mirrors the diminishing-returns shape.

## What the model leaves out

- Synchronous averaging over neighbourhoods (Hegselmann and Krause 2002). Updates are asynchronous pairwise encounters, as in Deffuant et al., so there is no synchronous convergence theorem to rely on.
- Repulsion. Opinions further apart than epsilon are never pushed further apart. Polarisation beyond clusters does not come from the opinion term; disagreement only lowers affinity.
- The full social-impact function of Latané 1981: strength, immediacy and number as a power law. Only the group-size factor is present.
- Per-person mu. influence_mu is a single global value.
- Cross-topic coupling. Topics are independent of one another.
- Minority persistence. The conformity gate is the only mechanism for keeping a minority view. Nowak et al. 1990 studied it in a full model.
- Confidence depends only on openness. The other traits do not change epsilon.
- Forgetting of opinion changes. Opinions have no memory of earlier values.
- Network-level conformity. Conformity only counts the three or more people in direct contact range.

## Sources

- Deffuant, Neau, Amblard and Weisbuch 2000, "Mixing beliefs among interacting agents": https://doi.org/10.1142/S0219525900000078 (abstract read via paper_get; catalog has no converted entry).
- Friedkin and Johnsen 1990, "Social influence and opinions": https://doi.org/10.1080/0022250X.1990.9990069 (abstract read via paper_get).
- Nowak, Szamrej and Latané 1990, "From private attitude to public opinion: A dynamic theory of social impact": https://doi.org/10.1037/0033-295X.97.3.362 (abstract read via paper_get).
- Latané 1981, "The psychology of social impact": https://doi.org/10.1037/0003-066X.36.4.343 (abstract read via paper_get).
- Asch 1956: https://doi.org/10.1037/h0093718 (cited from the plan; not read in this pass).
- Byrne 1961: https://doi.org/10.1037/h0044721 (cited from the plan; not read in this pass).
- McPherson, Smith-Lovin and Cook 2001: https://doi.org/10.1146/annurev.soc.27.1.415 (cited from the plan; not read in this pass).
- Candidate-pool titles only, not read: Hegselmann and Krause 2002, "Opinion dynamics and bounded confidence: models, analysis and simulation" (OpenAlex 146082c908eae45f8e6e8db0c94916223304a4c1); 10.1371/journal.pone.0043507, "Multi-Level Opinion Dynamics under Bounded Confidence"; 10.1016/j.automatica.2023.111302, "Bounded confidence opinion dynamics: A survey".
- Home-still doc_ids: none converted at time of writing. distill_search returned off-topic results for these queries.

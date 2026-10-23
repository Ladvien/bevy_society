# Relationships

## What bevy_society does

Every person keeps a sorted list of directed ties, one per person it has met. A tie stores affinity (from -1 to 1), trust (0 to 1), respect (-1 to 1), and a familiarity peak that fades as time passes without contact. Ties grow through repeated contact (mere exposure), drift back toward neutral when unused, and are nudged by structural balance (friends of friends tend to agree about third parties). Each person keeps at most 150 ties, and drops the least familiar ones first. A tie is labelled Friend, Rival, Acquaintance or Stranger from its affinity and familiarity, and a label change is written as a message.

## Equations as implemented

All rates are per game-day and are multiplied by `dt_days`, the length of one tick in game-days. Defaults are the `TieRates` fields in `src/relationships.rs`.

- Familiarity, power-law forgetting: f(t) = peak · (1 + (t − t_last) / τ)^(−d). Defaults τ = `familiarity_tau_days` = 1.0 and d = `familiarity_decay_exp` = 0.5. Sources: Anderson & Schooler 1991, DOI 10.1111/j.1467-9280.1991.tb00174.x (power-law forgetting); Murre & Dros 2015, DOI 10.1371/journal.pone.0120644 (power-law retention model). The exponent 0.5 is the code's choice; see the discrepancy below.
- Mere exposure on each contact, both directions: f = f(t); peak' = clamp(f + `exposure_per_day` · dt · (1 − f), 0, 1) with `exposure_per_day` = 2.0; affinity' = clamp(affinity + `mere_exposure_per_day` · dt · (1 − |affinity|), −1, 1) with `mere_exposure_per_day` = 0.05. Source: Zajonc 1968, DOI 10.1037/h0025848 (repeated exposure raises liking). The numeric rates are the code's choice.
- Affinity relaxes toward zero when there is no contact: affinity' = relax_toward_zero(affinity, `affinity_relax_per_day` · dt), with `affinity_relax_per_day` = 0.01. It never crosses zero. No source paper gives this rate.
- Structural balance: each tick, `balance_samples_per_tick` = 8 triads are sampled. A is uniform over the sorted persons. B and C are uniform over A's ties, and the tie B→C must exist. Then affinity(A→C) += `balance_rate_per_day` · dt · affinity(A→B) · affinity(B→C), with `balance_rate_per_day` = 0.5. Sources: Heider 1946, DOI 10.1080/00223980.1946.9917275 (balance theory); Cartwright & Harary 1956, DOI 10.1037/h0046049 (balance in signed graphs). The sampling rate is the code's choice.
- Capacity: when a person has more than `max_ties` = 150 ties, the ties with the lowest familiarity are removed until the count is 150. Source: Dunbar 1992, DOI 10.1016/0047-2484(92)90081-J (about 150 stable relationships). The "close" set is the top `close_ties` = 15 ties by max(0, affinity) · familiarity. Source: Zhou, Sornette, Hill & Dunbar 2005, DOI 10.1098/rspb.2004.2970 (layered sympathy groups).
- Labels, thresholds set in code (no source paper): Friend if affinity ≥ 0.4 and familiarity ≥ 0.3; Rival if affinity ≤ −0.4 and familiarity ≥ 0.2; Stranger if familiarity < 0.1; otherwise Acquaintance.
- Triadic closure: when A and B interact while C, a close tie of A, is in contact range, the familiarity between B and C is raised by `introduction_familiarity` = 0.1. Source: Kossinets & Watts 2006, DOI 10.1126/science.1116869 (triadic closure in an email network). The value 0.1 is the code's choice; the paper gives no such number.

## Why these defaults

The corpus gives qualitative support for the mechanisms and no numeric support for the rates. Gillespie et al. 2015 (doc_id `10.1371_journal.pone.0128900`) reports from 25,185 online respondents that "Homophily is the notion that people affiliate with others who are similar to themselves", and describes gender homophily as "a well-established phenomenon in heterosexual friendships". That supports the idea that similarity drives tie formation, which is the basis for the affinity gain from contact and for the similarity term in the opinion notes. The paper reports self-reported friend counts, so it does not inform any rate in the model. Note that the Gillespie et al. data is self-report from one website, so it is weak evidence for the general case.

The mere-exposure and balance rates, the affinity relaxation rate, the familiarity exponent, the label thresholds and the introduction familiarity were chosen to keep ties moving at a visible pace over game-days. They are not fitted to data.

## What the model leaves out

- Tie dissolution beyond the capacity rule: ties are never deleted on a bad experience; they only lose familiarity and drift toward zero.
- Homophily on attributes such as age, gender, income or occupation. Only faction membership and affinity drive similarity.
- Strength of weak ties (Granovetter 1973, DOI 10.1086/225469): the model does not separate weak from strong bridges.
- Reciprocity asymmetry beyond the directed edges: there is no explicit exchange-based rule (Homans 1958; Emerson 1976).
- Balance over all triads: only 8 random triads per tick are sampled, so balance converges slowly on large villages.
- Trust decay: trust changes only through deeds, gossip and witnessing, not with time.
- Attachment styles, power-law degree growth and preferential attachment (Barabási & Albert 1999, DOI 10.1126/science.286.5439.509; Watts & Strogatz 1998, DOI 10.1038/30918). The network is built by contact, not by a growth rule.

## Discrepancy

- `src/model/recall.rs` cites `docs/research/memory.md` to confirm the familiarity exponent 0.5. That note does not exist yet (`docs/research/` contains only `corpus/`), and no corpus paper I searched gives the exponent. The value stays unconfirmed.
- `src/relationships.rs` `introduction_familiarity` (0.1) is attributed to Kossinets & Watts 2006 in `src/core.rs`. That paper gives no such value. The number is a plan choice and should be described that way.
- `affinity_relax_per_day` (0.01) has no source paper. `src/model/balance.rs` says so and points to this note; this note cannot supply one.
- The label thresholds (0.4, 0.3, 0.2, 0.1) have no source paper.

## Sources

Corpus (home-still):

- `10.1371_journal.pone.0128900`: Gillespie, Frederick, Harari & Grov 2015, "Homophily, Close Friendship, and Life Satisfaction among Gay, Lesbian, Heterosexual, and Bisexual Men and Women", https://doi.org/10.1371/journal.pone.0128900. Read in full (region 1). Used for the homophily claim only.

Cited from metadata only (not in the corpus, so they do not count toward the corpus minimum):

- Heider 1946, https://doi.org/10.1080/00223980.1946.9917275
- Cartwright & Harary 1956, https://doi.org/10.1037/h0046049
- Zajonc 1968, https://doi.org/10.1037/h0025848
- Byrne 1961, https://doi.org/10.1037/h0044721
- McPherson, Smith-Lovin & Cook 2001, https://doi.org/10.1146/annurev.soc.27.1.415
- Dunbar 1992, https://doi.org/10.1016/0047-2484(92)90081-J
- Zhou et al. 2005, https://doi.org/10.1098/rspb.2004.2970
- Granovetter 1973, https://doi.org/10.1086/225469
- Kossinets & Watts 2006, https://doi.org/10.1126/science.1116869
- Barabási & Albert 1999, https://doi.org/10.1126/science.286.5439.509
- Watts & Strogatz 1998, https://doi.org/10.1038/30918
- Anderson & Schooler 1991, https://doi.org/10.1111/j.1467-9280.1991.tb00174.x
- Murre & Dros 2015, https://doi.org/10.1371/journal.pone.0120644
- Homans 1958, https://doi.org/10.1086/222355; Emerson 1976, https://doi.org/10.1146/annurev.so.02.080176.002003

Corpus gaps: searches for balance theory, triadic closure, mere exposure, and friendship-formation models returned no matching corpus papers. Those topics need papers added through the Phase A pipeline.

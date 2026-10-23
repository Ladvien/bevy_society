# Groups and status

## What bevy_society does

Each person belongs to zero or more factions. Members start ties to fellow members with a small affinity head start and accept each other's proposals a little more easily. Accepted interactions with someone from another faction that do no harm build a small amount of affinity, so contact can soften group lines. A person's standing is the average respect that people they know well have for them. When two people contest something, the winner is drawn from a logistic function of mood dominance plus standing, and the outcome shifts respect and mood for the loser, the winner and the witnesses. A planned social-field input gives each activity a score from how much its cell is occupied by the person's own faction compared with others, which is the Schelling tolerance idea.

## Equations as implemented

Defaults are the `Tuning` and `FactionDef`/`AcceptDef` fields in `src/core.rs`, `src/defs.rs` and `src/groups.rs`. All rates are per game-day and multiplied by `dt_days`.

- Ingroup bias: a new tie between co-members of a faction starts at affinity `ingroup_bias` = 0.2 (`FactionDef.ingroup_bias`, default 0.2). Source: Tajfel et al. 1971, DOI 10.1002/ejsp.2420010202 (minimal group paradigm). Tajfel 1971 gives no number, so 0.2 is the code's choice.
- Acceptance bonus: when actor and target share a faction, the acceptance probability gains `AcceptDef.ingroup` = 0.1. The same paper is cited; the value is the code's choice.
- Intergroup contact: an accepted interaction with no harmful deed between persons who share no faction adds `contact_gain` = 0.02 to affinity in both directions. Source: Pettigrew & Tropp 2006, DOI 10.1037/0022-3514.90.5.751 (contact reduces prejudice, meta-analysis). The 0.02 value is the code's choice.
- Standing: S_i = mean of respect(j→i) over persons j with familiarity(j→i) ≥ 0.2; S_i = 0 when there are none. Sources: Magee & Galinsky 2008, DOI 10.5465/19416520802211628 (status conferred by others); Berger, Cohen & Zelditch 1972, DOI 10.2307/2093465 (status characteristics).
- Contest: p(actor wins) = logistic(k · ((D_a + S_a) − (D_t + S_t))), with logistic(k, x0, x) = 1/(1 + e^(−k(x − x0))), k = `contest_steepness` = 4.0, x0 = 0, D = mood dominance, S = standing. One RNG draw decides the outcome. Sources: Bonabeau, Theraulaz & Deneubourg 1995, DOI 10.1016/0378-4371(95)00064-e (dominance by logistic contests); Hemelrijk 1999, DOI 10.1098/rspb.1999.0646 (dominance hierarchy in simulated groups). The steepness value is the code's choice.
- Respect after a contest: the loser's respect for the winner += 0.1; the winner's respect for the loser −= 0.05; each witness's respect for the winner += 0.05. Shame 0.4 for the loser and Pride 0.4 for the winner. These amounts are plan D6 choices; no source paper gives them, and the interaction code that applies them is not in the tree at 97fa17c.
- Schelling tolerance input (planned, social-field activity scoring, D9): OwnShare = own / (own + other) at the target cell, where own is the person's own faction's presence and other is the sum over other factions; OwnShare = 0.5 when both are 0. Source: Schelling 1971, DOI 10.1080/0022250X.1971.9989794 (segregation as a threshold process). The formula is the plan's definition; the paper describes the threshold idea, not this exact ratio.

## Why these defaults

The corpus has no paper on factions, intergroup bias, standing, contests, dominance or segregation. The searches I ran (dominance hierarchy, coalition formation, social identity, norm emergence, Schelling) returned no matching corpus paper. The only corpus hit near the topic was an autism-camouflaging study that uses social identity theory as background, and I did not rely on it.

So the defaults are judgement values, not fitted or quoted from the cited papers. The ingroup bias of 0.2, the acceptance bonus of 0.1, the contact gain of 0.02, the contest steepness of 4.0 and the respect amounts were chosen to make group effects visible within a few game-days without swamping individual experience. Tajfel 1971 and Pettigrew & Tropp 2006 support the direction of each effect; neither gives the magnitude.

## What the model leaves out

- Factions do not form, split or merge in play. Membership is set by the host (`join`/`leave` in scripts).
- No stereotype or prejudice beyond the ingroup bonus and the single contact gain.
- No status from occupation, wealth or role; standing comes only from respect among familiar persons.
- Contests are one-shot draws, with no escalation, no alliance and no persistent rank order beyond standing.
- No coalition formation (Axelrod 1986, DOI 10.2307/1960858; Centola & Baronchelli 2015, DOI 10.1073/pnas.1418838112) and no norm emergence (Young 1993, DOI 10.2307/2951778).
- The Schelling input is a single cell's ratio. It does not model movement toward a preferred neighbourhood mix.
- Faction stats (`FactionStats`) describe the current state only and are not used as input to any rule.

## Discrepancy

- `src/defs.rs` and `src/core.rs` attribute the ingroup bias (0.2) and the acceptance bonus (`AcceptDef.ingroup` 0.1) to Tajfel et al. 1971. That paper gives no numeric values, so the attribution overstates the source. These numbers should be described as code choices.
- `src/core.rs` attributes `contact_gain` (0.02) to Pettigrew & Tropp 2006. That paper gives no numeric value, so the same correction applies.
- The contest steepness (4.0) and the respect amounts (0.1, −0.05, 0.05) have no source paper. The steepness is cited to Bonabeau 1995 and Hemelrijk 1999 in `src/core.rs`; neither gives this value.
- The planned OwnShare ratio is presented in the plan as following Schelling 1971. The paper's threshold model is qualitative; the ratio is a plan formula, not a published one.

## Sources

Corpus (home-still): no matching papers. The searches above returned none that bear on this note. The autism social-identity paper was read only as background and is not cited as evidence.

Cited from metadata only (not in the corpus, so they do not count toward any corpus minimum):

- Tajfel et al. 1971, https://doi.org/10.1002/ejsp.2420010202
- Pettigrew & Tropp 2006, https://doi.org/10.1037/0022-3514.90.5.751
- Magee & Galinsky 2008, https://doi.org/10.5465/19416520802211628
- Berger, Cohen & Zelditch 1972, https://doi.org/10.2307/2093465
- Bonabeau, Theraulaz & Deneubourg 1995, https://doi.org/10.1016/0378-4371(95)00064-e
- Hemelrijk 1999, https://doi.org/10.1098/rspb.1999.0646
- Schelling 1971, https://doi.org/10.1080/0022250X.1971.9989794
- Axelrod 1986, https://doi.org/10.2307/1960858
- Centola & Baronchelli 2015, https://doi.org/10.1073/pnas.1418838112
- Young 1993, https://doi.org/10.2307/2951778

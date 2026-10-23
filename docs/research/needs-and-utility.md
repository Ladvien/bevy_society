# Needs and utility

## What bevy_society does

Every person carries a list of needs. Each need is a number from 0 (desperate) to 1 (satisfied), and it drains over game-days at a rate the definitions set. When a person is free, it scores every option it could take: an activity paired with a target, such as eating at the tavern, chatting with a neighbour, or wandering. Each option's score is the product of its considerations, and each consideration is a response curve over one input, such as a need level, the hour of day, or an affinity. The person picks one option at random, weighted by score, from the highest dual-utility rank that has any viable option. If nothing scores above zero, it idles for one game-hour.

The scoring system is the planned design (plan D7). On origin/main, `src/behavior.rs` holds only the data types (`Doing`, `DoingTarget`, `Destination`), and `src/model/curves.rs` holds the curve math. The equations below are the plan's specification and should be checked against the code once D7 lands.

## Equations as implemented

Need decay (plan D3, `Needs`): `level ← max(0, level − decay_per_day · dt_days)`. Default decay rates come from the starter pack: `social` 1.0/day, `rest` 0.7/day, `food` 0.9/day, `fun` 0.8/day. These rates are authored content, not paper values. Maslow (1943) and Baumeister and Leary (1995) motivate the idea that unmet needs drive behaviour, but neither gives a decay rate. Maslow: DOI https://doi.org/10.1037/h0054346 (metadata only, not converted). Baumeister and Leary: DOI https://doi.org/10.1037/0033-2909.117.3.497 (metadata only, not converted).

Response curves (`src/model/curves.rs`). Each input x is first normalised to [0,1], then mapped by one of these curves:

- `Linear(m, b)`: clamp(m·x + b, 0, 1)
- `Power(k)`: clamp(x, 0, 1)^k
- `Logistic(k, x0)`: 1 / (1 + e^(−k(x − x0)))
- `Step(at)`: 1 if x ≥ at, else 0
- `Hours(from, to)`: 1 inside the hour window [from, to) in game hours, wrapping past midnight when from > to

Designers author these per consideration, so "food matters a lot below 0.35" is written `Logistic(−10, 0.35)`, as in the starter `eat` activity.

Option score (plan D7). For an option with n considerations, each value s ∈ [0,1]:

- Score = weight · Π sᵢ

The ch13 text supports the plain product: "These scores are then multiplied together to obtain the overall score of the DSE itself," and zero-scoring considerations stop the evaluation early (Mark and Dill, corpus doc `gameaipro3-ch13-choosing-effective-utility-based-considerations`).

Dual-utility selection (plan D7, Dill 2015, DOI https://doi.org/10.1201/b18373-5):

1. Drop options whose weight is ≤ 0.
2. Keep the highest `rank` that still has an option with score > 0; drop lower ranks.
3. Within that rank, drop options scoring below `tier_cutoff` × the best score. `tier_cutoff` defaults to 0.25 (`src/core.rs`).
4. Pick one option at random, weighted by score, using one `unit()` draw from `SocietyRng`. The random number is reduced by each option's weight until it reaches zero.

Steps 1, 2 and 4 follow Dill's description (corpus doc `gameaipro2-ch03-dual-utility-reasoning`, DOI 10.1201/b18373-9 per the candidate pool). Dill describes the cutoff as a percentage "much worse" than the best, configurable per decision. The value 0.25 is the plan's default, not a value from the paper.

Fallback: no option with score > 0 gives activity `idle`, rank 0, for one game-hour.

## Why these defaults

The product-of-considerations form and the rank-then-weight selection come from Dill's dual-utility work and from the utility-AI chapters of Game AI Pro (corpus docs `gameaipro2-ch03-dual-utility-reasoning`, `gameaipro3-ch13-choosing-effective-utility-based-considerations`, `gameaipro1-ch09-introduction-utility-theory`, `gameaipro1-ch10-building-utility-decisions-into-existing`). Dill's Zoo Tycoon 2 example, as read in the corpus, places needs-driven behaviour at rank 0 and higher-priority behaviour at ranks in the high 90s, which is the ordering the plan's `sleep` (rank 1) over `wander` (rank 0) follows.

The 0.25 cutoff is a plan value chosen so that a clearly worse option is not picked over a close one. It is not a paper constant. The need decay rates are authored starter values chosen for a village day of about four minutes; no paper supplies them.

Corpus evidence on numeric need rates is thin. The corpus contains Bakkes, Tan and Pisan (2012) (DOI https://doi.org/10.1145/2336727.2336731, corpus doc `10.1145_2336727.2336731`) and an intrinsic-motivation typology (DOI https://doi.org/10.3389/neuro.12.006.2007, corpus doc `10.3389_neuro.12.006.2007`). Neither has been read for this note, so the plan does not cite them for rates.

## What the model leaves out

- Need interactions. Each need decays independently; the model has no coupling such as low rest lowering mood. Mood is handled separately in D3.
- Compensation for many considerations. The product lets one low consideration veto an option. The plan does not soften that veto.
- Mid-activity re-planning. A person keeps its activity until it ends, so a sudden need spike does not interrupt it.
- Learning. Considerations and curves are fixed; weights are not adapted from outcomes.
- Interruption by time-of-day. Durations are whole game-hours.
- Crowding beyond a place's capacity. An occupied place is excluded from the option set.

## Sources

- Dill, K. (2015), Dual-Utility Reasoning. DOI https://doi.org/10.1201/b18373-5 (metadata only, not converted). Corpus copy: `gameaipro2-ch03-dual-utility-reasoning`, DOI https://doi.org/10.1201/b18373-9.
- Mark, D. and Dill, K. (Game AI Pro 3, ch. 13), Choosing Effective Utility-Based Considerations. Corpus doc `gameaipro3-ch13-choosing-effective-utility-based-considerations`.
- Game AI Pro 1, ch. 9 (Introduction to Utility Theory), DOI https://doi.org/10.1201/b16725-15, corpus doc `gameaipro1-ch09-introduction-utility-theory`.
- Game AI Pro 1, ch. 10 (Building Utility Decisions into Existing Systems), DOI https://doi.org/10.1201/b16725-16, corpus doc `gameaipro1-ch10-building-utility-decisions-into-existing`.
- Maslow, A. H. (1943), A Theory of Human Motivation. DOI https://doi.org/10.1037/h0054346 (metadata only, not converted).
- Baumeister, R. F. and Leary, M. R. (1995), The Need to Belong. DOI https://doi.org/10.1037/0033-2909.117.3.497 (metadata only, not converted).

## Discrepancies

- Compensation (plan D7). The plan specifies s′ = s + (1 − s)·(1 − 1/n)·s with score = weight · Π s′. None of the four Game AI Pro texts in the corpus contains a compensation formula. Their text gives the plain product ("These scores are then multiplied together", `gameaipro3-ch13-choosing-effective-utility-based-considerations`). The compensated form is therefore unverified against the sources. Decision needed: keep the plan's form and cite it as a plan addition, or implement the plain product the book describes.
- Dual-utility cutoff (plan D7, `src/core.rs:176`, `tier_cutoff = 0.25`). Dill's text leaves the percentage configurable per decision. The value is a plan default, not a paper constant.
- Recency basis (plan D3, cited here for reference). The plan computes recency from hours since creation. Park et al. (2023) measure it from the last retrieval, with decay factor 0.995 per sandbox hour (corpus doc `10.48550_arxiv.2304.03442`, DOI https://doi.org/10.48550/arXiv.2304.03442). The plan wording needs confirming before the memory recall code is written.

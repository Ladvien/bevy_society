# Research: absence has a social weight (issue #35)

**Item.** Issue #35, the oldest open enhancement (#24 and #27 are release issues, done by this pass's release). No bug issue was open; parked branches `wiggum/parked-pass-{20,28,36,37}` are for older items and were not adopted.

**Sources.**
- Hazan & Shaver 1987, *Romantic love conceptualized as an attachment process*, DOI `10.1037/0022-3514.52.3.511` (resolved through doi.org). Mechanism only: proximity-seeking and separation distress are what an attachment bond is, so the people closest to someone feel their absence in proportion to the closeness and the feeling ends at the reunion. The paper gives no rate. The home-still corpus was unavailable on 2026-10-29 (`abstract_search` answered `gpu busy`, twice), so the paper is not yet in it.
- Prigerson et al. 2009, DOI `10.1371/journal.pmed.1000121`: already the crate's source for grief proportional to closeness; reused for the definition of "close" (`Friend`, or `familiarity_peak >= bereavement_familiarity`) and for closeness `0.5 (1 + affinity)`.

**Formula.** `level = missing_cap · closeness · (1 − e^(−days_away / missing_days))`, `Distress` toward the away person raised to `level` every society tick; reunion `Joy = reunion_fraction · level` at the return.

**Parameters.** None reported by the papers; all three are design defaults, labelled so on `Tuning`: `missing_cap` 0.4 (below a close friend's death at 0.6), `missing_days` 2.0, `reunion_fraction` 0.75.

**Believable output.** A close friend's distress climbs over the days and flattens, her mood word drops; a stranger's does not move; at the return a short joy and the distress gone within a day; a party away together misses nobody; a death while away is grief, not a missing.

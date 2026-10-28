# 2026-10-28-11 — research

Item: the last pass's top remaining item, break the retaliation ping-pong (73% of snubs at 24 people answer an insult the snubber received, `runs/2026-10-28-10/eval.md`). Open issues #24, #27 are release issues (skipped); #35 is an enhancement, ranked below this. This item was adopted from the parked tree of pass 36 (`39ce1a1`), which had the grudge-half-life change written but never built or gated.

## Sources

- McCullough, Fincham & Tsang 2003, *Forgiveness, forbearance, and time*, DOI 10.1037/0022-3514.84.3.540. Transgression-related avoidance and revenge motivation fall with time ("trend forgiveness"); forbearance is how much of it is never felt. Both are measured over weeks, in people recalling real offences.
- McCullough, Bono & Root 2007, *Rumination, emotion, and forgiveness*, DOI 10.1037/0022-3514.92.3.490. Rumination about a transgression holds forgiveness back, through anger; less rumination, faster cooling.
- Bondü & Richter 2016, DOI 10.3389/fpsyg.2016.00795 (already cited): the mechanism behind `snub`.

## Model

`Memories::grudge` was `max over memories of max(0, -valence) * recency(h)` with `recency = 0.995^h` (half-life 138 h, Park et al. 2023, DOI 10.48550/arXiv.2304.03442, which is a *retrieval* recency, not a forgiveness curve). A received insult therefore kept its weight for about a week and re-fed every contact. New: the grudge decays as `0.5^(h / grudge_half_life_hours)`, its own `Tuning` field; memory retrieval is unchanged. Neither paper reports a half-life in a game's hours, so the number is a **design default chosen by sweep** (12 / 24 / 48 / 138 h).

## Believable output

A quarrel flares and ends within a day or two; rivalry persists only where the offences keep coming. Friends stay as many as before, rivals stop being a third of a 60-person town, insults per person-day fall.

# Research: forgiveness that follows trust

## The problem, measured

`ctx.grudge(a, b)` is the worst of `a`'s bad memories about `b`, each times `recency(h) = 0.995^h` (half-life 138 h, Park et al. 2023, DOI 10.48550/arXiv.2304.03442). Every grudge fades on that one curve, whoever it is held against. The starter pack's `snub` (interaction `insult`, harm -0.4) has the grudge as its only consideration, so the victim answers, and the answer gives the first insulter a grievance of its own. At HEAD (`runs/2026-10-30-24/evidence/head/*/grudge_sources.txt`) **75% of snubs rest on an insult the snubber received and accepted**; the 60-person town ends a 30-day run at **277 rival pairs against 49 friend pairs** (`runs/2026-10-30-24/eval.md`). `needs-decision/what-does-the-village-owe-its-grudges.md` found that the one global lever (`affinity_relax_per_day`) cannot tell a grudge from a fondness, and that option 3 (grudges fading on their own terms) lacked a paper. This note supplies one.

## Sources

- **McCullough, Root Luna, Berry, Tabak & Bono 2010.** On the form and function of forgiving: modeling the time–forgiveness relationship and testing the valuable relationships hypothesis. *Emotion*. DOI `10.1037/a0019349`. Not open access; read from its abstract (paper_search record, crossref + europe_pmc + openalex). Forgiveness rises with time since the offence on a logarithmic curve ("by 3 months ... forgiveness had increased by two log-odds units"); "individual differences in rates of change were correlated with robust predictors of forgiveness"; and "forgiveness was uniquely associated with participants' perceptions that their relationships with their offenders retained value."
- **Burnette, McCullough, Van Tongeren & Davis 2012.** Forgiveness results from integrating information about relationship value and exploitation risk. *Personality and Social Psychology Bulletin*. DOI `10.1177/0146167211424582` (verified, paper_get). Two studies, one correlational and one experimental: "the combination of low exploitation risk and high relationship value predicted the greatest forgiveness."
- **Strelan, Karremans & Krieg 2017.** What determines forgiveness in close relationships? The role of post-transgression trust. *British Journal of Social Psychology* (online 2016). DOI `10.1111/bjso.12173` (paper_search record). Three studies (N = 108, 71, 184): the closeness → forgiveness link "may be explained by levels of post-transgression trust in the offender", and "trust always played the main mediating role ... even when taking into account ... apology".
- Context, not used for a number: Forster et al. 2021 (DOI `10.1038/s41598-021-92373-y`): apologies promote forgiveness by communicating relationship value. In this crate a conciliatory act is an accepted `chat` or `gossip` (trust +0.05 / +0.03), so it reaches forgiveness through trust without a new mechanism.

## The model

Relationship value and exploitation risk, read together, are what this crate's directed **trust** already is: it rises with every good exchange and falls by `witness_gain · harm` with every offence the holder suffers (`patient_reaction`, `src/reputation.rs`). Strelan et al. make trust the mediator outright. So the pace at which a grievance fades is

```text
pace(a → b)  = 1 + forgiveness · trust(a → b)        trust in [0, 1]; 0.5 when a holds no tie to b
ctx.grudge   = clamp( max over a's memories m about b of  max(0, −valence_m) · 0.995^(hours_m · pace) , 0, 1)
```

`forgiveness = 0` is pace 1: the existing curve, bit for bit (`x · 1.0 == x` in IEEE arithmetic). The pace multiplies the age the existing exponential reads, so a grievance at pace 2 has the weight after three days that it had after six. McCullough et al.'s curve is logarithmic over months; this crate keeps Park's exponential (the recency every memory already uses) and changes only its **rate per relationship**, which is the part of the finding that is about value. The form of the curve is left alone on purpose: changing it would move every grudge in every host, trusted or not.

## Parameters

- The papers give direction (more value, less risk → more forgiveness) and mediator (trust), no rate in game-days. `Tuning::forgiveness` is a **design default**, chosen by measurement on the village and town scenarios.
- Trust is read **now** (post-transgression, Strelan et al.), not frozen at the offence. A new offence lowers trust, so it also slows the fading of older grievances toward the same person: a serial offender is a high exploitation risk (Burnette et al.).

## What believable output looks like

- A one-off slight from someone the victim trusts (a friend, trust > 0.5) is answered at most once and then fades within a few game-days; the pair stays friends.
- A stranger who keeps insulting loses trust and is resented on the old 138-hour curve: feuds between people who have given each other reason still form and persist.
- Fewer insult replies (the 75% share falls), fewer rival pairs, more friends, compliments-to-insults closer to Baumeister et al. 2001's 5:1 floor (DOI 10.1037/1089-2680.5.4.323), and no saturation: rivals do not reach zero.

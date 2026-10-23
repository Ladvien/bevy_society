# Groups and status

## What bevy_society does

Factions are defined in the defs file. A person joins a faction through a `Memberships` component. Each faction has an in-group bias (`FactionDef.ingroup_bias`), which raises affinity and exposure between members. When two people contest something, the winner is chosen by a logistic rule over their status, and the winner's respect rises a little with each witness. Each person's status is the respect others hold toward them, so status is the contest outcome carried forward. Intergroup contact is a separate, weaker path. The model functions are on HEAD (`src/groups.rs`, `src/defs.rs`). The contest is wired into interactions in D7; the `resolve_contest` signature still takes a `witnesses` argument on HEAD, and D7 changes it.

## Equations as implemented

- Contest winner (`src/groups.rs:136`, `resolve_contest`): the actor wins with probability `p = 1 / (1 + exp(−steepness · (s_actor − s_target)))`, where `s` is each contestant's status and `steepness` = `Tuning.contest_steepness` = 4.0. Design default (no source paper gives this value); mechanism: Dill (2015) for the utility curve shape.
- Contest deltas (`src/groups.rs`): the winner's respect rises by 0.1 and the loser's falls by 0.05. Each witness's respect for the winner rises by 0.05 (`witness_contest`, specified for D7). These three values are design defaults.
- In-group bias (`src/groups.rs:63–74`): affinity and exposure toward a faction member are scaled by `FactionDef.ingroup_bias`.
- Shared faction bias: when two people share a faction, their exposure gets the same bias before acceptance (D7 step 2).
- Group size: `Tuning.tier_cutoff` = 0.25 (`src/core.rs`) is a design default (Dill 2015, mechanism: utility tiers).

## Why these defaults

- **Support cliques and group size.** Zhou, Sornette, Hill & Dunbar (2005) report a support clique of about 3.3 people and an equivalent group size of about 150 (`10.1098_rspb.2004.2970`, converted; passage checked). These numbers scale the faction sizes used in examples. The contest parameters are design defaults (no source paper gives these numbers).
- **Dominance.** Hemelrijk (1999) compares despotic and egalitarian dominance structures (`10.1098_rspb.1999.0646`, converted; passage checked). This supports letting contest outcomes depend on status without fixing a single hierarchy shape.
- **Large groups.** Poor (2019) describes guilds whose leaders manage "groups of 400 to 500 people" (`10.1145_3337722.3337760`, converted; checked for the group-size figure only). The crate does not model groups that large; this is context only.
- **Socially intelligent agents.** Oliveira (2019) studies merchants that interact with customers in a shared social setting (`10.1145_3337722.3337729`, converted; checked). This motivates per-person social status in a shared setting.
- **Intergroup mechanisms.** Tajfel (1971) and Bonabeau et al. (1995, DOI `10.1016/0378-4371(95)00064-e`) are the sources for in-group bias and hierarchy formation. Neither is converted; cited from DOI only. Bonabeau is the mechanism for the contest's steepness, not a numeric source.

## What the model leaves out

- Explicit group formation and dissolution: membership is set by the defs file and by script actions (`join`, `leave`); nothing forms a group on its own.
- Roles, offices and leadership within a faction.
- Status transmitted through institutions rather than through witnessed contests.
- Intergroup conflict at group scale: contests are one-to-one.
- Cultural transmission of faction norms.
- Empirical calibration of the contest curve against real hierarchies.

## Open contradictions and departures

- **Design defaults:** contest steepness, contest deltas, witness gain and the in-group bias default. Not fitted to a source.
- **Departure from the note's earlier text:** the earlier note assumed the contest used a witness list in its signature. The HEAD signature still takes `witnesses`; D7 drops it. The current note describes the D7 behaviour.
- **No contradiction found** with converted sources. Zhou (2005) and Hemelrijk (1999) support the scale and dominance mechanisms; neither fixes the contest numbers.

## Sources

- Zhou et al. (2005). https://doi.org/10.1098/rspb.2004.2970 · `10.1098_rspb.2004.2970`
- Hemelrijk, C. K. (1999). https://doi.org/10.1098/rspb.1999.0646 · `10.1098_rspb.1999.0646`
- Poor (2019). https://doi.org/10.1145/3337722.3337760 · `10.1145_3337722.3337760` (group-size figure only)
- Oliveira (2019). https://doi.org/10.1145/3337722.3337729 · `10.1145_3337722.3337729`
- Bonabeau et al. (1995). https://doi.org/10.1016/0378-4371(95)00064-e · not converted
- Tajfel (1971). Intergroup categorisation (DOI not checked against the text). · not converted
- Dill (2015). https://doi.org/10.1201/b18373-5 · not converted

# Changelog

All notable changes to bevy_society. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the crate is pre-1.0, so any release may break the API.

## [Unreleased]

### Added

- **Script runtime and rules (`RulesPlugin`).** The `ctx` / `fx` Rhai API over a per-tick snapshot: `ctx` reads the society as it stood at the tick start and `fx` queues writes that one exclusive system applies after `SocietySystems::Rules`. Triggers fire on `Tick`, `Hour`, `Custom` (`FireTrigger`), `TieLabelChanged`, `NormViolated`, `NeedBelow` and `RumorReach`, honouring `once` and `cooldown_hours`; a failed condition counts as false and a failed effect applies none of its actions. Every evaluation is sandboxed (10 000 operations, bounded strings, arrays and maps).

- **Beliefs, rumors and reputation (`BeliefsPlugin`).** Opinions per topic with bounded-confidence influence, Friedkin–Johnsen anchor pull and conformity to a strong group mean; a rumor book with the Daley–Kendall stifling rule and trust-scaled belief; and the witness, patient and actor reactions to a deed, including `NormViolated`. `PlantRumor` starts a rumor from the game or a script. See `docs/research/opinion-dynamics.md` and `docs/research/rumor-and-reputation.md` (Deffuant et al. 2000, DOI 10.1142/S0219525900000078; Daley & Kendall 1964, DOI 10.1038/2041118a0; Ohtsuki & Iwasa 2004, DOI 10.1016/j.jtbi.2004.06.005).

- **Engine-free model math (`bevy_society::model`).** Response curves, the Mehrabian/ALMA personality-to-mood baseline, power-law familiarity and exposure, Deffuant opinion influence with anchor pull and conformity, Daley–Kendall stifling, and triadic balance. Each formula's DOI sits in a comment beside it.

- **Relationships (`RelationshipsPlugin`).** Directed, weighted ties per person with power-law familiarity (Anderson & Schooler 1991), mere exposure (Zajonc 1968), affinity relaxation toward zero, Heider structural balance over sampled triads, a Dunbar-style cap on ties, and labels (stranger, acquaintance, friend, rival) with a `TieLabelChanged` message. See `docs/research/relationships.md` (Heider 1946, DOI 10.1080/00223980.1946.9917275; Dunbar 1992, DOI 10.1016/0047-2484(92)90081-J).

- **Groups and status (`GroupsPlugin`).** Faction memberships with an in-group bias, standing as respect conferred by familiar persons, logistic contests weighted by dominance and standing, intergroup contact (Pettigrew & Tropp 2006), and per-faction cohesion and regard each tick. See `docs/research/groups-and-status.md` (Tajfel et al. 1971, DOI 10.1002/ejsp.2420010202; Bonabeau et al. 1995, DOI 10.1016/0378-4371(95)00064-e).

- **Social fields (`fields` feature).** Per-cell crowd presence and commotion grids plus one presence grid per faction, deposited in person order and evaporated per game-day on `bevy_stigmergy`. Schelling's tolerance input is the `OwnShare` field (own / (own + other)), and `Gradient` activity targets follow the field toward or away from a cluster. See `docs/research/groups-and-status.md` (Schelling 1971, DOI 10.1080/0022250X.1971.9989794).

- **Mind (`MindPlugin`).** Emotions (OCC kinds with ALMA PAD values) are appraised by `Appraise` messages and decay with a 0.1 game-day half-life. Mood is pulled toward the intensity-weighted emotion centre and returns to a personality baseline (Mehrabian 1996; Gebhard 2005), with Hatfield-style contagion from affinity-weighted contacts. Needs decay at `decay_per_day`. See `docs/research/personality-and-mood.md` (DOI 10.1080/00049539608259510).

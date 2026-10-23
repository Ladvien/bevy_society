# Changelog

All notable changes to bevy_society. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the crate is pre-1.0, so any release may break the API.

## [Unreleased]

### Added

- **Engine-free model math (`bevy_society::model`).** Response curves, the Mehrabian/ALMA personality-to-mood baseline, power-law familiarity and exposure, Deffuant opinion influence with anchor pull and conformity, Daley–Kendall stifling, and triadic balance. Each formula's DOI sits in a comment beside it.

- **Relationships (`RelationshipsPlugin`).** Directed, weighted ties per person with power-law familiarity (Anderson & Schooler 1991), mere exposure (Zajonc 1968), affinity relaxation toward zero, Heider structural balance over sampled triads, a Dunbar-style cap on ties, and labels (stranger, acquaintance, friend, rival) with a `TieLabelChanged` message. See `docs/research/relationships.md` (Heider 1946, DOI 10.1080/00223980.1946.9917275; Dunbar 1992, DOI 10.1016/0047-2484(92)90081-J).

- **Groups and status (`GroupsPlugin`).** Faction memberships with an in-group bias, standing as respect conferred by familiar persons, logistic contests weighted by dominance and standing, intergroup contact (Pettigrew & Tropp 2006), and per-faction cohesion and regard each tick. See `docs/research/groups-and-status.md` (Tajfel et al. 1971, DOI 10.1002/ejsp.2420010202; Bonabeau et al. 1995, DOI 10.1016/0378-4371(95)00064-e).

- **Social fields (`fields` feature).** Per-cell crowd presence and commotion grids plus one presence grid per faction, deposited in person order and evaporated per game-day on `bevy_stigmergy`. Schelling's tolerance input is the `OwnShare` field (own / (own + other)), and `Gradient` activity targets follow the field toward or away from a cluster. See `docs/research/groups-and-status.md` (Schelling 1971, DOI 10.1080/0022250X.1971.9989794).

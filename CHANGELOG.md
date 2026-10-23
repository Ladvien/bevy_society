# Changelog

All notable changes to bevy_society. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the crate is pre-1.0, so any release may break the API.

## [Unreleased]

### Added

- **Engine-free model math (`bevy_society::model`).** Response curves, the Mehrabian/ALMA personality-to-mood baseline, power-law familiarity and exposure, Deffuant opinion influence with anchor pull and conformity, Daley–Kendall stifling, and triadic balance. Each formula's DOI sits in a comment beside it.

- **Social fields (`fields` feature).** Per-cell crowd presence and commotion grids plus one presence grid per faction, deposited in person order and evaporated per game-day on `bevy_stigmergy`. Schelling's tolerance input is the `OwnShare` field (own / (own + other)), and `Gradient` activity targets follow the field toward or away from a cluster. See `docs/research/groups-and-status.md` (Schelling 1971, DOI 10.1080/0022250X.1971.9989794).

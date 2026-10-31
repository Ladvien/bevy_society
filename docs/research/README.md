# Research index

This table maps each source module to its plugin, the papers behind it, and the note that explains it. Each note states which constants are design defaults and which are taken from a source. For corpus status (which papers are converted and embedded), see [corpus/README.md](corpus/README.md).

| Module | Plugin | Papers (DOI) | Note |
|---|---|---|---|
| `src/mind.rs`, `src/model/pad.rs` | `MindPlugin` | Gebhard 2005 (`10.1145/1082473.1082478`); Mehrabian 1996 (`10.1080/00049539608259510`, unverified); Gratch & Marsella 2004 (`10.1016/j.cogsys.2004.02.002`); Dias & Paiva 2005 (`10.1007/11595014_13`); Hatfield 1993 (`10.1111/1467-8721.ep10770953`); van Kleef & Côté 2022 (`10.1146/annurev-psych-020821-010855`) | [personality-and-mood.md](personality-and-mood.md) |
| `Chronotype` in `src/mind.rs`, `Input::BodyHour`, `ActivityDef::unavailable` | `MindPlugin`, `BehaviorPlugin` | Roenneberg et al. 2007 (`10.1016/j.smrv.2007.07.005`); Fischer et al. 2017 (`10.1371/journal.pone.0178782`); Reiter et al. 2021 (`10.3390/clockssleep3020021`); Sogawa et al. 2023 (`10.5334/jcr.225`); Wittmann et al. 2006 (`10.1080/07420520500545979`) | [chronotype.md](chronotype.md) |
| `src/model/recall.rs`, `Memories` in `src/mind.rs` | `MindPlugin` | Park et al. 2023 (`10.48550/arXiv.2304.03442`); Murre & Dros 2015 (`10.1371/journal.pone.0120644`); Anderson & Schooler 1991 (`10.1111/j.1467-9280.1991.tb00174.x`) | [memory.md](memory.md) |
| `src/model/curves.rs`, `src/behavior.rs` | `BehaviorPlugin` | Mark 2013; Dill 2015 (`10.1201/b18373-5`); Game AI Pro 3 ch. 13 | [needs-and-utility.md](needs-and-utility.md) |
| `src/model/opinion.rs`, `src/beliefs.rs` (opinion parts) | `BeliefsPlugin` | Deffuant et al. 2000 (`10.1142/S0219525900000078`); Flache et al. 2017 (`10.18564/jasss.3521`); Castellano et al. 2009 (`10.1103/RevModPhys.81.591`); Hegselmann & Krause 2002 (not converted) | [opinion-dynamics.md](opinion-dynamics.md) |
| `src/absence.rs` | `MindPlugin` | Hazan & Shaver 1987 (`10.1037/0022-3514.52.3.511`); Prigerson et al. 2009 (`10.1371/journal.pmed.1000121`) | [relationships.md](relationships.md#absence-missing-someone-and-the-return-issue-35) |
| `src/relationships.rs`, `src/model/balance.rs` | `RelationshipsPlugin` | Zhou et al. 2005 (`10.1098/rspb.2004.2970`); Sekara et al. 2016 (`10.1073/pnas.1602803113`); Gillespie et al. 2015 (`10.1371/journal.pone.0128900`); Zajonc 1968 (`10.1037/h0025848`); Heider 1946 (`10.1080/00223980.1946.9917275`); Cartwright & Harary 1956 (`10.1037/h0046049`) | [relationships.md](relationships.md) |
| `Tuning::rebuff_memory_hours`, `Rebuffed` in `src/behavior.rs` | `BehaviorPlugin` | Maner et al. 2007 (`10.1037/0022-3514.92.1.42`); Downey & Feldman 1996 (`10.1037/0022-3514.70.6.1327`); Bondü & Richter 2016 (`10.3389/fpsyg.2016.00795`); Stenseng et al. 2014 (`10.1177/0146167214554591`) | [rebuff.md](rebuff.md) |
| `src/model/rumor_dk.rs`, `src/rumor.rs`, `src/reputation.rs` | `BeliefsPlugin` | Daley & Kendall 1964 (`10.1038/2041118a0`); Moreno et al. 2004 (`10.1103/PhysRevE.69.066130`); Sommerfeld et al. 2007 (`10.1073/pnas.0704598104`); Zubiaga et al. 2017 (`10.48550/arXiv.1704.00656`) | [rumor-and-reputation.md](rumor-and-reputation.md) |
| `src/economy.rs`, `skill_level` in `src/core.rs`, `Input::Skill`, `Input::Surplus`, `actor_gives` / `target_gives` | `EconomyPlugin`, `BehaviorPlugin` | Acemoglu & Autor 2010 (`10.3386/w16082`); Henrich et al. 2005 (`10.1017/S0140525X05000142`); Becker 1965 (via `10.1007/s11150-005-4941-1`); Durkheim 1964 (via `10.29173_cjs6661`); Parunak et al. (`10.1007_11678809_10`); Hall 2005, reporting Hall & Milgrom (`10.3386/w11678`); the production/consumption economy (`eth-cgl-ar_games-Zun16a`) | [economics.md](economics.md) |
| `src/groups.rs` | `GroupsPlugin` | Zhou et al. 2005 (`10.1098/rspb.2004.2970`); Hemelrijk 1999 (`10.1098/rspb.1999.0646`); Bonabeau et al. 1995 (`10.1016/0378-4371(95)00064-e`); Tajfel 1971 | [groups-and-status.md](groups-and-status.md) |
| Whole crate (game-facing design) | all plugins | Park et al. 2023; Oliveira 2019 (`10.1145/3337722.3337729`); Oz-centric believable agents, 1999 (`10.1007/3-540-48317-9_12`); Game AI Pro 1 ch. 43 | [social-simulation-games.md](social-simulation-games.md) |

## Notes

- Mehrabian 1996 PAD coefficients are unverified until the paper is in the corpus.
- Mark & Dill compensation is a deliberate design choice (`needs-and-utility.md`).
- Recency is measured from memory creation; this departs from Park et al. 2023, which measures from last retrieval (`memory.md`).
- Every constant marked "Design default (no source paper gives this number)" in `src/core.rs` is a design default, not a fitted value.

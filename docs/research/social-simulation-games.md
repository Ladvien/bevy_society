# Social simulation games

## What bevy_society does

bevy_society is a library for the social layer of a game: people with personality, mood, memory, ties, groups, beliefs, rumours and daily activities, driven by a fixed-step tick. A game defines factions, activities, interactions, triggers and dialogue in RON and Rhai scripts, then reads the outcomes from messages (`InteractionResolved`, `ScriptEvent`, `NormViolated`, `RumorReach`). The crate does not render or plan a story; it provides the simulated social state a game can react to. The model functions are on HEAD. The daily-life systems that use them are D7 work, and the example village is F1–F4 work.

## Equations as implemented

The crate's equations are those listed in the other notes: recency `0.995^h` (`memory.md`), familiarity power law (`relationships.md`), Deffuant-style opinion update (`opinion-dynamics.md`), mood pull and contagion (`personality-and-mood.md`), telling weight and stifling (`rumor-and-reputation.md`), and compensated utility (`needs-and-utility.md`). No new equation is introduced here.

- Tick cadence: each society tick is `tick_secs` = 0.25 s, which is 16 fixed steps, with `seconds_per_day` = 240 setting game time (`src/core.rs`).
- Recency: `recency(h) = 0.995^h` with `h` in game hours since creation. Departure from Park et al. (2023), who measure from last retrieval (see `memory.md`).

## Why these defaults

- **Believability as a target.** Park et al. (2023) model memory, reflection and planning for believable agents, and their recency factor is the source for the crate's recency rate (`10.48550_arxiv.2304.03442`, converted; passage checked: "we treat recency as an exponential decay function over the number of sandbox game hours since the memory was last retrieved. Our decay factor is 0.995."). The departure is recorded in `memory.md`.
- **Rich personality.** The Oz-centric believable-agents paper (1999; authors not checked from the text) defines believable agents as "autonomous agents that exhibit rich personalities" (`10.1007_3-540-48317-9_12`, converted; passage checked). The personality model (`personality-and-mood.md`) follows this aim.
- **Social believability.** Oliveira (2019) studies socially intelligent merchants that interact with customers (`10.1145_3337722.3337729`, converted; checked for believability). This motivates the interaction and gossip systems.
- **The gap this crate targets.** Game AI Pro 1 ch. 43 states: "More progress has been made creating NPCs that engage in purely autonomous activity or small-group behavior ... than creating NPCs that participate in ongoing social activity" (`gameaipro1-ch43-architecture-character-rich-social-simulation`, converted; passage checked). That is the gap in ongoing social activity the crate addresses.
- **Design defaults.** Tick, day length, `close_ties` = 15 and `bark_chance` = 0.35 are design defaults (no source paper gives these numbers).

## What the model leaves out

- Planning and long-horizon goals; behaviour is local utility scoring each tick.
- Natural-language generation: barks and dialogue are authored lines chosen from lists and RON nodes, not generated text.
- Procedural world state beyond places, factions and activities defined in the file.
- Player modelling and adaptive difficulty.
- Calibration against human play data. The example village is a demonstration, not a validated model.

## Open contradictions and departures

- **Departure, documented:** recency from creation, not last retrieval (Park 2023). Recorded in `memory.md` and in code.
- **Design defaults:** tick rate, day length, close-tie count and bark chance.
- **No contradiction found** with converted sources. The believability sources support the aim, not the numbers.

## Sources

- Park, J. S. et al. (2023). Generative Agents. https://doi.org/10.48550/arXiv.2304.03442 · `10.48550_arxiv.2304.03442`
- Oz-centric believable agents (1999; authors not checked from the text). https://doi.org/10.1007/3-540-48317-9_12 · `10.1007_3-540-48317-9_12`
- Oliveira, R. (2019). Socially intelligent merchants. https://doi.org/10.1145/3337722.3337729 · `10.1145_3337722.3337729`
- Game AI Pro, vol. 1, ch. 43, Architecture for character-rich social simulation · `gameaipro1-ch43-architecture-character-rich-social-simulation` (converted)

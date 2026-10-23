# Social simulation games: exchanges, triggers and dialogue

This note covers the social side of bevy_society: how two persons resolve a social exchange, how Rhai triggers react to what happened, and how dialogue plugs in through `bevy_speech_bubbles`. The design draws on the social-simulation game literature (McCoy et al.'s Comme il Faut and Prom Week, Evans and Short's Versu, the Talk of the Town and Ensemble systems) and on Park et al. 2023 for the memory side. Only the Park et al. 2023 text and the Game AI Pro and Oliveira and Santos texts in the corpus were read. Other papers are cited from metadata only and marked as such.

## What bevy_society does

A villager does not act on a single rule. When an activity such as `chat` or `gossip` reaches its target, the game resolves one social exchange between the actor and the target. A script rule can refuse the exchange. Otherwise it is accepted with a probability that depends on how the two people feel about each other. An accepted exchange can move opinions, spread a rumour, decide a contest, or start a bark. Every exchange writes memories for both sides. Scripts see a snapshot of the society taken at the start of the tick and act only through queued effects. A dialogue conversation is started with `TalkTo`, and its nodes can carry effects written in the defs file (plan D7, D8, D11).

## Equations as implemented

Acceptance probability (plan D7, `AcceptDef`), with the social-exchange frame from McCoy et al. 2014 (DOI https://doi.org/10.1109/tciaig.2014.2304692, metadata only):

p = clamp(base + affinity·aff(t→a) + trust·(trust(t→a) − 0.5) + agreeableness·(A_t − 0.5) + pleasure·P_t + ingroup·[shared faction], 0.02, 0.98)

Defaults: base 0.5, affinity 0.3, trust 0.2, agreeableness 0.2, pleasure 0.1, ingroup 0.1. One `unit()` draw decides the exchange.

Rejection (plan D7): aff(a→t) −= 0.05, and the actor appraises Distress with intensity 0.3.

Memory valence for each side of an exchange (plan D7): ±(0.3 + |deed harm|), with importance taken from the interaction definition.

Recency for memory retrieval (plan D3, `Memories::recall`): r = 0.995^(hours), matching the decay factor in Park et al. 2023 (DOI https://doi.org/10.48550/arXiv.2304.03442, corpus `10.48550_arxiv.2304.03442`). The plan's `hours` is the time since the memory was created. Park et al. measure it from the last retrieval. See Discrepancies.

Bark rate (plan D7): each exchange writes a bark with probability `bark_chance` = 0.35 (`SocietySettings`). Barks come from templates with `{self}`, `{target}`, `{subject}` and `{place}` placeholders.

Trigger rate limits (plan D8): `once` fires once per (trigger, actor). `cooldown_hours` limits a trigger to once per game-hour per (trigger, actor).

Script limits (plan D2, D8): `set_max_operations(10_000)`, `set_max_call_levels(16)`, `set_max_string_size(1024)`, `set_max_array_size(256)`, `set_max_map_size(64)`.

Dialogue (plan D11): `TalkTo { npc, player }` picks the first conversation rule, ordered by priority descending and then definition order, whose condition holds. A node effect runs when the conversation enters the named node. A finished conversation writes `MemoryKind::Talked` for both sides with valence 0.1 and importance 0.4.

## Why these defaults

The exchange frame follows the social-practice idea in McCoy et al.'s Comme il Faut and Prom Week work and in Evans and Short's Versu. The corpus holds the Game AI Pro 1 chapter `gameaipro1-ch43-architecture-character-rich-social-simulation` (2012, no DOI in the index), which covers social-simulation architecture. Its text was not read closely enough to cite a specific mechanism. The numeric defaults in the acceptance formula are plan values chosen by hand. No corpus paper was found that fixes them, so they are tuning choices, not evidence.

The recency rate comes from Park et al. 2023. The corpus text gives an exponential decay over sandbox game hours with a factor of 0.995. It combines recency, importance and relevance, each min-max scaled to [0,1] (`10.48550_arxiv.2304.03442`). The plan keeps recency and importance and replaces relevance with a match on the other person.

Oliveira and Santos 2019 (DOI https://doi.org/10.1145/3337722.3337729, corpus `10.1145_3337722.3337729`, full text available) is the corpus source for the social-game design this plan follows. The plan does not reproduce its model.

The Talk of the Town and Ensemble papers are cited from metadata only. Their corpus texts were not read, so this note makes no claim about their mechanisms.

## What the model leaves out

- Natural-language generation. Barks are authored templates. Park et al. 2023 generate dialogue from the model. bevy_society does not.
- Multi-turn negotiation. An exchange resolves in one step with one random draw.
- The full volition-summing model of Comme il Faut across every potential partner. The plan proposes one exchange per arrival.
- The relevance term of Park et al. Recall ranks by recency, importance and whether the memory is about the other person.
- Dialogue branching by affinity beyond the first matching conversation rule.
- Learned exchange outcomes. All outcome weights are authored in the defs file.

## Discrepancies

- Park et al. 2023 measure recency from the last retrieval of a memory. The plan's `Memories::recall` (D3) measures it from the memory's creation. A memory that is retrieved often decays more slowly in the paper. The plan decays it at the same rate regardless of retrieval. This is a plan choice, not a paper value. The code should either keep the plan's creation-time rule and document it, or track the last retrieval time as the paper does.
- The acceptance-formula weights (0.5, 0.3, 0.2, 0.2, 0.1, 0.1) and the rejection penalty (0.05, Distress 0.3) are plan values. The corpus does not supply them, so this note does not claim that any paper sets them.

## Sources

- Park et al. 2023, generative agents, DOI https://doi.org/10.48550/arXiv.2304.03442 (corpus `10.48550_arxiv.2304.03442`, full text).
- Oliveira and Santos 2019, DOI https://doi.org/10.1145/3337722.3337729 (corpus `10.1145_3337722.3337729`, full text).
- Game AI Pro 1 chapter 43, social simulation architecture (corpus `gameaipro1-ch43-architecture-character-rich-social-simulation`, 2012, no DOI in the index).
- McCoy et al. 2014, Comme il Faut, DOI https://doi.org/10.1109/tciaig.2014.2304692 (metadata only).
- McCoy et al. 2012, Prom Week, DOI https://doi.org/10.1145/2282338.2282384 (metadata only).
- Evans and Short 2014, Versu, DOI https://doi.org/10.1109/TCIAIG.2013.2287297 (metadata only).
- Ryan et al. 2015, Talk of the Town, DOI https://doi.org/10.1609/aiide.v11i3.12825 (metadata only).
- Kelly et al. 2024, Ensemble, DOI https://doi.org/10.1145/3649921.3659841 (metadata only).
- Jurado et al. 2019, DOI https://doi.org/10.1145/3337722.3341859 (metadata only).

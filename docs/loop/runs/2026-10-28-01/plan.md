# Plan: measure the saturation, do not damp it

## Why

The log's #1 was hostility saturation, and the previous review's zoom pass had returned NOT YET on
it: affinity pinned at −1.0 with `affinity_relax_per_day = 0.01` unable to damp it. The obvious
move was to raise relaxation or floor the deed's weight. That would have been tuning a symptom
without knowing what was driving it.

## What the measurement changed

The runaway turned out to be three things, and only one of them was in the model:

1. **The cluster scenario gave its people no places.** With only Person targets available, gossip
   held the social need high enough that `chat`'s own `Need(social) Linear(-1, 1)` gate never
   opened — measured, chat ran **zero times in 2000 ticks** — leaving gossip and snub as the only
   options. Fixed in the scenario: each group gets a hearth.
2. **`snub`'s slope saturated at grudge 0.4.** `Linear(2.5, 0)` made a single witnessed insult
   enough to keep snubbing the best option forever. Fixed to `Linear(1.0, 0)`, which still lets a
   maximum grudge dominate.
3. **The harness ran a 60x-shortened game-day while the definitions are calibrated for the
   default.** A 0.1-hour `snub` lasted 1.07 ticks, so a person re-decided every tick and repeated
   the best option — 38 snubs per person per day. The village branch also carried a `240.0` its
   own comment contradicted, which is why cross-iteration rival counts were never comparable.

## Design decision, and what the literature said

The previous review asked for damping. Before adding any, the corpus was searched for
conflict de-escalation, forgiveness and negative influence. **Takács, Flache & Mäs 2016**
(`10.1371_journal.pone.0157948`) was downloaded and converted: people do not shift their opinions
away from those they dislike; negative influence is not empirically supported. Damping is
therefore not something to add to the influence rule. What the literature locates is *who one
chooses to interact with*, which is the decision layer — and that is what the scenario fix
addresses.

No `Tuning` field was added and no rate invented.

## Touched

- `assets/starter.society.ron` (`snub` curve 2.5 → 1.0),
- `examples/evidence.rs` (cluster places, day length),
- `docs/research/rumor-and-reputation.md`, `CHANGELOG.md`, `docs/loop/log.md`,
- `docs/loop/runs/2026-10-28-01/`.

No `src/` behaviour.

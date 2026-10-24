# Research: is there a mental-health model the crate can honestly implement?

Focus passed to this iteration: **mental health**. This note records what the literature search
found and why it does not support a build, so the next iteration does not repeat it.

## What was searched

`distill_search` over the home-still corpus, four passes, metadata only:

| Query | Best score | What came back |
|---|---|---|
| `allostatic load chronic stress` | 0.500 | `10.1016_s0893-133x(99)00054-8`, `10.1016_j.tics.2011.10.005` |
| `neuroticism emotional reactivity recovery` | 0.500 | `W2131452244`, `10.1186_s40479-025-00299-y` |
| `mood baseline personality affective dynamics` | 0.500 | `10.1177_1754073915590620`, `10.3389_fpsyg.2025.1623886` |
| `burnout exhaustion recovery agent simulation` | 0.500 | `10.1186_s12888-018-1589-y`, `10.5271_sjweh.3573`, `10.1016_j.burn.2014.04.003` |
| `emotional regulation reappraisal suppression` | 0.519 | `W2133156861`, `10.31219_osf.io_8dbh4` |
| `stress coping appraisal Lazarus` | 0.532 | `10.18297_etd_2392`, `10.1097_01.nnr.0000263970.08878.87` |
| `resilience psychological recovery adversity` | 0.518 | `10.1037_sgd0000132`, `10.1017_s0954579423000536` |

Scores that peak at 0.5 with unrelated clinical papers mixed in mean the corpus has no strong match.
Compare `bounded confidence opinion dynamics`, whose top hit scored 0.54 and was the right paper.

## What the two best hits actually are

Both are converted and indexed, so both were read rather than assumed:

- **`10.1016_j.tics.2011.10.005`** (McEwen & Gianaros, *Trends in Cognitive Sciences*): a review of
  allostasis and resilience. Its contribution is conceptual — "resilience denotes the ability to
  respond to stressors by appropriate engagement and efficient *termination* of allostatic
  responses". It gives no rate, no parameter and no equation. What it does supply is a design
  principle, quoted above.
- **`10.1016_s0893-133x(99)00054-8`**: a review of the hippocampus, HPA-axis feedback and mood
  disorders. Neuroendocrinology and memory. Nothing an engine-free social model can implement.

Everything else in the top hits is clinical: burnout rehabilitation subtypes, mindfulness trials,
childhood adversity. Useful for a game about therapy; not for a utility-AI society.

## Verdict

**No build from this literature.** A `Tuning` field invented for "mental health" would have to cite a
paper that does not state its number, and this crate's rule is that a default either comes from a
paper or says outright that it is a design default. Adding a mechanism whose only support is a
review's conceptual framing would be exactly the kind of number the loop exists to prevent.

What the corpus *does* support, and what the crate already implements: mood returning to a
personality-derived baseline (Mehrabian 1996 / Gebhard 2005), emotion decay with a half-life, and
neuroticism lowering emotional stability in that baseline (`model::pad`). The allostasis quote above
names the one thing missing — **efficient termination of the response** — which the crate models as
mood return to baseline and emotion half-life, both already present.

## Consequence for this iteration

The focus cannot be served with a cited model, so this iteration does not attempt it. It records the
finding here and picks the top-ranked item from the loop's own candidate list instead, which is also
where the evidence points: the dead-code inventory names six accessors that nothing reads, and each
one is a host-facing convenience that was promised and never finished.

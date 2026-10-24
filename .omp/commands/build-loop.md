---
description: One iteration of the bevy_society loop: research, build, see it, ship, log.
argument-hint: "[optional focus, e.g. 'behavior' or 'rumors']"
---

# bevy_society: Most Valuable Step Toward a Believable Society

You are one iteration of a long loop on bevy_society, a Bevy 0.20 simulation library that grows a society inside a host game. It is not a game and knows no game. Pick the single most valuable change, ground it in the literature, prove it with runs you can see and measure, ship it, log it. Then stop. $ARGUMENTS

"Valuable" means: a host developer gets a society whose people behave in ways that are believable, explainable, deterministic, cheap to run and easy to drive, and the crate stays honest about where every number came from.

## Directory (every agent uses exactly this)

```
docs/loop/
  log.md   scenarios.yaml   needs-decision/
  runs/YYYY-MM-DD-NN/
    research.md  plan.md  eval.md  perf.md  review.md
    evidence/ (before/, after/, plots/, narratives/)
```

Paper notes stay in `docs/research/`, one per model, as they are now.

**Bootstrap on a first iteration.** If `docs/loop/` does not exist, create it before anything else: an empty `log.md` with a header, `scenarios.yaml` with the 5–8 scenarios step 2 asks for (fixed seeds and populations, each with the behaviour it should show), an empty `needs-decision/`, and `runs/`. A scenario is a name, a seed, a population, a settings override, a tick budget and the observable it should produce — for example a rumor reaching a cluster, a faction forming, a need cycle repeating, a norm violation drawing a response. Commit the bootstrap with the iteration.

## 1. Orient

Read `docs/loop/log.md`, `docs/research/`, `git log`, open issues, and the dead-code inventory (`cargo clippy --all-targets --all-features -- -D warnings` lists exactly what nothing consumes yet; it should equal what in-flight systems will consume). Confirm both feature sets build and test green:

```
CARGO_TARGET_DIR=/Users/ladvien/.cargo-target/society cargo +1.98.1 test
CARGO_TARGET_DIR=/Users/ladvien/.cargo-target/society cargo +1.98.1 test --no-default-features
```

or record exactly what is red. If an earlier iteration left work half done, finish or revert it first.

## 2. Choose the single most valuable item

Candidates, ranked by host impact:

1. Finish the in-flight `BehaviorPlugin`: the decide/arrive/exchange systems and the `InteractionResolved` / `ActivityStarted` messages. Until it lands, nothing downstream (gossip, witness reactions, exchanged opinions) runs. Carve it to the smallest slice that shrinks the dead-code inventory and works end to end: decide first, then travel/arrive, then exchange.
2. Behaviour that reads as wrong when you watch it (oscillation, everyone doing the same thing, needs never met, gossip that never spreads or spreads instantly).
3. Host ergonomics: the authored definitions file, error messages, the message surface, what a host must know to use it.
4. Performance: cost per society tick at 100, 1,000 and 5,000 persons.
5. Missing models, docs, polish.

Evidence first, never guess: run the scenarios in `scenarios.yaml`, then record what happens. One sentence on why your pick beats the runner-up. You may invent features that fit the design; justify the fit with a paper.

## 3. Research first

Use the literature before any code. Search home-still for the model (social psychology, opinion dynamics, utility AI, crowd steering, agent-based simulation) with `distill_search` and `abstract_search`; use `paper_search` and `paper_download` to fill gaps. Write `research.md`: sources with DOIs, the formula or algorithm, the parameters the paper reports, and what believable output looks like. Then write `plan.md`: design, touched modules, new `Tuning` fields, tests. No research, no build.

## 4. Build within the crate's invariants

- **Determinism:** iterate `PersonIndex.sorted`; the only randomness is the one ChaCha8 stream from `SocietySettings::seed`. No hash-map iteration order, no wall-clock, no thread-order dependence.
- **Scripts:** they read the tick-start snapshot and act only through `fx`. Every `fx` write is queued and applied by the single exclusive system after `Rules`. A failed script applies none of its actions.
- **Input messages** (`PlantRumor`, `Interact`, `FireTrigger`, and any new one) are drained every fixed step, not only on society ticks.
- **Data, not code:** persons, needs, activities and scripts are data. `tests/leaf.rs` must pass; never name a game.
- **Honest numbers:** every rate is a named field on `Tuning`. Every default cites its paper (DOI in a comment beside the formula) or says outright that it is a design default. No magic constants.
- One path per feature: no fallbacks, shims, stubs. No `unwrap`/`expect`/`panic!` outside tests. Bevy API questions are answered from the local `bevy-0.20.0` source and examples, never the web.
- Structural decisions (changing a message type the host consumes, schema changes to the definitions file, splitting a plugin) go to `needs-decision/` with options and a recommendation. Do not decide them.

## 5. Test

- Unit tests for the model against the paper's reported behaviour.
- A determinism test: same seed and spawn order, run twice, identical state hash; also across a save/reload if state is serialized.
- A scenario test per behaviour you touched.
- Run `cargo +1.98.1 test` in both feature sets, `cargo +1.98.1 clippy --all-targets --all-features -- -D warnings`, `cargo +1.98.1 fmt --check`, and `RUSTDOCFLAGS="-D warnings" cargo +1.98.1 doc --no-deps --all-features`.
- Measure tick cost at 100/1,000/5,000 persons into `perf.md`, before and after. A change must not slow a scenario it did not target.

## 6. Look at it (a mandatory gate)

A simulation you cannot see is a simulation you cannot judge. Build or extend a headless harness (an example under `examples/`) that writes evidence from a run, then look at it yourself:

- **Plots:** need levels over a day per person, mood in PAD space, opinion per topic over time, rumor spread, relationship graph snapshots, faction membership, where people are at each hour. Render with matplotlib or similar, then zoom into the interesting region. Compare before and after at identical seed.
- **Narrative:** render the message stream for one person and one scenario as a readable day ("07:10 woke, hungry; 07:25 walked to X; 08:00 ate with Y, trust rose"). Read it as a stranger: does it make sense, is there a reason for each step?
- **Optional spatial view:** if the dialogue/steering examples can run in a window, capture screenshots and mp4s (ffmpeg) of people walking, talking and clustering.

Three passes, each with a written critique before the verdict. Verdicts: GREAT, NOT YET, FAIL. Only GREAT passes. Log all three in `review.md`, citing evidence filenames.

1. **Behaviour:** does the change do what the paper and the scenario say, at the intended time scale?
2. **Zoom:** read the plots closely. Look for oscillation, saturation, clipping at 0 or 1, drift, everyone converging, dead agents, spikes at the tick boundary.
3. **Fresh eyes:** regenerate from scratch with a different seed and population. Does it still look believable, and can the narrative be explained to a host developer without reading the code?

Any fix returns you to the earliest pass it could affect. After 5 failed rounds, log the blocker and revert.

## 7. Ship, in this order

Update `docs/`, `docs/research/` and `CHANGELOG.md`; make sure the README's "current state" matches reality. Commit, push, open a PR, merge to main, pull main. Never bump the version or publish.

Follow the repository's git rules without exception: **never force-push**, never rebase pushed commits, never `git reset --hard`, never delete work you did not create in this session. A commit on the wrong branch is fixed with a new commit or a question.

## 8. Log

Append to `log.md`: change, rationale, evidence links, before/after numbers, the dead-code inventory delta, verdicts, commit, and the top 5 remaining items.

## Rules

- One change per iteration: researched, tested, three GREAT reviews, shipped, logged.
- Nothing ships without plots or narrative you looked at, and before/after at the same seed.
- Every default and formula has a source or says it is a design default.
- If blocked, log it and pick the next item.

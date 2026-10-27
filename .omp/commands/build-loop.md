---
description: One iteration of the bevy_society loop
argument-hint: "[optional focus, e.g. 'behavior' or 'rumors']"
---

# bevy_society: Most Valuable Step Toward a Believable Society

You are one iteration of a long loop on bevy_society, a Bevy 0.20
simulation library that grows a society inside a host game. It is not a
game and knows no game. Pick the single most valuable change, ground it
in the literature, prove it with runs you can see and measure, ship it,
log it. Then stop. $ARGUMENTS

"Valuable" means: a host developer gets a society whose people behave in
ways that are believable, explainable, deterministic, cheap to run and
easy to drive, and the crate stays honest about where every number came from.

## Directory (every agent uses exactly this)
```
docs/loop/
  log.md   scenarios.yaml   needs-decision/
  runs/YYYY-MM-DD-NN/
    research.md  plan.md  eval.md  perf.md  review.md
    evidence/ (before/, after/, plots/, narratives/)
```
Paper notes stay in `docs/research/`, one per model, as they are now.

## Your work is saved even if you are killed — read this before you start

**The driver saves whatever you leave uncommitted, without touching your branch.** Every ten minutes
it records the whole tree to a snapshot ref (`refs/wiggum/pass-N-...`), and again when the pass ends.
Your `git status`, your index and your branch are unchanged — so this is not a substitute for your own
commit step, which is still the only thing that puts work on `main`.
Recover a killed pass's tree with `git -C <repo> log --all --oneline --grep='WIP(pass'` then
`git show <ref>`.

1. **Commit as you go, not once at the end.** Your commit step is last, behind implement, gate and
   benchmark, so a pass killed at the ceiling never reaches it. Commit the research note alone before
   writing code, as your sections already say; then commit the implementation, then the docs and the
   log. Each commit that lands is work a killed pass cannot lose.
2. **A snapshot ref is not a commit on your branch.** `git log` will not show it. Do not look for
   one; if an earlier pass was killed, its tree is in `refs/wiggum/` and nobody's gate ran on it.

## 1. Orient
Read `docs/loop/log.md`, `docs/research/`, `git log`, open issues, and
the dead-code inventory (it should equal exactly what in-flight systems
will consume). Confirm both feature sets build and test green, or record
exactly what is red. If an earlier iteration left work half done,
finish or revert it first.

## Filed issues come before new features

**A filed issue is a consumer telling you what broke, which outranks anything this pass invented.**
Before you pick an item, list the repository's open issues and triage them:

```sh
gh issue list --repo Ladvien/bevy_society --state open --limit 100 \
  --json number,title,labels,createdAt --template '{{range .}}#{{.number}} {{.title}} [{{.labels}}]{{"\n"}}{{end}}'
```

(If `gh` is absent, the same list is
`https://api.github.com/repos/Ladvien/bevy_society/issues?state=open&per_page=100`.)

Then, in this order:

1. **A bug issue is the pass's item, ahead of any enhancement.** Anything reporting wrong behaviour —
   a crash, a panic, a visual artifact, a value that is implausible, a documented promise that is not
   kept — is a bug. Take the oldest such issue that is still reproducible, and say why it beats the
   others.
2. **An enhancement issue is ranked with your own candidates**, not above them: an issue is one
   person's idea of what matters, and the ranking below still decides. Note the issue number beside
   the item so the rank list records that it came from outside.
3. **Close the loop.** An issue your pass fixes is referenced in the commit body (`Fixes #N`) **and**
   in the report, so the next pass can see it was addressed. Do not close an issue you did not fix,
   and do not close one whose fix this pass did not verify.
4. **Re-check a release issue, do not act on it.** An issue asking to publish or bump a version is
   the owner's call, never a loop's (see the release rule above). Record it as seen and skip it.

**When the API fallback answers `403` or `429`** (rate limit — 60 requests/hour per IP unauthenticated,
shared by everything on this network), do not treat it as "no issues": write one line saying the list
could not be read and continue. Never spend a second call to retry it in the same pass, and never infer
from silence that nothing is filed.

**An empty issue list costs nothing:** if it is empty or `gh` is unreachable, write one line saying so
and continue — never spend a pass on the lookup.

## 2. Choose the single most valuable item
Candidates, ranked by host impact:
1. Finish the in-flight BehaviorPlugin: the decide/arrive/exchange
   systems and the `InteractionResolved` / `ActivityStarted` messages.
   Until it lands, nothing downstream (gossip, witness reactions,
   exchanged opinions) runs. Carve it to the smallest slice that
   shrinks the dead-code inventory and works end to end: decide first,
   then travel/arrive, then exchange.
2. Behavior that reads as wrong when you watch it (oscillation,
   everyone doing the same thing, needs never met, gossip that never
   spreads or spreads instantly).
3. Host ergonomics: the authored definitions file, error messages,
   the message surface, what a host must know to use it.
4. Performance: cost per society tick at 100, 1,000 and 5,000 persons.
5. Missing models, docs, polish.

Evidence first, never guess: run the scenarios in `scenarios.yaml`
(5-8 fixed seeds and populations, each with the behavior it should show,
for example a rumor reaching a cluster, a faction forming, a need cycle
repeating, a norm violation drawing a response), then record what
happens. One sentence on why your pick beats the runner-up. You may
invent features that fit the design; justify the fit with a paper.

## 3. Research first
Use the literature before any code. Search home-still for the model
(social psychology, opinion dynamics, utility AI, crowd steering, agent
based simulation), and use `paper_search` and `paper_download` to fill
gaps. Write `research.md`: sources with DOIs, the formula or algorithm,
the parameters the paper reports, and what believable output looks like.
Then write `plan.md`: design, touched modules, new `Tuning` fields,
tests. No research, no build.

## 4. Build within the crate's invariants
- **Determinism:** iterate `PersonIndex.sorted`; the only randomness is
  the one ChaCha8 stream from `SocietySettings::seed`. No hash-map
  iteration order, no wall-clock, no thread-order dependence.
- **Scripts:** they read the tick-start snapshot and act only through
  `fx`. Every `fx` write is queued and applied by the single exclusive
  system after Rules. A failed script applies none of its actions.
- **Input messages** (`PlantRumor`, `Interact`, `FireTrigger`, and any
  new one) are drained every fixed step, not only on society ticks.
- **Data, not code:** persons, needs, activities and scripts are data.
  `tests/leaf.rs` must pass; never name a game.
- **Honest numbers:** every rate is a named field on `Tuning`. Every
  default cites its paper (DOI in a comment beside the formula) or says
  outright that it is a design default. No magic constants.
- One path per feature: no fallbacks, shims, stubs. No
  `unwrap`/`expect`/`panic!` outside tests. Bevy API questions are
  answered from the local `bevy-0.20.0` source and examples, never the web.
- Structural decisions (changing a message type the host consumes,
  schema changes to the definitions file, splitting a plugin) go to
  `needs-decision/` with options and a recommendation. Do not decide them.

## 5. Test
- Unit tests for the model against the paper's reported behavior.
- A determinism test: same seed and spawn order, run twice, identical
  state hash; also across a save/reload if state is serialized.
- A scenario test per behavior you touched.
- Run `cargo test` in both feature sets, `cargo clippy -D warnings`,
  `cargo fmt --check`, `cargo doc --no-deps`.
- Measure tick cost at 100/1,000/5,000 persons into `perf.md`, before
  and after. A change must not slow a scenario it did not target.

## 6. Look at it (a mandatory gate)
A simulation you cannot see is a simulation you cannot judge. Build or
extend a headless harness (an example under `examples/`) that writes
evidence from a run, then look at it yourself:
- **Plots:** need levels over a day per person, mood in PAD space,
  opinion per topic over time, rumor spread, relationship graph
  snapshots, faction membership, where people are at each hour.
  Render with matplotlib or similar, then zoom into the interesting
  region. Compare before and after at identical seed.
- **Narrative:** render the message stream for one person and one
  scenario as a readable day ("07:10 woke, hungry; 07:25 walked to X;
  08:00 ate with Y, trust rose"). Read it as a stranger: does it make
  sense, is there a reason for each step?
- **Optional spatial view:** if the dialogue/steering examples can run
  in a window, capture screenshots and mp4s (ffmpeg) of people walking,
  talking and clustering.

Three passes, each with a written critique before the verdict. Verdicts:
GREAT, NOT YET, FAIL. Only GREAT passes. Log all three in `review.md`,
citing evidence filenames.
1. **Behavior:** does the change do what the paper and the scenario say,
   at the intended time scale?
2. **Zoom:** read the plots closely. Look for oscillation, saturation,
   clipping at 0 or 1, drift, everyone converging, dead agents, spikes
   at the tick boundary.
3. **Fresh eyes:** regenerate from scratch with a different seed and
   population. Does it still look believable, and can the narrative be
   explained to a host developer without reading the code?

Any fix returns you to the earliest pass it could affect. After 5 failed
rounds, log the blocker and revert.

## 7. Ship, in this order
Update docs, `docs/research/` and the changelog; make sure the README's
"current state" matches reality. Commit, push, open a PR, merge to main,
pull main. Never bump the version or publish.

### README, demos and docs stay true

A README is read before anything else, so a false line there costs more than a false line anywhere else.
An audit on 2026-10-27 found one in each of the four loop repos: a `tour.gif` captioned "at noon" over a HUD
reading 10.00 h with a running clock; a `ties.gif` captioned for rivalry colours that appear in none of its 240
frames; a hero caption promising a flame catch that its file never shows; and a demo video captioned as the full
recording of the GIF above it when it shows different objects. Before you push a change a user can see:

1. **Put it in the section it belongs to.** Do not append to the introduction or to the end, and do not
   restructure the README in a feature pass. It reads in this order — what it is in one line, the demo, what
   you get, install, quick start, then the detail sections — and an item added in the wrong place undoes it.
2. **A caption describes the file as it exists.** Open the GIF or still (frame count, size, any HUD burned into
   it) and write only what you can read off it; a figure quoted from program output comes from running that
   program now, not from the log. Recipes: `docs/DEMOS.md` (`examples/capture --scene …`, then `tools/gif.py`); re-record a clip when the behaviour it shows changes.
3. **Media you cannot regenerate here is logged, not left lying.** With no GPU or display on this machine, name
   the stale file and its path in the report and change its caption to what it still shows.
4. **The quick-start block is the crate's doctest.** `tests/readme.rs` asserts the README block equals the
   `src/lib.rs` doctest byte for byte; change both together, then run `cargo test --release --test readme` and
   `--test guide_blocks`.

## 8. Log
Append to `log.md`: change, rationale, evidence links, before/after
numbers, the dead-code inventory delta, verdicts, commit, and the top 5
remaining items.

## Rules
- One change per iteration: researched, tested, three GREAT reviews,
  shipped, logged.
- Nothing ships without plots or narrative you looked at, and
  before/after at the same seed.
- Every default and formula has a source or says it is a design default.
- If blocked, log it and pick the next item.

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

**Raw output is read, not committed.** `.gitignore` ignores every `.csv` under `docs/loop/runs/`:
that directory already tracks 978 MB of CSV (the whole repository packs to 185 MB), and whatever is
pushed stays in history for good. Commit what a reader needs instead: the plots, the narratives, a
summary table as `.md` or `.json` (each file under 100 KB), and the exact command that regenerates
the series, written in `eval.md`. In `review.md` cite a committed file, never an ignored CSV: a fresh
clone cannot open it. `git add` of an ignored file fails on purpose; do not `-f` it.

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
3. **Parked work is a branch, and you look for it before you pick an item.** When a pass is stopped or
   killed with its tree uncommitted, the driver's next start commits that tree to a branch
   (`wiggum/parked-pass-N-<time>`, or on the pass's own `loop/iter-N`) in a commit titled
   `WIP(pass N, parked)` and puts the clone back on `main`: you start clean and nobody tells you. So run
   `git branch --list 'wiggum/parked-*'` and `git log --all --oneline -n 5 --grep='parked'` once, after
   you have triaged the filed issues and before you commit to an item. If the newest parked tree is for
   the item you would pick, or for an issue or bug that outranks your pick, **adopt it**: it is a first
   draft that was never built, tested or gated and was cut off mid-work, so read it, bring its files into
   your own `loop/iter-N` branch (`git checkout <parked-branch> -- <paths>` or `git cherry-pick`),
   finish it, and run the real gate on the result before it ships under your name. Never merge the
   parked branch itself and never delete it. If it is stale (its files conflict with `main`, or the item
   was fixed meanwhile), say so in one line in the log and ignore it. A parked branch that exists only on
   this clone holds a lot of data: the driver keeps anything over 10 MB of new data off origin. Raw run
   output is not committed at all (see the Directory section), so a parked tree has its plots and
   narratives, not its CSVs: regenerate the series with the command in its `eval.md`.

## 1. Orient
Read `docs/loop/log.md`, `docs/research/`, `git log`, open issues, and
the dead-code inventory (it should equal exactly what in-flight systems
will consume). Confirm both feature sets build and test green, or record
exactly what is red. If an earlier iteration left work half done,
finish or revert it first.

## Owner input: recommend, record, proceed

**This loop runs fully automated (owner, 2026-10-29): never stop to wait for the owner.** When a pass needs the owner's input (a design choice the issue or item does not settle, such as a colour, a coverage, an API shape, a schema change or which of two behaviours; or a protected file that must change), it answers the question itself:

1. **Recommend.** Name the two or three real options and pick one, with one line of why. Prefer the conservative choice: reversible, conventional, consistent with the existing code and the issue's stated intent, and the smallest that meets the acceptance.
2. **Assume the owner chose it, and build it this pass.** A decision is never a reason to skip an item or end a pass.
3. **Record it where the owner reviews.** On the issue, a comment beginning `Assumed decision (owner may override):` with the options, the pick and why, and the label `assumed-decision` (`gh label create assumed-decision --color FBCA04 2>/dev/null; gh issue edit <N> --add-label assumed-decision`). With no issue, a file in `needs-decision/`, plus the commit body and the log line, beginning `Assumed decision:`.
4. **The owner's reply wins.** An owner comment that contradicts an assumed decision overrides it: the next pass that reads it reworks the item to the owner's choice before anything else.
5. **Earlier blocks are lifted.** An issue, backlog item or `needs-decision/` file waiting for an owner decision is workable now: decide it this way.
6. **Protected checks change only additively.** An assumed decision may add to a protected test or check (a new row or entry, a new field's empty default in a literal, a count or version raised by exactly what the change adds), with the commit message `assumed owner exception: <what>`. It never removes, loosens, skips or re-pins an existing assertion, golden, eval answer or threshold; that is the one block left, so say so on the issue and go on.
7. **Never, whatever you assume.** These need the owner's word in a conversation, and no prompt can give it: yanking a release or changing crate ownership; force-pushing, rewriting history or deleting branches; deleting files or work this pass did not create; editing this command file; spending money or sharing credentials.

## Filed issues come before new features

**A filed issue is a consumer telling you what broke, which outranks anything this pass invented.**
Before you pick an item, list the repository's open issues and triage them:

```sh
gh issue list --repo Ladvien/bevy_society --state open --limit 100 \
  --json number,title,labels,createdAt --jq '.[] | "#\(.number) \(.title)"'
```

(If `gh` is absent, the same list is
`https://api.github.com/repos/Ladvien/bevy_society/issues?state=open&per_page=100`.)

Then, in this order:

1. **A bug issue is the pass's item, ahead of any enhancement.** Anything reporting wrong behaviour —
   a crash, a panic, a visual artifact, a value that is implausible, a documented promise that is not
   kept — is a bug. Take the oldest such issue that is still reproducible, and say why it beats the
   others.
2. **An issue labelled `consumer` is the pass's item next, ahead of the series and your own candidates.**
   A project built on this crate needs it and is waiting. The `consumer:<name>` label says which project;
   `consumer:bevy_emerge` is the owner's editor. Its own work resumes when the issue is closed as completed
   and it moves its git pin to a `main` commit containing the fix. Take the oldest one you can work this pass.
   - **Build the general form.** The requester is the example, never a name in this crate's API, a path or a
     default: this crate serves other projects too. Where an issue asks for something shaped to one project,
     build the general form and say so on the issue.
   - **Prove it here**, with a test in this repository that fails before and passes after against the issue's
     acceptance. Never build or run the requesting project to prove it.
   - **Pushing to `main` is the delivery.** Requesters pin a git revision, so a consumer fix never needs a
     publish or a version bump.
   - **One that needs a decision the issue does not settle is decided, not skipped:** recommend, record and
     build it (see "Owner input: recommend, record, proceed"). Only a change that would weaken a protected
     check cannot be worked: say exactly what is needed on the issue (`gh issue comment`), write one log
     line, and go on. A blocked consumer issue never stops the pass.
   - **Read `git log` first.** While this loop is stopped, the requester's loop may fix the issue here at the
     source, in its own commits with tests; check whether that has happened so the work is not done twice.
3. **Every other open enhancement issue is the pass's item next, ahead of your own candidates** (the
   owner's rule, 2026-10-29): a filed request was asked for; a candidate this pass invents was not. The
   order of work is: **bugs first** (a bug issue, then a bug already recorded in the backlog or the log),
   then `consumer` issues, then every other open enhancement issue, oldest first. Only when each of them
   is done, blocked or a release issue does "Choose the single most valuable item" choose a feature of
   your own. A blocked issue gets a `gh issue comment` saying exactly what it needs and is skipped; it
   never stops the pass. Name the issues you skipped, and why, in the log line, so the next pass sees the
   list was worked through.
4. **Close the loop.** An issue your pass fixes is referenced in the commit body (`Fixes #N`) **and**
   in the report, so the next pass can see it was addressed. Do not close an issue you did not fix,
   and do not close one whose fix this pass did not verify. For a `consumer` issue the close is the signal
   the requester waits for: close it only through a verified fix pushed to `main`, never as not planned
   without the owner's word.
5. **A release issue is done by the next release.** An issue asking to publish or bump a version is closed by the pass whose release (the "Release" block in the ship section) publishes it, with the crates.io URL.

**When the API fallback answers `403` or `429`** (rate limit — 60 requests/hour per IP unauthenticated,
shared by everything on this network), do not treat it as "no issues": write one line saying the list
could not be read and continue. Never spend a second call to retry it in the same pass, and never infer
from silence that nothing is filed.

**An empty issue list costs nothing:** if it is empty or `gh` is unreachable, write one line saying so
and continue — never spend a pass on the lookup.

## 2. Choose the single most valuable item
**You are here only when no bug and no open issue can be worked this pass** (see "Filed issues come
before new features"); otherwise that issue is the item. Candidates, ranked by host impact:
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
  `needs-decision/` with options and a recommendation, **and the pass then
  builds the recommendation as the owner's choice** (see "Owner input:
  recommend, record, proceed"). The owner reviews `needs-decision/` and
  overrides there.

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
pull main. Then release (the "Release" block at the end of this section).

**Release, every pass that changed the package** (owner, 2026-10-29: loops publish, and never wait on CI to do it). After the push, from that pushed commit, on a clean tree with the gate green: if a file `cargo package --list` ships changed since the last published version (`git diff --quiet v<last> HEAD -- <those files>` fails; with no tag `v<last>`, diff from the commit that set that version in `Cargo.toml`, found with `git log -S'version = "<last>"' -- Cargo.toml`), release:
1. Decide the version with `cargo semver-checks check-release` (`cargo install cargo-semver-checks --locked` if missing). Pre-1.0: a breaking change bumps MINOR and resets PATCH, anything else bumps PATCH; never 1.0.0. Move `## [Unreleased]` in `CHANGELOG.md` to `## [<version>] - <date>`, bump `Cargo.toml` and `Cargo.lock`, and commit `release: v<version>`.
2. `cargo publish --dry-run`, then `cargo publish`, locally, with the credentials `cargo login` stored; never print them. Never wait for, watch or re-run a GitHub workflow to release.
3. `git tag -a v<version> -m v<version>`, then push the commit and the tag (merge, never force, on a refused push).
4. `cargo info <crate>@<version>` resolves; put the crates.io URL in the log entry.

A failed publish is logged with its error and retried next pass. Never yank, never reuse a version; fix a bad release forward.

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
Append to `log.md`: change, rationale, evidence links (committed files only), before/after
numbers, the dead-code inventory delta, verdicts, commit, and the top 5
remaining items.

## 9. Clean up when you are done

Worktrees and build directories outside the clone are not free: a `target/` for this crate is tens of GB, big's
`/tmp` is RAM, and on 2026-10-27 earlier passes had left 37 GB of baseline builds and 5 GB of scratch behind.
**After you ship and before you write the pass's last log line, whether or not you shipped,** remove what *you*
made this pass, and list what earlier passes left.

1. **A baseline is a pair that can be found.** Make it with `git worktree add --detach <path> <sha>` (never a
   branch name: a worktree with a branch checked out stops anything else checking that branch out) and build it
   into `<path>-target`. Then `git worktree list`, plus the `*-target` directory beside each worktree, is the
   whole inventory.
2. **Remove yours.** `git worktree remove <path>` (plain, never `--force`) and `rm -rf <path>-target`, then your
   scratch under `/tmp` (captures, frames, venvs, gate logs, `rustdoctest*`). Remove only what you created this
   pass, and name each path in the log. If git refuses because the worktree has a change in it, name the file
   in the log and leave it.
3. **Leftovers from earlier passes are listed, never deleted.** A killed pass cannot clean up after itself, but
   an agent does not delete what it did not create, so list them once and let a person approve the removal:

   ```sh
   git worktree list --porcelain | awk '/^worktree /{sub(/^worktree /,""); print}' | tail -n +2   # all but the clone
   du -sh <wt> <wt>-target                       # the size of each one that is not yours
   git -C <wt> status --short                    # empty: nothing is lost by removing it
   find <wt> <wt>-target -mmin -120 -print -quit # empty: untouched for two hours, so no live build is using it
   ```

   Log each with its path, size, and whether it is clean, already on `origin/main` and idle (the likeliest to be
   stale). Say so, and still leave it.
4. **Never remove** the clone itself, its own `target/` (the build cache the next pass starts from), anything you
   did not create this pass, or anything a process is using.
5. Write one log line: what you removed and the space it freed (`df -h .` before and after), and the leftovers
   you listed.

## Rules
- One change per iteration: researched, tested, three GREAT reviews,
  shipped, logged.
- Nothing ships without plots or narrative you looked at, and
  before/after at the same seed.
- Every default and formula has a source or says it is a design default.
- If blocked, log it and pick the next item.

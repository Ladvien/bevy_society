# bevy_society — notes for agents

A Bevy 0.20 library that simulates societies of NPCs: personalities, moods, needs, memories, relationships, opinions, rumors, reputation, factions and status; utility-AI daily lives; Rhai-scripted triggers; and dialogue and barks through `bevy_speech_bubbles`. Every model is distilled from social-psychology and game-AI papers held in the home-still corpus, and every behaviour is deterministic for a given seed.

## Force pushes and deletions need the user's express permission

Never do any of the following unless the user has approved that specific action, by name, in the current conversation. A loop brief, a skill, a standing instruction such as "commit, push, PR, merge", or an earlier session's approval is not permission.

- Force-push in any form: `git push --force`, `--force-with-lease`, `--force-if-includes`, a `+` refspec, or pushing rewritten history. Deleting a remote branch or tag (`git push --delete`, `gh pr merge --delete-branch`, an API delete).
- Throwing away commits or uncommitted work: `git reset --hard`, `--merge` or `--keep`; `git checkout -- <path>` or `git restore` over changes you did not make; `git clean`; `git stash drop` or `clear`; `git branch -D`; `git worktree remove --force`; `git rebase` or `git commit --amend` on pushed commits; `git filter-repo` or `filter-branch`.
- Deleting files, directories, build caches, target dirs, models, datasets, database rows, issues, PRs, releases or cloud resources that you did not create in this session (`rm`, `rm -rf`, `git rm`, `cargo clean`, an API delete).

When one of these seems necessary, stop and ask. Name the exact command, what it destroys and whose work it touches. A commit that landed on the wrong branch is fixed with a new commit or a question, never by resetting a shared working tree or force-pushing. Files you created yourself in this session (scratch scripts, temp dirs, your own worktrees) you may delete.

## Source of truth

This repository, [`Ladvien/bevy_society`](https://github.com/Ladvien/bevy_society), is canonical. MIT OR Apache-2.0. Its ladvien dependencies (`bevy_speech_bubbles`, `bevy_stigmergy`, `bevy_orca`, `bevy_emerge_rng`, and `bevy_fairy` in the examples) are consumed by pinned git `rev`. A missing feature or a bug in one of them becomes an issue (`enhancement` or `bug`) or a PR on that repository, never a patch or a copy here: search its open issues first and add to a matching one. Bump a `rev` here in its own commit.

## Version pin

Pinned to **Bevy 0.20.0**. Read the vendored source (`~/.cargo/registry/src/index.crates.io-*/bevy-0.20.0/`, its `examples/` and `_release-content/migration-guides/`), not bevy.org, which documents `main` and has been wrong for a pin more than once. MSRV is 1.97.1 (Bevy 0.20's). On big_mac the default toolchain is older: run every command as `cargo +1.98.1 …`.

## Layout

- `src/lib.rs` — crate docs and flat re-exports (no prelude).
- `src/defs.rs` — authored definitions (`DefsFile`), validation, and the compiled `SocietyDefs` with id interning and spawn helpers.
- `src/script.rs` — Rhai limits and `ScriptCompiler` (compile-time only; the runtime `ctx`/`fx` API lands with D8).
- `src/mind.rs` — `MindPlugin` (`SocietySystems::Mind`): emotions and the `Appraise` message, PAD mood with contagion, needs decay, personality and the `Add<Person>` baseline observer.
- `src/model/mod.rs` — engine-free math module root; re-exports `Curve`.
- `src/model/curves.rs` — response curves (`Curve`, `logistic`, `hour_window`).
- `src/model/pad.rs` — personality-to-mood baseline (Mehrabian 1996, ALMA).
- `src/model/recall.rs` — recency, power-law familiarity, exposure, grudge.
- `src/model/opinion.rs` — Deffuant influence, anchor pull, similarity, conformity.
- `src/model/rumor_dk.rs` — Daley–Kendall stifling, telling weight, belief after hearing.
- `src/model/balance.rs` — triadic balance step and relaxation toward zero.
- `src/relationships.rs` — `Ties` (directed weighted edges), familiarity and exposure, affinity relaxation, structural balance, `Dunbar` capacity trim, and tie labels (`RelationshipsPlugin`, set `Relate`).
- `src/groups.rs` — `Memberships`, `Status` (conferred standing), contests, intergroup contact, and `FactionStats` (`GroupsPlugin`, set `Relate`).
- `src/fields.rs` — social fields (feature `fields`): crowd presence/commotion and per-faction presence grids on `bevy_stigmergy`, `FieldInput` scoring and gradient targets.
- `assets/starter.society.ron` — the starter pack, parsed by `DefsFile::starter()`.
- `tests/defs.rs` — definition parsing, merging, id interning and error messages.
- `tests/fields.rs` — millers' presence field points toward their cluster (feature `fields`).
- `tests/leaf.rs` — the crate boundary: dependency list, forbidden names, engine-free `src/model/`, lookup-only hash collections.
- `tests/mind.rs` — emotion half-life, mood lift toward Joy and return to baseline, needs decay at `decay_per_day`.

## Build and test

- `cargo test` and `cargo test --no-default-features`.
- `cargo build` and `cargo build --no-default-features`: these take no dev-dependencies, so they compile exactly the bevy features a consumer gets; the full-umbrella dev-dependency would hide a hole in the trimmed feature list.
- `cargo build --examples`.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`.
- Examples need a font path: `cargo run --example village -- /System/Library/Fonts/Supplemental/Arial.ttf`.

## Research

Every model comes from a paper in the home-still corpus. Search it with the home-still MCP tools `distill_search` (chunks) and `abstract_search` (one hit per paper), and read with `markdown_read`. Put the DOI in a code comment at the formula (`// Deffuant et al. 2000, DOI 10.1142/S0219525900000078`) and in `docs/research/<topic>.md`. New papers are added with the corpus pipeline in `tools/corpus/` (documented in `docs/research/corpus/README.md`).

## The non-negotiables

- **No game knowledge.** `tests/leaf.rs` lists forbidden names (a game's camera marker, physics crate, the games this crate serves). Persons, places, needs, activities and scripts are data the host hands in.
- **One path per feature.** No fallbacks, no legacy shims, no stub placeholders.
- **No `unwrap()`, `expect()` or `panic!` outside tests,** except the documented plugin-order checks in each plugin's `finish()`. Misconfiguration gets a loud `error!`/`warn!`, never a panic.
- **Determinism.** Iterate persons sorted by `PersonId` (`PersonIndex.sorted`), draw only from `SocietyRng`, and never iterate a `HashMap`/`HashSet` (lookup only; `tests/leaf.rs` allow-lists the files that may name them). Same seed and same inputs give the same society, bit for bit.
- **Scripts see the tick-start snapshot and act through `fx`,** never the world.
- **Every rate is per game-day** and every dt is `SocietyClock::dt_days`.
- **`src/model/` is engine-free math**: no `bevy` in it.

## Design facts worth keeping

- **`publish = false`** because the ladvien dependencies are `publish = false` git dependencies; it can be published once every one of them is on crates.io.

## Rules

- **A missing `Res<T>` panics its system.** Take `Option<Res<T>>`, or have the plugin `init_resource` it.
- **All run conditions are evaluated — there is no short-circuit.** A bare `Res<T>` in a `.run_if(..)` closure panics whenever that resource is absent.
- **`Single<..>` silently skips its system on a non-unique match.**
- **Lifecycle observers take the event:** `On<Add<A>>`, `On<Remove<A>>`. Pointer events are flat in 0.20 (`On<PointerClick>`).
- Do not hard-wrap markdown.
- Close any Bevy app you launch for a test.
- Every dependency added to `Cargo.toml` gets a GitHub link in its comment and a row in README's Built on table.
- Changing the starter pack or a default changes `docs/research/*.md` and the CHANGELOG in the same commit.

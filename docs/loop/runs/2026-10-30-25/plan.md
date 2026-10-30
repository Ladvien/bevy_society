# Plan: `Tuning::forgiveness`

**Pick.** No open issues (bevy_society; bevy_emerge's only open issue for this crate, #106, has its crate side on `main`). Parked branches are older passes' items, already adopted or stale. Ranked candidates: (1) the town rivals itself: 277 rival pairs against 49 friend pairs at 60 people, 75% of snubs answering an insult received (weakest model, behaviour that reads wrong); (2) re-measure the scale table on a quiet host (load 69 now: not measurable); (3) attribute the other half of `decide_activities` (no profiler permission); (4) restate `rumor-crosses-a-cluster`. Pick (1): it is the defect a host sees first, and `needs-decision/what-does-the-village-owe-its-grudges.md` option 3 ("grudges fade on their own terms") was waiting for a paper, which `research.md` supplies. It beats the runner-up because the runner-up cannot be measured on this host today.

**Design.** `recall::forgiving_pace(trust, forgiveness) = 1 + forgiveness · clamp(trust)`; `Memories::grudge_at_pace(other, now, pace)` reads each memory at `age · pace` (`Memories::grudge` is pace 1, unchanged); the snapshot copies `Tuning::forgiveness` and `ctx.grudge(a, b)` uses `a`'s trust in `b` (0.5 with no tie, as `ctx.trust`). Nothing else reads a grudge, so this one path is the feature.

**Touched.** `src/model/recall.rs` (pace + unit test), `src/mind.rs` (`grudge_at_pace`), `src/core.rs` (`Tuning::forgiveness`), `src/snapshot.rs` (field + `grudge`), `examples/evidence.rs` (`--forgiveness`), `tests/forgiveness.rs`, `docs/research/memory.md`, `CHANGELOG.md`.

**New `Tuning` field.** `forgiveness: f32`, design default. Ship at the measured value if every protected test passes with it on, else ship `0.0` (bit-identical to HEAD) and record the measured value as the recommendation. Known risk: `tests/rumor_book.rs` pins over 1000 rumors at 200 people, and rumors come only from insults in the starter pack, so fewer insults may fail it (pass 24's pack change did).

**Tests.** `recall` unit (pace off is 1, grows with trust, clamps); `tests/forgiveness.rs` (a trusted offender's grievance falls below a trigger's bar while a distrusted one's does not; off, trust changes nothing; a forgiving village is deterministic). Evidence: `examples/evidence` before/after at village seeds 7/42/11 and the 60-person town at seed 7, fresh eyes at seed 99 for both.

**Not saved state.** The pace is computed from saved state (memories, ties) and a `Tuning` field, so a save needs nothing new.

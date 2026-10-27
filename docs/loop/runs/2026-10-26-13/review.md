# Review — `DefsFile::merge` carries `goods` and `skills`

Three passes. The iteration's *first* candidate (diluting the affinity a witness writes) is recorded in
`eval.md` as measured, rejected and reverted; this review is of what shipped.

## Pass 1 — Behaviour: does the change do what it claims?

**The defect, reproduced before it was fixed.** `merge` carried nine of eleven lists. `compile` resolves
`consumes` and `produces` by name, so the omission did not surface at the merge — it surfaced as
`DefsError::UnknownId { kind: "good" }` against whichever *activity* used the good, in a pack that plainly
declares it. Three tests were written against the old code and **all three failed**:

| test | old `merge` | new `merge` |
|---|---|---|
| `a_merged_pack_contributes_its_goods` | **FAIL** | pass |
| `a_good_defined_in_both_packs_is_a_duplicate` | **FAIL** (no duplicate error; the good was dropped instead) | pass |
| `a_good_only_the_merged_pack_consumes_resolves` | **FAIL** | pass |

And with the fix in place, the whole existing `tests/defs.rs` (8 prior tests) still passes — so the eleven
lists that already merged were not disturbed.

**Why this and not the runner-up, again.** This was the loop log's #1 for two iterations, and last iteration's
own review filed it as "a schema question, not a drive-by" — the question being *what does a merge mean for a
good the other pack already names?* It has the same answer the other ten lists already had: a duplicate id is
`DefsError::Duplicate`, and the receiver is unchanged. There was no open question, only an unasked one, and the
cost of asking it was one `check_disjoint` + one `extend` pair. It now matches the guide's own promise,
"merging rejects a **duplicate id** — in any list", which was false when it was written.

**Verdict: GREAT.**

## Pass 2 — Zoom: the shipped packs, not a synthetic fixture

The reason this defect mattered is that **every one of the crate's own examples does this merge**:

- `examples/evidence.rs` — `starter` + `village.society.ron`
- `examples/capture.rs` — same
- `examples/village.rs` — same

So the fix is exercised by every scenario the loop has ever measured, and the check that it changed nothing
there is the strongest available: the three `village-evening` runs collected for the rejected candidate were
produced on the **pre-fix** build and are the *before* for this change. `goods` was already the only list the
village pack did not contribute, so the merge result is byte-identical and no scenario can move.

I verified that rather than assuming it: `village.society.ron` declares no `goods:` or `skills:` at all
(`grep -n "goods\|skills" examples/assets/village.society.ron` → no match), so the two lists now carried are
both empty in the merge the examples perform. That is what makes "no before/after numbers for this change" an
honest statement rather than a missing measurement: **the change has no behavioural surface in the shipped
packs**, and the tests are the evidence.

**Verdict: GREAT**, with the absence of before/after numbers recorded as a property of the change rather than
an omission.

## Pass 3 — Fresh eyes: what does a host developer see?

The failure this fixes is a host's, not the crate's: split a pack, add a good in the second file, and the error
you get names an activity in the first. Told without reading code:

> *"If you split your definitions across files, `merge` now carries goods and skills with everything else. Before
> it silently dropped them, so a good you added in a second file made the first file's `consumes` fail to
> compile. A good named in both files is a duplicate, like every other id."*

The guide says exactly that, and the changelog says it with the upgrade note a 0.3.0 host needs.

**Verdict: GREAT.**

## What the iteration records as still wrong

1. **The reverted candidate, and its real finding.** Diluting the affinity a witness writes takes rivals to
   **zero** at seed 7 (27/2 → 39/0) and leaves seed 11 at zero. Reverted. The finding that matters is the
   premise: `needs-decision/good-is-outweighed-by-bad.md` reported the village at **18 friends / 42 rivals**,
   and it is measured at **27/2, 21/3 and 20/0**. That file is corrected in this iteration, because every
   decision resting on it is now resting on an inverted fact.
2. **Seed 11 has no rivalry at all, at any point in 17 game-days.** Not investigated. It is the new open
   question, and it is sharper than the old one: the crate can produce a village with zero hostility and
   nothing warns.
3. **`warn_unmade_goods` is silent for a good nothing declares at all** — it iterates declared goods, so a
   `consumes: [("flour", 1.0)]` from a pack whose good list never arrived fails at compile instead. That is the
   `UnknownId` path and it is correct, but it is the message the merge bug used to produce, which is why the
   diagnosis was hard. No change proposed.
4. `Tuning::trade_reserve` still keeps four meals after a meal became a quarter loaf. Carried.
5. `rumor-crosses-a-cluster` still `pending`; the clock decision still open.

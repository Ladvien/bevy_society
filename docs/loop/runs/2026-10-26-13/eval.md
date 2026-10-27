# Eval — a crowd damps the liking a witness forms (NOT SHIPPED, reverted)

Iteration `2026-10-26-13`. The change worked as designed and the design was wrong. This file records the
measurement, the two defects in my own premise, and why the tree is back where it started.

## The change, as built

`src/reputation.rs::witness_reaction` divided the **trust** a witnessed deed cost by
`witnesses^Tuning::witness_dilution` (0.5) and wrote **affinity** (`0.15 · approval`) and **respect**
(`0.1 · approval`) undiluted by every witness. All three now divide by the same `seen` factor.

Five tests in `tests/witness_dilution.rs`. Verified in both directions: with `src/reputation.rs` stashed back to
the old divides, the three dilution tests **FAIL** and the two identity tests pass; with the change, all five pass.

| test | old code | new code |
|---|---|---|
| `more_witnesses_means_less_trust_lost_by_each` | pass (already diluted) | pass |
| `more_witnesses_means_less_affinity_lost_by_each` | **FAIL** | pass |
| `more_witnesses_means_less_respect_lost_by_each` | **FAIL** | pass |
| `a_deed_seen_by_one_bystander_moves_the_tie_as_it_always_did` | pass (bit-identity) | pass |
| `a_witness_that_objects_still_objects` | **FAIL** | pass |

Two fixture defects found and fixed while building it, both of which would have made the tests lie:

1. **The first draft averaged the patient in with the witnesses.** `patient_reaction` moves trust and nothing
   else, so every figure came out at exactly **half** its true value — `0.0600` against the `0.1200` the write
   is worth. Fixed by skipping index 0 (actor) and index 1 (patient).
2. **The first draft measured five other affinity movers and called it the deed.** Mere exposure, relaxation,
   structural balance, the dyad balance and intergroup contact all nudge the same tie on the same ticks;
   together they were worth `+0.06` against the deed's `-0.12`. The fixture now zeroes all five so the change in
   the tie **is** the write under test, which is what makes the single-witness case assertable exactly.

## The before, measured — and it contradicts the file this iteration started from

`village-evening`, 17.03 game-days, three seeds, **before** build from a worktree at `d825bdd`:

| seed | friends | rivals | accepted compliments | shares | rumor reach | interrupted |
|---|---|---|---|---|---|---|
| 7 | **27** | **2** | 1 119 | 126 | 0.333 | 332 |
| 42 | **21** | **3** | 1 124 | 137 | 0.208 | 380 |
| 11 | **20** | **0** | 1 085 | 108 | 0.250 | 324 |

**The village is already friend-dominated, and by a wide margin.** `needs-decision/good-is-outweighed-by-bad.md`
states "**18 friend pairs against 42 rival pairs**, 6.5% friendly / 15% hostile" as the current state; that was
measured on 2026-12-28 and is **stale**. The affinity ledger it quotes (+160.4 from 1 604 compliments against
−241.2 from 7 920 reactions) is stale with it. Since that measurement the village has gained the quarter-loaf
meal, the shipped `bake`, the crowding price and the `snub` grudge gate, and the ratio inverted.

This is the second time in three iterations that re-reading the state beat the plan. It is also the reason the
change is reverted: **the problem it was built to fix no longer exists in the direction it assumed**, and at two
of three seeds there is almost nothing left to dilute.

## The after, measured — and why it is reverted

Same three seeds, same pack, same settings, only the two divides:

| seed | friends before → after | rivals before → after | shares | rumor reach |
|---|---|---|---|---|
| 7 | 27 → **39** | 2 → **0** | 126 → 67 | 0.333 → 0.333 |
| 42 | 21 → **30** | 3 → 1 | 137 → 82 | 0.208 → **0.583** |
| 11 | 20 → **43** | 0 → **0** | 108 → 146 | 0.250 → 0.333 |

- **Two of three seeds lose every rivalry.** Seed 7 goes 2 → 0 and seed 11 was already 0. A society with no
  hostility is the exact failure the crate has hit five times (`friendship-has-no-diminishing-return.md`): the
  network is bounded by the clamp, not by the model, and the village becomes a clique.
- **Friends rise 44% at seed 7 and 115% at seed 11** — 43 friend pairs in 24 people is a large share of the 276
  possible pairs. The clamp at `affinity_ceiling` 0.35 is what stops it going further, which is precisely the
  "friendship is a monotone function of contact volume" defect the clamp was introduced to paper over.
- **Sharing collapses at two seeds** (126 → 67, 137 → 82). A friendlier village trades less, which is a
  side-effect nothing in the literature predicts and nothing in the plan anticipated.

The mechanism behaved exactly as designed — the crowd stopped multiplying the write that shapes labels — and the
result is a *less* believable village, because the write it damped was the only thing still producing rivals.

## What the literature does and does not license

Sommerfeld 2007 (DOI `10.1073/pnas.0704598104`) does say a report from many observers moves each recipient less.
It says nothing about **which of three tie fields** carries that dampening, and the crate's choice to damp trust
alone was defensible: trust is the field `AcceptDef` reads, so dampening it there is what makes a village's
accept rate survivable, which was the measured defect the field was added for.

`good-is-outweighed-by-bad.md`'s own analysis says the affinity lever is `0.15 · approval` and had not been
measured as a lever. It is measured now, and it is a lever that works — in the direction that removes rivals.
The stale premise is what made that look like the right direction.

## Verdict of this iteration

**Nothing shipped.** The research note, this eval, and the plan stay as the record. `src/`, `tests/`,
`assets/` are byte-identical to `d825bdd`; `git diff` against it is this run's markdown only.

## The correction this iteration owes the repository

`needs-decision/good-is-outweighed-by-bad.md` must stop reporting 18 / 42 as the state; the measured state is
**27 / 2, 21 / 3, 20 / 0** at seeds 7, 42 and 11. That is written into the decision file as part of this run, and
it changes what the open question *is*: not "why does the village settle hostile" but "why does one seed in three
have no rivalry at all, and is 27 friends in 24 people too many".

# 2026-10-30-29 — plan: `ActivityDef::urgent`, a rank that follows a need

## The item

Hunger is not prepotent (pass 28's top item). Researched in `research.md`: half of the person-ticks at `food` < 0.05 are spent
on something other than the meal on the shelf; the Sims' need buckets and Maslow's prepotency say a low need should take the
choice, and the crate's dual-utility ranks are static.

## Design

1. **`ActivityDef::urgent: Option<UrgentDef { need, below, rank }>`** (`src/defs.rs`), `#[serde(default)]`, so no pack changes.
   Compiled to `ActivitySpec::urgent: Option<Urgent>` with the need resolved; `ActivitySpec::rank_for(levels)` is the activity's
   rank, raised to `urgent.rank` while `levels[need] < below`. Refused at load: an unknown need, `below` outside `(0, 1]`, an
   `urgent.rank` that is not above the activity's own (it would change nothing). No `Tuning` field: the number is the pack's.
2. **`decide_activities`** (`src/behavior.rs`): the tier test uses `rank_for(&needs.0)` instead of `spec.rank`. One indexed
   comparison per option; nothing else in the decision changes, and a pack without `urgent` is bit-identical to HEAD.
3. **The trace says why** (`src/trace.rs`): `ScoredOption::rank` is the tier the option was in, so a host asking why a person
   left a 0.93 `work` for a 0.82 `eat` reads rank 1 against rank 0; `Decision::describe` says so when the chosen option is not
   the best scorer because of its tier.
4. **The overlay takes it** (`examples/assets/provisioning.society.ron`): `eat` and `bake` carry `urgent` on `food`. `bake` has to,
   because on a bare shelf `eat` scores a fourteenth (priced, not gated) and the household that can make bread is the one that
   solves the hunger. `below` and the rank are design defaults, picked by the sweep in `eval.md`.
5. **The starter pack is not changed**, for the reason measured in `research.md`: on a pack whose `bake` is a clock-driven dawn
   errand a prepotent `eat` empties the shelves and makes the floor worse (19 -> 27 -> 52 %).
6. **The harness** (`examples/evidence.rs`): `floor.txt` (who is at the floor on `food`, what they were doing, whether a meal was
   on the shelf, the spells of hunger) and `--urgent BELOW` (put `eat`, and `bake` on the overlay, in the bucket on any pack).
   `tools/plot_urgent.py` draws the before/after.

## Not in this pass

`urgent` for `rest` (sleep already has rank 1 and a night window); an urgent *interruption* (a person already mid-`work` is not
pulled off it: activities end by duration and the decision runs on the end); flipping the starter pack
(`needs-decision/households-provision-themselves.md`); a need-bucket inferred from `satisfies` (rejected: `bake` satisfies
nothing and has to be in the hunger bucket on a bare shelf).

## Tests

- `tests/urgent.rs` (a small pack in RON, both feature sets): a hungry person takes `eat` every time while a fed one takes the
  higher-scoring `play`; no table to eat at falls through to `play`; `urgent.rank` equal to a static rank puts the two in one
  tier; the three load errors name the activity; a pack with no `urgent` decides as before.
- `tests/provisioning.rs`: the hungry bare household with an `urgent` overlay bakes **and** is not pulled off a meal it holds.
- The decision trace records the tier.
- Protected pins on the default pack are unchanged (the pack is not edited): `leaf`, `rumor_book`, `friendship_bound`,
  `label_hysteresis`, `starting_larder`, `need_cycle`, `economy`, `save`, and the readme/guide blocks.

## Evidence

Village (24) at seeds 7 / 42 / 11 / 99 and town (60) at seeds 7 / 99 on the overlay, with and without `urgent`, same seed; the
floor share, spells of hunger, bread a head, friends / rivals / insults / shares, and the default pack's byte-equality with HEAD.
Fresh eyes: a different seed and a 40-person town, and the library's own 240-second day. Three reviews in `review.md`.

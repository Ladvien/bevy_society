# Review: a meal is what was on the shelf

Three passes, each written before the next begins. Evidence is in `evidence/` and the numbers are in `eval.md`.

## Pass 1 — Behavior: does the change do what the paper and the scenario say?

**Claim.** An activity's `satisfies` is its **satiation** term, and satiation follows what was eaten, not the time
spent at the table. Zandstra 2000 (DOI `10.18174/197257`), verified on disk at
`~/.cache/bevy_elements_corpus/markdown/10/10.18174_197257.md:207`: *"Satiation is the process that terminates
the eating within a meal, and influences the size of meals and snacks. Satiety is the state that occurs after
eating ... and inhibits further eating, which influences the frequency of meals."* The crate implemented the
duration faithfully and the amount not at all.

**The defect, re-derived rather than taken from the earlier pass.** From the price run's own committed series
(seed 23, 24 people): **4.80 loaves baked against 21.06 completed meals** a game-day, and **23.2% of ticks with
no bread anywhere**. Every one of those meals paid its full `satisfies`. That is not a tuning gap; two systems
described one event differently.

**The mechanism tests bite.** `tests/economy.rs` now pins four shapes and each fails against the old code:

| test | what it would catch |
|---|---|
| `a_meal_from_an_empty_shelf_feeds_nobody` | the defect itself: `food` stays at 0 with no bread, and the store is still debited to nothing (nobody was blocked) |
| `a_fed_meal_pays_in_full` | a change that broke the fed case: one loaf feeds and is consumed |
| `half_a_loaf_feeds_half` | a threshold smuggled in where the data gives a linear fraction: ratio `0.5 ± 0.06` |
| `an_activity_that_consumes_nothing_is_unaffected` | collateral damage: an activity with no `consumes` still pays in full from an empty store |

**The end-to-end numbers, two populations.** Seed 23/24: `food` **0.450 → 0.339**, ticks with no bread
**23.2% → 3.8%**, price 1.575 → 0.841, friends **26 → 26**, rivals 5 → 2, reach 0.42 → 0.46. Fresh eyes,
seed 42/16: `food` **0.416 → 0.330**, no-bread ticks **9.3% → 4.1%**, friends 16 → 20, rivals 1 → 0.

**`food` going *down* is the point, not a regression.** The village was fed by arithmetic; it is now fed by 4.0
loaves a bake. A mean that stays at 0.45 while a quarter of the run has an empty larder is the number lying.

**One thing this pass found that was not the plan, and it is larger than the plan.** The producer `bake` lived
only in `examples/assets/village.society.ron`, never in `assets/starter.society.ron` — so the pack the crate
ships as a host's starting point had `eat` consuming a good **nothing produced**. It was invisible for as long
as the crate has had goods, because a meal from an empty shelf was paid in full. It is fixed (the activity moved
to the starter pack) and now warned about (`warn_unmade_goods`, `tests/unmade_goods.rs`).

**Verdict: GREAT.** The model is the paper's, the defect is closed at both populations, and the shipped pack
gained the producer it always needed and a warning that says so when a host's own pack lacks one.

## Pass 2 — Zoom: read the plots closely

`evidence/plots/food.png` (seed 23) and `food-fresh.png` (seed 42), three panels on one x-axis each, before red
and after blue, with the **before** run's famine spans shaded so the same band is visible behind both lines.

- **The famine band is where the change is.** Days 10.5–13.5 at seed 23 are a 3.4-game-day stretch with no bread
  at all in the before run. The red line is *flat and high* through it (0.45–0.60), because the meals kept being
  paid. The blue line is not: it falls into the band and recovers on the next bake. The shaded region is not
  present in the after run at all — the village keeps bread on 96% of ticks.
- **Nothing clips.** The blue line reaches 0.0 once, on day 0, and that is the run's own start: `Needs` begins at
  the pack's `initial` of 1.0 and falls at 0.9/day while the first bake is at hour 6. It is not a clamp firing
  and it is not repeated in the body of the run (minimum over the last 17 days: 0.015).
- **The step shape of the store is correct, not an artefact.** `bread_mean` moves in steps because a bake lands
  three-to-four loaves at once and eating draws them down. The red line's steps go *down* through the famine and
  stop at exactly 0.0 — the arithmetic still feeding people from nothing.
- **The price is never at its floor.** The after run's price reaches exactly `0.50` on 40% of ticks, and that is
  `v(1) = 0.5`, the **full-store** value, not the `price_floor` of 0.25. Verified arithmetically: every tick at
  0.50 has `bread_mean ≥ 1.0`. The first implementation of this iteration's pack (6 loaves a bake) sat at 0.50
  on **62%** of ticks and lost friends 26 → 21; that is what moved the calibration to 4.
- **The third panel is the honest one.** `before − after` starts near 0, rises to +0.57 while the village is
  genuinely short, crosses zero around day 8, and oscillates within ±0.2 after. That is a change that matters
  where food is short and does nothing where it is not — which is the shape the design predicted.

**A finding the plot pass produced and the plan did not anticipate:** the supply does **not** scale with the
population. Bakes a game-day are **3.3 at 16 people and 3.9 at 24**, because the dawn window is one hour long
and the number of people who fit an errand into it is what limits the supply. Raising loaves a bake 3 → 6 barely
moved bakes a day (3.28 → 3.91). Recorded in the pack beside the number, because a host scaling its population
must scale `produces` and cannot expect the hour to do it.

**Verdict: GREAT**, with the calibration history kept rather than tidied away — the 6-loaf glut is in `eval.md`
because it is the measurement that chose the shipped number.

## Pass 3 — Fresh eyes: a different seed and population

Seed **42**, **16 people**, same binary, same pack, no per-run tuning: `evidence/fresh-before/s42_p16` and
`evidence/fresh-after/s42_p16`.

| | before | after |
|---|---|---|
| mean `food` | 0.416 | **0.330** |
| ticks below 0.1 `food` | 0.0% | 6.2% |
| ticks with no bread anywhere | 9.3% | **4.1%** |
| mean bread per person | 0.416 | **0.817** |
| friends / rivals / shares | 16 / 1 / 39 | **20 / 0 / 73** |

The shape survives: the store holds above zero for the same 96% of ticks, the price sits near nominal (0.895),
and `food` falls by a comparable amount. The fresh population is *smaller* and the same 4-loaf bake oversupplies
it slightly less than at 24 — consistent with the flat supply the plot pass found.

**The one number that does not improve is `food` itself, and that is the honest limit of this iteration.** The
village now feeds itself from what it bakes and the bakes are calibrated for 16–24 people; a host with 200
people gets the same 3.9 bakes a day. That is now stated in the pack, in `docs/guide/authoring.md` and in
`eval.md`, rather than discovered by a host.

**Can a host developer be told this without reading the code?** Yes: *"An activity that consumes a good now pays
its needs in proportion to what the store actually had — a meal with no bread costs the walk and feeds nobody,
and half a loaf feeds half. Activities that consume nothing are unchanged. If your pack consumes a good nothing
produces, the crate tells you at startup."*

**Verdict: GREAT.**

## What this review corrects, and what it records as still wrong

0. **A test that failed, and it was a fixture built on the old lie.** `tests/reciprocity.rs`'s
   `the_village_keeps_both_labels` ran 4 200 ticks — a little over one game-day on a village that bakes only at
   first light. Under the old payout that window was fed anyway and rivalries formed over meals nobody ate;
   now rivals are **0 at 4 200**, **0 at 8 400** and **8 at 12 600**. The run was extended to three game-days
   with the arithmetic in its comment. **The assertion did not change** — it still demands both labels,
   mutual and one-way ties and a gap under 0.10 — so this is the fixture reaching the state it claims to
   measure rather than a bar being lowered. Recorded here because a failing test is the first place to look for
   a real regression, and this one needed the distinction drawn explicitly.

1. **Fixed in passing:** `src/economy.rs`'s module header, `docs/guide/authoring.md`,
   `docs/research/economics.md` and `docs/research/needs-and-utility.md` all said "an empty store never stops
   anyone eating" as though that were the whole truth. It is half of it, and each now says the other half.
2. **Recorded, not fixed: `DefsFile::merge` does not carry `goods`.** Found while moving `bake`: a packed merged
   in with a new good silently has none. It is a one-line omission beside the eleven that are carried, but it
   changes what a merged pack means, so it is the next iteration's or a `needs-decision/`, not a drive-by.
3. **Recorded, not fixed: `Tuning::trade_reserve` (1.0) was sized when a meal was a whole loaf.** Now that a meal
   is a quarter, a reserve of a whole loaf is four meals. It did not change the shipped numbers at 24 people
   (shares went 29 → 110 for the reasons in `eval.md`), but the number no longer means what its doc says.
4. **Kept on disk and labelled:** the 6-loaf and 3-loaf calibration runs (`~/.cache/food-sweep/`) are not in the
   repo; their numbers are in `eval.md`'s table.

## Verification on the final tree

Both feature sets (`cargo test --release`, `--no-default-features`), `clippy --all-targets --all-features
-- -D warnings`, `fmt --check` and `doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`: all exit 0.
The counts are in `eval.md`.

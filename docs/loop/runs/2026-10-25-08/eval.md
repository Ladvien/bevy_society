# Evaluation: goods change hands

Instruments: `tests/trade.rs` for the mechanism; `examples/evidence.rs` at `village-evening`'s own settings
(24 people, 64 s/day, 17.03 game-days) for the village, **nine seeds** (1-6, 7, 42, 11) per configuration,
one process per seed, the same binary for every row of a table.

## What was built

- **`actor_gives` / `target_gives`** on an interaction's `on_accept`: lists of `(good, amount)`. Both sides move in
  one call; each gives what it has, so a pair cannot create a good.
- **`Input::Surplus(good)`**: the joint surplus of a transfer, `value(target) − value(own)` with
  `value(held) = 1 / (1 + held)`, zero unless the giver holds **more than** `Tuning::trade_reserve` (1.0) and the
  gap is at least `Tuning::trade_margin` (0.1).
- The village pack: a `share` activity scored by `Surplus("bread")`, `bake` producing **3** loaves and weighted by
  `Skill("craft")` so only the diligent do it.
- `Decision` copies the neighbours' stores **only when some activity reads `Surplus`** (nothing else pays).

## The mechanism, in `tests/trade.rs` (7 tests, all green)

| test | asserts |
|---|---|
| `a_unit_is_worth_more_to_whoever_has_less` | `value_of` is strictly decreasing and positive |
| `a_trade_moves_one_unit_to_the_person_who_has_none` | two loaves and none level to one each, giver keeps the reserve, total conserved |
| `two_equal_stores_do_not_trade` | no surplus, no transfer |
| `a_giver_at_or_below_the_reserve_does_not_trade` | half a loaf and exactly one loaf (the reserve) move nothing |
| `a_barter_moves_a_good_each_way_and_conserves_both` | two goods, each side gives and receives, both totals conserved |
| `trade_narrows_the_spread_of_stores` | one holder of four loaves, three with none: standard deviation under 60% of the control's, total unchanged |
| `trade_is_deterministic` | two identical runs trade identically |

## A bug found by reading the log as a stranger

The first gate was "the giver holds at least the reserve". `shares.txt` (one line per loaf, giver, receiver, both
stores) then read:

```
d2 10h35  Vesna -> Dena   (now Vesna holds 0.0, Dena holds 1.0)
d2 16h45  Ysolde -> Quill (now Ysolde holds 0.0, Quill holds 1.0)
```

Villagers were giving away the last loaf they had. That is generous and not what Henrich et al. report — the meat
that is shared is the **surplus** — and it made `Tuning::trade_reserve`'s own doc ("how much a person keeps") false.
The gate is now **more than** the reserve, so a whole-loaf giver is left with at least one. The effect, nine seeds:

| | shares in 17 game-days | spread of bread (sd), days 2-17 | largest store |
|---|---|---|---|
| no `share` (control) | 0 | 0.854 | 3.17 |
| gate "at least the reserve" | 94.9 | 0.529 | 2.01 |
| **gate "more than the reserve" (shipped)** | **37.1** | **0.529** | **1.97** |

**Fifty-eight fewer shares and the same levelling**: the extra gifts were last loaves handed to a neighbour who ate
them, and levelled nothing. The evidence harness now logs each share as `before -> after` for both sides, because
"Dena holds 0.0, Hilda holds 0.0" (a loaf that arrived and was eaten in one tick) could not be told from a failed
transfer.

## The village, nine seeds

The first plan said "the four regression numbers do not move" and compared three seeds. **That is not a valid
test**: any change to the decision path reshuffles the trajectory, and at the *baseline* the seed-to-seed spread
is friends 2.9, rivals 5.8, rumor reach 0.23, interruptions 27 (standard deviations). Seeds 7 / 42 / 11 moved
rivals by +8 / −2 / −13 on the first, one-loaf version of this change, and a six-seed sweep of the same two
binaries showed the *means* equal. The comparison that means something is the mean of a paired sweep.

Before is the commit this branched from (`37d80e8`); after is the shipped pack. `evidence/sweep.md`:

| seed | friends (before → after) | rivals | rumor reach | interrupted endings | shares |
|---|---|---|---|---|---|
| 1 | 15 → 22 | 21 → 24 | 0.292 → 0.333 | 299 → 304 | 30 |
| 2 | 10 → 16 | 23 → 11 | 0.125 → 0.125 | 292 → 300 | 37 |
| 3 | 17 → 13 | 26 → 26 | 0.125 → 0.208 | 368 → 321 | 40 |
| 4 | 17 → 23 | 11 → 26 | 0.083 → 0.083 | 318 → 324 | 34 |
| 5 | 20 → 18 | 11 → 21 | 0.167 → 0.250 | 302 → 342 | 34 |
| 6 | 15 → 17 | 10 → 10 | 0.458 → 0.125 | 360 → 302 | 36 |
| 7 | 16 → 23 | 24 → 17 | 0.792 → 0.083 | 335 → 311 | 40 |
| 42 | 12 → 16 | 17 → 18 | 0.083 → 0.042 | 300 → 338 | 41 |
| 11 | 18 → 21 | 21 → 13 | 0.083 → 0.125 | 302 → 330 | 42 |
| **mean of 9** | **15.6 → 18.8** | **18.2 → 18.4** | **0.245 → 0.153** | **320 → 319** | **37.1** |

Paired differences, mean ± standard error over 9 seeds: friends **+3.2 ± 1.2**, rivals **+0.2 ± 2.7**, rumor reach
**−0.093 ± 0.083**, interrupted endings **−0.4 ± 11.2**. Rivals and interruptions are level. **Friends are up, at 2.6
standard errors**: a share moves trust 0.05 each way and friendship needs trust. **Rumor reach is down 38%, at 1.1
standard errors**, which is inside the noise — and the baseline's own value is pulled up by one seed (seed 7, 0.792)
that the same seed no longer repeats. It is reported as it is, not as a win.

**The build is deterministic across builds.** The renamed `share` pack reproduced the previous build's nine
summaries exactly on every number but the key names, on a different build of the example.

## What got it here: six configurations, same nine seeds

| configuration | friends | rivals | rumor reach | interrupted | shares | spread (sd) |
|---|---|---|---|---|---|---|
| before (HEAD) | 15.6 | 18.2 | 0.245 | 320 | - | - |
| `share`, 1 loaf a bake, slope 1 | 16.4 | 17.2 | 0.236 | 333 | 19.8 | 0.183 |
| 2 loaves a bake | 17.7 | 15.4 | 0.366 | 329 | 59.7 | 0.446 |
| 3 loaves, `Skill("craft")`, slope 1 | 17.8 | 18.3 | 0.218 | 315 | 65.0 | 0.577 |
| 3 loaves, `Skill("craft")`, slope 2, gate "at least" | 17.1 | 16.3 | 0.301 | 331 | 94.9 | 0.529 |
| **shipped: same, gate "more than the reserve"** | 18.8 | 18.4 | 0.153 | 319 | **37.1** | **0.529** |

At one loaf a bake **a loaf was eaten as fast as it was baked**: the village averaged 0.05 of a loaf a person and
shared 20 times in 17 days, so there was nothing to trade (and a low spread is not a good sign there — it is a
village with no bread to be uneven about). Three loaves from a few hands leaves the baker a surplus, which is also
the first division of labour in the pack: the diligent bake and the rest receive. Slope 2 on `Surplus` lifts a loaf
to an empty shelf from a 0.5 score to 1.0 so a share is chosen more often than once in thirty decisions.

## Does sharing do anything? Same pack, `share` removed

`evidence/plots/spread.png`, nine seeds,
days 2-17, the control being the same binary built without the `share` activity (`evidence/control/`):

| | no `share` (control) | with `share` (shipped) |
|---|---|---|
| spread of bread across villagers (sd) | **0.854** | **0.529** (−38%) |
| largest store in the village | 3.17 | 1.97 |
| mean store | 0.384 | 0.241 |
| loaves shared | 0 | 37.1 |

**Sharing levels the village** — the biggest store falls by 38% and the spread by 38% — **and the mean store falls
too**, because loaves that reach a person with an empty shelf are then eaten. That is the economic effect: bread
reaches the eaters instead of piling up with the bakers.

## The plan's price claim was wrong

`plan.md` predicted that "the price stops running to its clamp once trade is possible", from Parunak et al.'s
"in repeated trades, balances supply and demand". **It does not, and it could not have.** A transfer neither
produces nor consumes, so `Δprice = speed · (produced − consumed)` is untouched by it directly. Measured over the
same nine seeds, days 2-17:

| | share of ticks at the **floor** (0.25) | at the **ceiling** (4.0) | mean price |
|---|---|---|---|
| before this change, 1 loaf a bake | 1% | 0.1% | 1.61 |
| 3 loaves, no `share` | 21% | 4.9% | 1.69 |
| 3 loaves, `share` (shipped) | **28.5%** | 3.8% | 1.47 |

Two things are true at once. **Sharing does not stabilise the price.** And **three loaves a bake made the price
swing across its whole band**: `Δprice = 0.5 × produced`, so a three-loaf bake moves it by 1.5 against a range of
3.75, and the price spends over a quarter of the run on the floor. At one loaf it hardly touched a clamp. The price
is not scale-invariant: it moves by flow *units*, not by flow per head, so a bigger bake or a bigger village swings
it harder (the 5 000-person benchmark declares no goods and would not show it). That is a defect in last
iteration's model that this iteration's pack exposes, and reading `produce_and_consume` to explain it found a worse
one: `consumed` is what `Stock::take` *removed*, so `produced − consumed` is the net change in what people hold.
**The price rises when bread piles up**, the wrong way round, and a real shortage is invisible because an eater
with an empty shelf takes nothing. The sign was flipped last iteration to make the shortage case of
`tests/economy.rs` pass, and the reason given there describes this artefact. Repairing it means counting unmet
demand and scaling by head, which needs its own research and measurement; it is recorded as the first remaining
item rather than fixed inside a change about trade, and `docs/research/economics.md` and the README both say so.

## What is not good enough

1. **The price is not scale-invariant and follows net stock** (above). The village's price saturates both clamps.
2. **The village has one good, so what it does is sharing, not barter.** The engine does a two-way swap and tests it;
   the pack has no second good because nothing would consume it.
3. **A share is rare**: 37 in 17 game-days, about 2 a game-day village-wide, and 3 in the six game-hours of the
   clip (which opens with bread handed out; the HUD says so).
4. **The giver is paid only in trust and emotion**, not in standing or a debt that is later repaid.
5. **Rumor reach is down at 1.1 standard errors**, and the baseline value it falls from is one outlier seed.
6. **The reserve is read when a share is chosen, not when it resolves.** The pair has to meet first, and the giver
   may eat on the way: 1 of 76 logged shares (`seed99_36people`, `d7 19h27`) left a giver at `1.0 -> 0.0`. Closing it
   would make every `actor_gives` respect a reserve, which a gift of a quest item should not.
7. **Rivalries explode with population at the baseline** (18 at 24 people, 183 at 36, seed 99). Found, not caused here.

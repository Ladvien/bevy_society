# Review: goods change hands

Three passes, each with a written critique before the verdict. Evidence is cited by file under `evidence/`.

## Pass 1 — Behavior: does it do what the paper and the scenario say?

**Claims to check.** (1) A good moves only when there is a joint surplus (Hall & Milgrom as reported by Hall 2005,
`10.3386/w11678`). (2) The exchange is pairwise and needs no auctioneer (Parunak et al., `10.1007_11678809_10`).
(3) Repeated trades balance supply and demand, which the plan read as "the price stops running to its clamp".

**Critique.**

- (1) holds, and is asserted rather than eyeballed: equal stores do nothing, a giver at or below the reserve does
  nothing, and a holder of two loaves facing an empty shelf levels to one each (`tests/trade.rs`). `shares.txt`
  shows the same on the village: every line is a whole loaf from a holder of 2-4 to somebody with none, e.g.
  `d8 08h33  Kell -> Edric  (Kell 4.0 -> 3.0, Edric 0.0 -> 1.0, over this tick)`
  (`evidence/narratives/seed7/shares.txt`).
- (2) holds by construction: `Perceive` picks the pair, the acceptance draw decides, no market place exists.
- (3) **failed, and could not have passed.** A transfer neither produces nor consumes, so `produced − consumed` is
  untouched. Measured, the price spends 28.5% of its ticks on the floor with sharing and 21% without
  (`evidence/plots/spread.png`, bottom left; `eval.md`). What *is* true is that stores level: spread of bread
  0.854 → 0.529 and the largest store 3.17 → 1.97 over nine seeds.
- **A bug the numbers hid, found by reading the log.** The first gate let a villager give away the last loaf
  ("Vesna holds 0.0"), which made the `trade_reserve` doc false. Fixing it cut shares from 94.9 to 37.1 **and left
  the levelling unchanged**, which also says the extra 58 gifts were doing nothing.
- **A race the fix leaves.** The gate is read when a share is *chosen*; the exchange resolves on arrival, and the
  giver may have eaten in between. One logged share in 76 left a giver at `1.0 -> 0.0`
  (`seed99_36people/shares.txt`, `d7 19h27 Rhea`). Documented, not closed: closing it means making every
  `actor_gives` respect a reserve, which a quest-item gift should not.

**Verdict: GREAT** for what the change claims (surplus-gated transfer, conservation, levelling). The price
prediction in `plan.md` was wrong, is recorded as wrong, and is not claimed.

## Pass 2 — Zoom: read the plots closely

`evidence/plots/spread.png`, four panels, nine seeds.

- **Spread (top left).** Control sits at 0.8-1.0 from day 2; sharing sits at 0.35-0.65, never touching it. The IQR
  bands touch only briefly around days 6-8, where the control dips. No oscillation, no drift in the gap, no convergence to a
  constant: the sharing line wanders, which is what a village with a daily bake looks like.
- **Largest store (top right).** 3.0-3.6 against 1.1-2.3. Sharing is **not saturating at the cap** (`stock_ceiling`
  6.0), and the control never reaches it either.
- **Cumulative shares (bottom right).** Flat for the first 1-6 game-days (stores have to fill: nobody has more than
  one loaf early), then a steady climb, 30-42 per seed, which is the seed-to-seed consistency the 95-share version
  lacked (it ran 48-149). No spike at a tick boundary.
- **Price (bottom left).** **Clipping at both clamps, in both lines.** Floor 21% / 28.5% of ticks, ceiling 4.9% /
  3.8%. This is the finding: it is a saw-tooth of ±0.5-1.5 steps from lumpy production, in the control as much as
  with sharing, so sharing neither causes nor cures it. The cause is last iteration's price model (unit-valued, and
  it follows net stock, so it rises when bread piles up). **Not fixed here**: it needs unmet demand counted and its
  own research, and it is the first remaining item. The README and `economics.md` now say so.
- **Dead agents / drift.** The 40 shares at seed 7 come from 15 distinct givers, the busiest giving 6 times in 17
  game-days and nobody more than twice in a day: the bakers share, and no one villager is the village's pantry.

**Verdict: GREAT on the change, with the price defect recorded as pre-existing and separately owned.** I considered
marking this NOT YET because the plot shows clipping; it is not this change's clipping — a one-loaf pack clipped 1%
of the time, a three-loaf pack 21% *without* sharing — and the pack change that exposed it was needed to give
sharing anything to share (20 shares in 17 days at one loaf).

## Pass 3 — Fresh eyes: another seed, another population, a stranger's reading

`evidence/narratives/seed99_36people/` (36 people, seed 99), and `seed7/shares.txt` read cold.

**Stranger's read of `shares.txt`.** Each line says who gave to whom and both stores before and after; every one is
a whole loaf from 2-4 to 0. A host developer can explain it in a sentence: *"somebody with a spare loaf, beside
somebody with none, sometimes hands one over, and both are a little more trusting after."* Nothing needs the code.

**At 36 people it still behaves.** 36 shares in 17 game-days (the 24-person mean is 37), bread spread 0.164 against
0.515 in the no-share control at the last tick, friends 23 against 16.

**Honest side findings, none caused by this change:**

| 36 people, seed 99 | friends | rivals | rumor reach | interrupted |
|---|---|---|---|---|
| before this iteration | 24 | **183** | 0.306 | 649 |
| same pack, no `share` | 16 | 126 | 0.194 | 715 |
| shipped | 23 | 138 | 0.056 | 695 |

- **Rivalries explode with population**: 18 at 24 people, **183 at 36** at the *baseline*. That is not this
  iteration's, it is the largest believability problem this run found, and it is in the log as a remaining item.
- Rumor reach at 36 people is 0.056 against 0.306 at baseline in this one seed. The nine-seed paired difference at
  24 people is −0.093 ± 0.083, inside the noise; one 36-person seed proves nothing either way.

**Verdict: GREAT.** The mechanism reads the same at another seed and a 50% larger village, and the narrative can be
explained without reading the code. What this pass found about rivalry scaling is recorded, not blamed on sharing.

## The GIF

`docs/media/goods.gif`, frames 0 / 80 / 156 / 239 inspected, plus all six older clips as a contact sheet (frames
0 / 120 / 239 each). **Readable:** gold rings, small dark rings, a white line with a ring on the receiver, and a HUD
that says `5 loaves shared · 17 hold bread, 7 do not` against `0 loaves shared · 12 hold bread, 12 do not` at the
start. **Not a flattering clip**: a share is a half-second line, so the HUD counter is what carries it. The six
older clips re-recorded to bit-identical frames after the final change (md5 of frames 0 / 120 / 239 equal), so they
were not re-encoded.

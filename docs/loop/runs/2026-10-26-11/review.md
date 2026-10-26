# Review: a price read off the stores, per head

This review was written by the **next** pass, not the one that built the change: pass 11 ended with its work
uncommitted and the driver saved it as `WIP(pass 11)`. Everything below was re-derived from the committed
evidence rather than taken from `eval.md`, and the one thing the pass never did was the three-pass review
itself, so this file is that review.

## Pass 1 — Behavior: does the change do what the paper and the scenario say?

**Claim.** A price should be a scarcity signal: the same store is dearer for more mouths, an empty store is
dearer than a full one, and a village whose inflow equals its outflow holds a steady price. The old
`Δprice ∝ (produced − consumed)` failed all three, and the corpus's own game-economy model supplies the
replacement (`eth-cgl-ar_games-Zun16a` §3.1.1).

**The paper was checked against the source, not the note.** The markdown for `eth-cgl-ar_games-Zun16a` (read
on disk at `~/.cache/bevy_elements_corpus/markdown/et/eth-cgl-ar_games-Zun16a.md`, lines 129 and 157-183)
carries both halves of the citation verbatim:

> "The price modifier term v(r) doubles the price if the storage is empty and halves the price if the storage is
> full." (line 129)

> "The resource price p is directly calculated from the storage ratio r = S_cur/S_max, where S_cur is the number
> of resources currently in storage and S_max is the storage size." (line 157)

and the formula `p = w·v(r) + k`, `v(r) = −2(r−1)` for `r < 0.5`, `1.5 − r` otherwise. `src/model/price.rs`
implements `v` **character for character**, and its three named points are the paper's own sentence
(empty → 2.0, half → 1.0, full → 0.5). The dropped terms are dropped for stated reasons: `w` is an outside
market feed the crate must not have, `k` is ±10% noise determinism forbids.

**The defect is real and quantified, and I re-derived every number from the committed CSVs** rather than
trusting `eval.md`:

| | before | after |
|---|---|---|
| correlation of price with mean bread per person | **+0.598** | **−0.997** |
| ticks on the floor (0.25) | 25.3% | 0% |
| ticks on the ceiling (4.0) | 6.0% | 0% |
| price standard deviation | 1.214 | 0.292 |

The sign is the result: the old price **rose as bread piled up**. A scarcity index with the wrong sign is not
a tuning problem.

**The per-head requirement is a test, not a claim.** `the_same_store_is_dearer_per_head` asserts six loaves for
one person is `0.5` and for thirty is `1.6`. No unit-counting formula can express that.

**Verdict: GREAT.** The model is the paper's, the defect's sign is repaired, and the two recorded defects from
four iterations of log entries (net stock; units not heads) are both closed by the same substitution.

## Pass 2 — Zoom: read the plots closely

`evidence/plots/price.png`: price (top) and the store it is read from (bottom), same seed, same pack, one pair
of axes, before in red and after in blue.

- **Red walks the whole band** 0.25–4.00 and is visibly **anti**-correlated with the store in places: at day
  ~1.5 the store falls and the price also falls. At day ~3.5 the store peaks and the price is at the ceiling.
  That is the defect drawn.
- **Blue tracks the store inversely at every visible turn** and never touches either clamp.
- **Nothing clips, saturates at 0 or 1, or drifts.** Checked arithmetically, not by eye: the only ticks at
  exactly the empty-store value 2.0 are the 4 622 whose `bread_mean` is **exactly 0.0**. That is starvation
  reported, not a clamp firing — a distinction the plot alone cannot make, which is why it is measured.
- **The step shape is correct, not an artifact.** Price is a function of state, so it moves when the store
  moves and is otherwise flat. A flow integral would drift; this cannot.

**A finding the plot pass did not state, and it is the one a host will hit.** With `price_full_store = 1.0`
the village's median price is **1.50**: the default declares a village holding a quarter of a loaf a head to be
*permanently short*, because a day's supply a head is 1.0 and it holds 0.25. Recomputed directly from the store
series at four defaults:

| `price_full_store` | median price | min | max |
|---|---|---|---|
| 0.25 | 0.50 | 0.50 | 2.00 |
| 0.50 | **1.00** | 0.50 | 2.00 |
| 1.00 (shipped) | 1.50 | 0.79 | 2.00 |
| 2.00 | 1.75 | 1.29 | 2.00 |

At `0.5` the observed store sits on the nominal unit, which is what "half full is nominal" is for. This is not
a bug — "how many days does the village have?" is a legitimate question and the answer here is a quarter — but
the shipped default reads the village as always hungry and never once reaches the glut end. It is recorded as
the top remaining item rather than re-tuned, because changing it moves every price in every host's game and
deserves its own before/after.

**Verdict: GREAT** for the change; one honest limitation recorded above and not smoothed over.

## Pass 3 — Fresh eyes: a different seed and population

**The pass's own fresh-eyes run was wrong and I redid it.** `evidence/fresh/s19_p36/` runs **36 people in a
30-seat village** (the place capacities in `examples/evidence.rs::village_with` are 6 + 10 + 8 + 6 = 30). The
loop log records that this exact configuration has produced **two false results** that reached the log before
being caught, and here it shows: bread per head is **0.028** at 36 people against **0.208** at 24, so the
"fresh" run spends its life at the empty-store end (17 570 of 69 631 ticks at 2.0) and demonstrates scarcity at
a density the pack was never authored for.

The honest fresh-eyes run is therefore a **new seed at a population the village can seat**: seed 23, 24 people,
17 game-days, written to `evidence/fresh24/s23_p24/`. Same binary, same pack, no per-run tuning. The shape must
survive it: price falling when the store fills, no clipping at either end, and a day cycle rather than a random
walk.

**Can a host developer be told this without reading the code?** Yes: *"The price of a good is now a reading of
how full your village's stores are, per person — an empty store is worth double, a full one half, half full is
the nominal unit. The same six loaves are dearer for thirty villagers than for three. It is a function of the
world rather than an accumulator, so it cannot drift, and your scripts can read it with `ctx.price("bread")`."*

**The seated fresh run passes the shape test, and shows a famine the old price hid.**
`evidence/plots/price-fresh24.png`: correlation **−0.9976**, range **0.79 – 2.00**, standard deviation 0.342,
and the 16 719 ticks at exactly 2.0 are again **all** ticks whose `bread_mean` is exactly 0.0. The price falls
whenever the store fills and rises whenever it empties, with no clipping at the full-store end.

The panel also makes visible something no earlier plot could, because the old price was not a reading of
anything: **the village holds zero bread for 23.9% of its ticks**, in runs as long as **3.39 game-days**
(days 10.30–13.69). The price reports that state exactly right — it is the one number in the crate that now
does. But a village where a quarter of all ticks are a famine is a **behaviour** fact, not a pricing one, and
nothing reads the price to react to it. Recorded as the top remaining item.

**Also verified here, and it is the check that this is one change:** the whole society is bit-identical apart
from `price`. `summary.json` matches field for field except `price` (0.500 → 1.583) and the two `narrative.txt`
files are **byte-identical**. No decision reads a price, so nothing else may have moved — and nothing did.

**Verdict: GREAT** for the change, with the pass's `s19_p36` panel marked as an overfilled village rather than
used as evidence, and the seated `fresh24/s23_p24` run carrying Pass 3.

## What this review corrects

1. **The pass's fresh-eyes run is an overfilled village.** `eval.md` never presented it as a verdict — the
   run wrote no `review.md` at all, which is the gap this file fills — but the panel existed as evidence and a
   plot. Kept on disk, labelled here, and superseded by `fresh24/s23_p24`.
2. **`tests/economy.rs`'s module header** still said the price "climb[s]" in a starved village — the old model's
   premise. Restated.
3. **`scenarios.yaml`'s `village-sharing` entry** still called the 28%-on-the-floor price an open defect. Updated.
4. **`README.md`** had a doubled "and ... and" from the edit. Fixed.

## Verification on the final tree

Recorded in the run `log.md` entry: both feature sets, `clippy -D warnings`, `fmt --check` and
`doc -D warnings`. The test inventory is in `eval.md`.

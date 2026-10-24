# Should a rumor that nobody can tell again be retired?

## The problem, measured

`RumorBook.rumors` only ever grows. A rumor's `alive` flag is cleared in exactly one place —
`retire_about`, when its **subject despawns** — so a rumor whose every knower has stifled (Daley &
Kendall) stays `alive = true` forever.

Measured on 200 people on one plaza, 3 000 ticks:

| | count |
|---|---|
| rumors in the book | **992** |
| `alive` | **992** |
| still tellable by at least one person | **70** |
| untellable | **922 (93%)** |

**Correction 2026-11-14: the tick-cost claim below was wrong.** It said the per-tick walk over a growing
book was a **4× blowup** after 3 000 ticks. Measured directly — 200 people, four places, five samples of
100 ticks across a 6 000-tick run, with and without a `RumorReach` trigger:

| tick | rumors | ms/tick, no trigger | ms/tick, with a trigger |
|---|---|---|---|
| 1 000 | 765 | 5.76 | 5.95 |
| 3 000 | 2 412 | 3.51 | 3.07 |
| 5 000 | 4 299 | 4.80 | 3.93 |

The cost **does not grow**; it drifts down as the village settles. The 4× figure came from comparing a
short benchmark against a long one and attributing the whole difference to the book. `tests/rumor_book.rs`
now pins the truth.

What the book does cost is **memory**, at about 48 bytes a rumor: 105 KiB at 3 000 ticks and 228 KiB at
6 000 for 200 people, and roughly 5 MiB for 5 000 people over a comparable run.

That changes the urgency rather than the recommendation. A 4× tick blowup would justify a breaking
semantic change; 5 MiB over a long large run does not. The second half of the original claim holds: on a
3 000-tick run **87%** of the book is untellable (301 of 2 250 tellable), so a retirement rule is still
the right fix for memory.

## Why it is a decision and not a bug fix

`alive` has a documented meaning — "the subject is still in the world" — and three places read it:
`best_telling`'s eligibility, the `RumorReach` trigger, and `retire_about`. Changing *when* it is
cleared changes what a host sees when it reads `RumorBook`, and changes whether a `RumorReach` trigger
can fire late.

## Options

### 1. Retire when no living spreader holds the rumor

After the gossip step, clear `alive` for any rumor with no `Knowledge` entry where `spreader` is true.

- **For:** matches Daley–Kendall exactly — a rumor with no teller cannot spread, so `alive` becomes
  "can still circulate", which is what every reader already assumes. The scan is O(persons × knowledge)
  once per tick, which is cheaper than the O(rumors) walk it removes.
- **Against:** a host that holds a `RumorId` past its retirement now gets `alive == false` sooner; and
  a `RumorReach` trigger cannot fire on a rumor that died before reaching its fraction (which is
  correct, but is a behaviour change).

### 2. Drop retired rumors from the `Vec`, renumbering ids

- **For:** memory actually falls.
- **Against:** every `RumorId` in every `Knowledge`, memory and trigger binding shifts. This breaks
  the host-facing id contract and the script API. **Rejected.**

### 3. Leave `alive` alone; skip untellable rumors in the `rules.rs` walk only

- **For:** no semantic change at all; the cost is what motivated this.
- **Against:** does not bound memory, and the walk still has to *determine* tellability, which needs
  the same scan option 1 does.

## Recommendation

**Option 1**, with the scan folded into the existing gossip step so it costs nothing extra. It makes
`alive` mean what every reader already assumes, removes 93% of the book from the per-tick walk, and
leaves ids stable.

Deferred until the operator decides, because it changes what a host sees in `RumorBook.alive`.

## What ships now

The `Knowledge` cap (`knowledge_capacity`, default 64), which is not a decision: it matches
`memory_capacity` and `max_ties`, both of which already bound a per-person list, and it changes no
observable behaviour at the default village (all 4 200 series rows and the narrative are byte-identical).

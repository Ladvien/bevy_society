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

Two costs follow, both measured:

- `rules.rs` walks the **entire book every tick** looking for `RumorReach` triggers, so tick cost grows
  with the length of the run. 1 000 people: **15.0 ms/tick** over a short run, **61.9 ms/tick** after
  3 000 ticks of settling — a **4× blowup** with no increase in living people.
- A person's `Knowledge` also grew without bound (344 entries after 4 000 ticks on 12 people). That
  is now capped at `SocietySettings::knowledge_capacity` (default 64), which removed 2.3% of the long
  run's cost. The remaining growth is the book.

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

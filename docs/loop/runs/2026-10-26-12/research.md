# Research: a meal is what was on the shelf

## The defect, measured before any reading

The price pass (run `2026-10-26-11`) ended by noticing a fact it could not fix, because fixing it was a
different change: **the village holds zero bread for 24.0% of its ticks**, in one unbroken run of **3.39
game-days** (`evidence/fresh24/s23_p24/series.csv`, seed 23, 24 people, 17.62 game-days), and the price
reports that state exactly — every tick at the famine value `2.0` is a tick whose `bread_max` is exactly
`0.0`. Nothing in the simulation reads it.

Counted from that same series, separating arrival ticks from travel ticks with `Doing::label`'s own
convention:

| | per game-day |
|---|---|
| bread **baked** (`bake`, 3 loaves a completed use) | **4.80** |
| `eat` **completed** | **21.06** |
| `eat` ticks spent walking to the table | 20.4% more than the ticks spent eating |

So about **77% of this village's meals are eaten out of an empty store**, and every one of them pays the
full `satisfies: [("food", 1.2)]` for its whole duration. `produce_and_consume` (src/economy.rs) takes
what `Stock::take` finds, which is nothing when the shelf is empty, and reports it to no one. The decision
model never learns that the meal did not happen.

That is not a tuning gap. It is two systems describing the same event differently: the economy says "no
bread changed hands", the needs model says "you are fed".

## What the literature says the number should be

**The size of a meal is set by what is eaten, not by sitting at a table.** Zandstra 2000 (Wageningen
thesis, DOI `10.18174/197257`), verified on disk at
`~/.cache/bevy_elements_corpus/markdown/10/10.18174_197257.md:207`:

> "The control of appetite involves two different processes, satiation and satiety. ... **Satiation is the
> process that terminates the eating within a meal, and influences the size of meals** and snacks. Satiety
> is the state that occurs after eating of a food or meal has ended, and inhibits further eating, which
> influences the frequency of meals and snacks."

The paper's own split is the whole argument. `satisfies` is the crate's **satiation** term — it is paid per
game-hour *of the meal* — and a meal's satiation follows the amount consumed. With no food there is no
satiation regardless of how long a person stands in the kitchen. The crate implements the duration
faithfully and the *amount* not at all.

**Homeostasis, the frame the needs model already claims.** Maslow 1943
(DOI `10.1037/h0054346`, in the corpus at `10/10.1037_h0054346.md:48-65`) grounds the physiological needs
in Cannon's homeostasis — "the body's automatic efforts to maintain a constant, normal state" — and adds
that "appetites (preferential choices among foods) are a fairly efficient indication of actual needs or
lacks in the body." `docs/research/needs-and-utility.md` already leans on this framing. A homeostatic
drive is reduced by what restores the controlled variable; a person who finds the larder empty has not
restored it.

**What the game-AI corpus contributes, and what it warns against.** Zoo Tycoon 2's guests "would select
options based purely on their needs (i.e., their hunger need ...)" (`gameaipro2-ch03-dual-utility-reasoning`,
line 64), which is the pattern the crate already has. The warning is in
`gameaipro3-ch34-1000-npcs-60-fps:101`, where a shipped game **removed** internal hunger state because
"internal state acted as 'hidden information' that made it difficult for both the designer and the player
to understand why an individual is behaving in a certain way". Both halves are the same lesson and they
point at this defect: the state a host can already see (`Stock`, `Prices`, `ctx.stock`) must be the state
that decides the outcome, or the simulation is lying about what it showed you.

## The formula

For an activity with `consumes: [(good, amount)]`, pay `satisfies` in proportion to the fraction of what it
asked for that the world actually gave:

$$ \text{paid} = \text{satisfies} \times \min\!\left(1, \frac{\text{taken}}{\text{asked}}\right),
\qquad \text{taken} = \texttt{Stock::take}(good, amount) $$

Three properties this shape has, and each is load-bearing:

1. **A meal with the bread is unchanged.** `taken == asked` gives a factor of exactly `1.0`, so every
   existing village that is fed keeps its numbers to the bit.
2. **A meal with nothing pays nothing.** `taken == 0` gives `0.0`. This is the defect closed.
3. **A partial meal pays partially.** Half a loaf for a whole one feeds half as much — the linear reading
   of "satiation follows intake", and the only shape that does not invent a threshold nobody published.

**An activity that consumes nothing is untouched**, which is every activity in the pack except `eat`: the
factor is `1.0` and no stock is read. That keeps the change inside one activity's arithmetic.

**Which `satisfies` slots are scaled is a real question and the answer is "all of them".** The crate has no
per-slot notion of *what* a need is restored by — `satisfies: [("food", 1.2)]` and
`satisfies: [("social", 0.3)]` on `play` are the same kind of data. Attaching a good to individual slots
would be a definitions-file schema change (`satisfies: [(need, rate, Some(good))]`), which is a
`needs-decision/` item, not something to decide mid-iteration. Scaling every slot of an activity that
declared a `consumes` is the smallest rule that is honest: an activity that consumes a good did not happen
unless the good was there, whatever it pays. The starter pack has exactly one such activity, and it pays one
need. The docs say so, and the limitation is recorded rather than hidden.

## Parameters

**None.** There is no new `Tuning` field and no new rate. The factor is `taken / asked`, both of which are
already authored in the definitions file. A `Tuning::partial_meal_penalty` would be a number with no source,
inventing a curve where the authored quantity already gives one.

## What believable output looks like

- **A fed village is bit-identical.** Summary fields and the narrative stream match the pre-change run
  except where a meal was actually short. This is the guard that the change is one change.
- **The famine stops feeding people.** `food` must fall during the 3.39-day famine, not hold at its usual
  mean. Measured as the difference in mean `food` during zero-bread ticks, before and after.
- **Nobody is blocked** — `eat` is still chosen with an empty shelf, so the village still walks to the
  table, still spends the ticks, and simply is not fed. Gating was measured to wreck the village
  (`runs/2026-10-25-06/eval.md`: seed 42 rivals 29 → 55 and rumor reach 0.50 → 0.08) and is not what this
  is. The failure is **paid for**, not **prevented**.
- **The price becomes readable, not causal.** `Prices` is a function of the store and no decision reads it;
  a village that eats less bread in a famine starves the store *more slowly*, so the famine runs shorter.
  That is a second-order consequence to measure and report, not to claim.

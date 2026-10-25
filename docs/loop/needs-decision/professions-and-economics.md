# Should a society have professions and an economy?

> **Requested 2026-10-25. Not started.** It is a definitions-schema change of the largest kind: it adds two
> new kinds of authored data and makes them interact with four existing models.

## What a host has now

- `Place` with `capacity` and `PlaceTags`, and activities that target a tag (`work` needs a place tagged
  `work`).
- `Status` — conferred standing, moved by contests and witnessed deeds — and `Memberships` for factions.
- Nothing that says *what a person does for a living*, nothing a person can be short of, and nothing to
  trade.

A village where everyone works at "a place tagged work" behaves as one undifferentiated labour pool: the
`work` activity is `Hour(8, 17)` and its only target filter is the tag, so a miller and a weaver are the
same person with a different faction colour.

## Why it is a decision, not a change

1. **Two new definition lists** (`professions`, `resources` or `goods`). The definitions file is the
   surface a host ships and edits, and `DefsFile`'s loader is `deny_unknown_fields`, so this is a
   schema break for any host that pins a pack — and it must be merged into `DefsFile::starter()` without
   changing any existing default.
2. **It changes what `Tuning` must hold.** A wage, a price, a stock ceiling: three rate-shaped numbers that
   the crate's own rule says must each cite a paper or say "design default" and be measured.
3. **It interacts with four existing models** — needs (a wage buys food, so `food` stops being satisfied
   only by `eat`), factions (a profession is not a faction but correlates with one), status (a wage
   confers standing) and rumors (a bad deal is a deed). Getting one wrong quietly invalidates the
   measurements in `docs/research/`.

## Options

### 1. A profession is a tag on a person, and an activity names one

A person carries `Profession(id)`; an activity may require `profession: "miller"` in its `target` or as a
consideration. No money at all.

- **For:** tiny, no new resource, and it immediately makes the village heterogeneous — the cheapest way to
  answer "do people behave differently?".
- **Against:** it is a title, not an economy. Nothing is scarce and nothing is exchanged.

### 2. A profession plus a stock of goods, with a market place

Each person holds `Inventories` keyed by a goods list; `work` produces, `eat` consumes, a new `trade`
interaction swaps at a price that moves with stock.

- **For:** the actual request, and it gives `Place("work")` and `Place("food")` a *reason* to be distinct.
- **Against:** a large model with several unsourced numbers, and it duplicates the existing
  `satisfies`-based need loop unless needs are re-expressed in goods.

### 3. Needs become demand for goods, and nothing else changes

`satisfies` names a good instead of a need; a person with no good goes without.

- **For:** no new decision machinery — the utility model already chooses activities, and now choosing is
  buying.
- **Against:** it changes the meaning of the starter pack's `satisfies` for every existing host, and needs
  named `social`/`rest`/`fun` are not goods at all.

## Recommendation

**Option 1 first, as its own iteration, measured on `village-evening`.** It is the slice that answers the
question the request is actually asking — do people with different work behave differently? — without a
schema break for money, and it makes option 2's later arrival additive rather than a rewrite. Option 2 is
the destination; option 3 should be rejected because it breaks every existing pack's `satisfies`.

Whichever is chosen, the first measurement is the same: run `village-evening` with three professions and
show that the activity mix and the tie graph differ by profession. If they do not, the model is decoration.

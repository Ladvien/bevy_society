# Which field says someone actually spoke?

> **Found 2026-10-25-03. No change made: this is a save-format decision.** The measurement that raises it is
> in `docs/loop/runs/2026-10-25-03/`.

## The problem, measured

A friendship can end for two very different reasons, and the crate cannot tell them apart:

| seed 7, 16.4 game-days, 504 endings | count |
|---|---|
| within six game-hours of a witnessed **insult** involving the pair | 99 |
| within six game-hours of **any exchange** with that person | 349 |
| **with no interaction behind them at all** | **56** |

The first two are events a host should be told about. The third is the label crossing a bar from continuous
drift — relaxation, balance, similarity, proximity exposure — and announcing it as a relationship change is
false. `Tuning::friend_band` (0.05, shipped 2026-10-25-03) absorbs the *shallow* ones and moves this third
count only 56 → 43, because a deeper drift still ends the friendship.

**The reason it cannot be fixed now:** nothing on `Tie` records that a conversation happened.

- `last_contact_day` looks like it should. It does not: `expose_contacts` stamps it on **mere proximity**
  inside `contact_radius` (4 m), so two people standing in the same square for an evening stamp each other
  every tick without saying a word.
- `Ties::entry` / `expose_toward` both funnel through `expose`, which stamps the same field.

So "quiet friendship" and "friendly pair who were in the same market" are indistinguishable, and a band
keyed on "no recent contact" would hold friendships for the wrong reason.

## Options

### 1. Add `last_exchange_day` to `Tie`

Stamped only where an exchange is resolved (`resolve_interactions`, both the accept and the refuse path).

- **For:** exactly the signal needed, one field, and the semantics are unambiguous. `friend_band` could then
  apply only when the pair has not spoken recently, which is the shape that fixes the 43.
- **Against:** `Tie` is serialized — `TieSave` gains a field, so `SocietySave::VERSION` must go to 2 and every
  host's saved society is invalidated. That is a host-visible break and cannot be decided in an iteration.

### 2. Reuse `last_contact_day` and narrow what stamps it

Stop `expose_contacts` stamping it; stamp only on exchange and on `introduce_toward`.

- **For:** no new field, no save change.
- **Against:** it changes what an existing public field *means*, which is worse than breaking the format
  loudly. `Tie::familiarity` reads `last_contact_day` as "when we last met" — power-law forgetting would then
  decay from the last *conversation* rather than the last sighting, which silently changes the relationship
  model and the `friend_familiarity` gate along with it.

### 3. Leave it; accept that some drift endings are announced

- **For:** no schema change, and 43 of 242 remaining endings is small.
- **Against:** it is a false message, and a host that shows "you are no longer friends" on it is showing a
  relationship change that did not happen.

## Recommendation

**Option 1, in an iteration that can spend the version bump.** It is the only one that adds the signal
without changing what an existing field means, and `SocietySave::VERSION` exists precisely so this can be
done deliberately: `restore` already refuses a save from another version rather than guessing, so the
migration story is "a host with a 0.2 save gets a `Version` error it can act on", which is the crate's
stated behaviour.

Until then, `Tuning::friend_band` is the stopgap and the changelog says so: it holds shallow dips, not
causal ones.

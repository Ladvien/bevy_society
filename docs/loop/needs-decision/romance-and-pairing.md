# Should a society have romance?

> **Requested 2026-10-25. Not started.** Like professions, this is a definitions-schema change, and it is
> larger: it needs a *dyad* that outlives either person's opinion of the other, which the tie model
> deliberately does not have.

## What a host has now

`Tie` is one directed edge per person: affinity, trust, respect, familiarity, a label, and the day of last
contact. `TieLabel` is `Stranger | Acquaintance | Friend | Rival`. There is no pairing, no exclusivity, no
shared state between two people, and no asymmetry between "how I feel" and "what we are".

That is deliberate — `docs/research/relationships.md` lists "Kin, role or structural ties (family,
employer)" under **what the model leaves out** — and it is why two people cannot be married in this crate
today.

## Why it is a decision, not a change

1. **It needs a new kind of state: a dyad that both people share.** Every existing relationship fact is
   per-direction, and this iteration's `dyad_balance` only *converges* the two directions. Romance needs a
   record that belongs to the pair: who is with whom, since when, exclusively or not. `SocietySave` has no
   container for it, so this is a save-format bump on top of a schema change.
2. **It changes what `TieLabel` means.** `Partner` cannot be a fifth `TieLabel`, because a label is derived
   from that direction's affinity/trust and a marriage is not. Either the label set grows into a *state*
   (breaking for every host that matches on `TieLabel` exhaustively) or a separate component carries it.
3. **It interacts with four models that already have measured defaults**: affinity and the friend band
   (a partner is not merely a high-affinity friend), jealousy and third parties (a new source of `Anger`
   and of `NormViolated`, which will move the mood numbers in `docs/research/personality-and-mood.md`),
   factions (a cross-faction marriage is the strongest intergroup contact the crate could have, and
   `Pettigrew & Tropp` is already cited for `contact_gain`), and bereavement (the crate already grieves a
   death; a partner's death should be the strongest case and is currently just `familiarity`).
4. **Every rate needs a source or an admission.** Courtship, exclusivity, dissolution: three new
   `Tuning` fields with no paper that gives their values.

## Options

### 1. A `Partners` component on each person, holding at most one `Entity`

Two people each carry the other's entity and the game-day it began. Nothing else changes.

- **For:** the smallest thing that answers "who is with whom", it survives a save with one new field, and
  it lets dialogue and scripts branch on it (`ctx.partner(a) == b`).
- **Against:** no courtship, no dissolution, no jealousy. It is a flag, not a model.

### 2. Courtship as an activity, partnership as a threshold

A `court` interaction that raises an existing but unused dimension — `respect` is currently the least
exercised — and a `Partner` label that appears when mutual affinity and trust both pass a bar for a held
duration. Dissolution when they fall below a lower one.

- **For:** uses machinery the crate already has (the interaction/exchange path, the label bands from the
  last three iterations), and the bands already exist for `Friend`.
- **Against:** a second label system layered on `TieLabel`, and the duration requirement is a new kind of
  state ("has been above the bar since day X") that `Tie` does not carry — the same gap that blocks
  `last_exchange_day`.

### 3. Leave relationships unlabelled and let the host do romance

Ship nothing; document that a host builds pairing on top of `Ties` and its own components.

- **For:** zero risk to the measured models, and it is what the crate's stated scope already says.
- **Against:** every host that wants romance writes the same courtship loop, and the crate's own research
  note lists the absence as a gap.

## Recommendation

**Option 1, after professions, as its own iteration** — and only with a measurement attached: run
`village-evening` with pairing enabled and show that the tie graph and the mood distribution change in a way
a stranger can explain. If a `Partner` flag moves nothing measurable, it is a label a host could have added
itself and the crate should not grow a schema for it.

The first thing to measure is **whether the crate can already express it**: take one pair, set mutual
affinity and trust above every bar, and see whether anything downstream behaves differently from a very
good friendship. If nothing does, option 3 is the honest answer.

## Note on the existing `Deceased`/bereavement path

`Tuning::bereavement_familiarity` (0.3) is the crate's line between an acquaintance and someone you knew,
and `docs/research/relationships.md` says it "exists so a family member grieves". A partner should grieve
harder than a friend; today the only lever is `familiarity`, which a partner already shares with a close
friend. That asymmetry is the strongest argument for pairing being real state rather than a label.

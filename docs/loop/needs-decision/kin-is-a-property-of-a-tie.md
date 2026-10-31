# Where does "this person is my sibling" live?

> **Assumed decision (owner may override), 2026-10-31-05: a blood relation is an `Option<Kin>` on each directed `Tie`, the two directions carry each other's converse, and `Tuning::kin_premium` (1.0, a design default) multiplies a gift's score toward a relative.** Researched and measured in `docs/loop/runs/2026-10-31-05/`; the sources are in `docs/research/kinship.md`.

## The state

The crate had no family. `docs/research/relationships.md` listed "kin, role or structural ties (family, employer)" among what the model leaves out, and the only mention of a relative in the code was the doc on `Tuning::bereavement_familiarity`: a family member grieves because the author gave them a high familiarity. A host could not say who is related to whom, a pack could not score by it, a script could not ask, and a Dunbar-style trim of a long-unvisited tie would forget a sister. `docs/loop/needs-decision/romance-and-pairing.md` had named the missing piece for pairing too: a fact that belongs to two people.

## Options

- **(a) `kin: Option<Kin>` on each directed `Tie`** (picked). One byte on a struct already copied into every decision; `signal_value` reads it with the tie it already holds, so the scoring loop pays nothing; a save carries it per tie with `#[serde(default)]` (a file from the previous release loads as no relatives, `VERSION` stays 5); `trim_keeping` sees it; the editor's `PlacementFile::capture` writes the name back. The price is that a dyad's fact lives twice, and spawning (`TieSpec::kin`) keeps the two copies converse of each other: `Mara` lists `Ada` as `Child`, `Ada`'s tie to `Mara` is made `Parent`, and two authored directions that are not converses are refused with both names.
- **(b) A symmetric pair registry** (a resource keyed by the two people). The right shape for a dyad and the shape a pairing model will want, but it is a lookup per scored candidate, a new container in `SocietySave`, and a second place a person's ties are kept. Deferred, not rejected: `Tie::kin` can be derived from such a registry later without a host noticing.
- **(c) Leave kin to hosts**, which build it on their own components. Rejected: a pack could not score by it, and the premium below cannot be a pack's business, because a pack written before kin existed (the shipped village's `share`) would have to be edited to favour a relative.

## The premium

`Tuning::kin_premium` multiplies the score of an activity whose interaction **gives goods and takes none** (`actor_gives` non-empty, `target_gives` empty) by `1 + premium · 2r` toward a relative with relatedness `r`. It sits on the score, not in the acceptance draw, because Curry, Roberts & Dunbar 2013 measured the **giver's** willingness. It sits on the engine and not in the pack, as `crowding_penalty` does, so a pack written before kin existed gets it unedited.

- **On by default at 1.0**, a design default (the three sources give the direction and the dependence on `r`, no size). Options: (i) `1.0` on, picked, since it touches nothing in a society with no relatives (the harness at seed 7, 6 game-days, `cmp`: `summary.json`, `narrative.txt`, `series.csv`, `activities.txt`, `partners.txt`, `reapproach.txt` and `shares.txt` byte-identical to the build before) and no society has relatives yet; (ii) `0.0` off, `kin_premium` a host's opt-in, which would leave every host that authors a family with a family that does not favour its own until it reads this note.
- **Not a swap.** A `target_gives` interaction is an exchange, and the premium is for altruism.

## What it does not do

No pairing, no jealousy, no inheritance, no births, no kin recognition under uncertainty, no nepotism in witness reactions (`docs/research/kinship.md`, "What is not modelled"). A relative is **not** kept alive by contact differently from a friend: that finding (Roberts & Dunbar 2011, DOI `10.1016/j.evolhumbehav.2010.08.005`) could not be read, so the only difference in how a relative's tie ages is that the Dunbar cap never removes it.

## What deferring costs

Nothing is lost by overriding: setting `Tuning::kin_premium` to `0.0` is the previous release's ranking, and removing the `kin` writes from a placement file leaves a society with no relatives, which is the previous release byte for byte.

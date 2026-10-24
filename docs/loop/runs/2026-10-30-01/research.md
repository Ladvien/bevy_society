# Research: gossip needs something to tell

The log's #1 was `chat` starving against `gossip`, escalated last iteration in
`needs-decision/chat-and-gossip-share-one-need.md` with three options and a recommendation. This
iteration implemented the recommended option 2.

## Why that option, in one line

**Sommerfeld et al. 2007** (`10.1073/pnas.0704598104`) — *Gossip as an alternative for direct
observation in games of indirect reciprocity* — names gossip's function as reputation
**information**, distinct from conversation. A villager with no information has nothing to gossip
about, so the gate is not a trick to break a tie: it is the mechanism.

(That paper's converted document is the abstract page only, so it supports the framing and supplies
no parameter. Nothing was lifted from it.)

## The expression gap that had to be closed first

Option 2 needs a definitions file to ask "does this person have anything to tell?". It could not:

- `ctx.knows(a, rumor)` requires a **rumor id**, and an authored file has no way to name one —
  rumors are created at runtime by `PlantRumor` or by `fx.rumor`.
- Nothing else in `ctx` reports anything about a person's knowledge.

So `ctx.tellable(a)` was added: whether `a` knows any rumor it is still spreading. It is the
*teller's* half of `rumor::gossip`'s choice, and only that half — whether a particular listener
already knows the rumor needs the rumor book and its subject entities, which the snapshot does not
carry. A gossip that turns out to repeat something the listener already had still stifles the
teller, exactly as `gossip` does.

## What the gate bought, measured

**In the village, it works as intended:**

| activity | before | after |
|---|---|---|
| `chat` | 8746 | **9312** (+6.5%) |
| `gossip` | 15050 | 14160 |
| `snub` | 6004 | **1617** (−73%) |
| `work` | 14601 | 17086 |
| `eat` | 14824 | 16931 |

Fewer gratuitous gossips means fewer insults: `snub` falls by nearly three quarters, because snub
is driven by grudges, and most grudges came from insults that gossip interactions produced.

**In the cluster it changes almost nothing** — `chat` 128 → 140 — and that is now understood rather
than disappointing: the cluster generates **279 rumors** over its run, so `tellable` is true for
almost everybody and the gate never closes. The gate can only differentiate where rumors are
scarce, which is the village (28 rumors among 24 people).

So the starvation in the cluster has a different cause from the one the decision note assumed. The
gate is correct and worth having — it is the literature's mechanism and it cuts insults by 73% —
but it does **not** resolve the 1:24 ratio. That needs the note's option 1 (a second need), and the
note is updated rather than closed.

## Verdict

One expression gap closed (`ctx.tellable`), one starter-pack gate added on the literature's
mechanism, and a measured 73% reduction in snubbing. The cluster starvation is **not** fixed, and
the reason is measured: too many rumors for the gate to bite.

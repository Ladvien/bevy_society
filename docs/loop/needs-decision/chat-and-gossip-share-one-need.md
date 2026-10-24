# Should `chat` and `gossip` keep sharing the social need?

## The problem, measured

`chat` and `gossip` both target a person, both satisfy `social`, and both compete at rank 0. In the
village they coexist well (chat:gossip 1 : 1.7 over 4200 ticks). In a cluster of twelve where gossip
is always the best available option, `chat` gets **128 person-ticks against gossip's 3093** — a
1 : 24 starvation.

Two attempts to tune it out, both measured and reverted:

| attempt | chat person-ticks | outcome |
|---|---|---|
| baseline | 128 | — |
| gossip `satisfies` 1.0 → 0.3 per hour | **106** | worse |
| gossip interaction effect 0.15 → 0.05 | **153** | +20%, social still pinned at 0.903 |

**Why no rate can fix it.** A gossip fires about 17 times per person per game-day. Any per-instance
supply `X` with `17X ≥ 1.0` (the daily decay) keeps `social` above the ≈0.45 at which `chat`'s
`Need(social) Linear(-1, 1)` gate would open. So `X ≥ 0.06` locks chat out, and `X < 0.06` makes
gossiping worthless. The loop closes on itself: gossip supplies social → social stays high → chat's
gate stays shut → gossip stays the best option.

## The literature's framing

**Sommerfeld et al. 2007**, *Gossip as an alternative for direct observation in games of indirect
reciprocity* (PNAS, `10.1073/pnas.0704598104`, corpus score 0.700 — the strongest hit in the corpus
for this topic) frames gossip as a substitute for **direct observation**: its function is
reputation information, not affiliation. That is a different function from conversation, and the
starter pack models both as "social need, satisfied by talking".

(The converted document is the abstract page only, so it supports the framing and supplies no
parameter.)

## Options

### 1. A second need: `belonging` for chat, `social` for gossip

`chat` satisfies `belonging`; `gossip` satisfies `social` (curiosity about others). The two then
stop competing for one resource and a person can want both.

- **For:** the two activities finally mean different things; no rate tuning needed; the defs file
  already supports any number of needs.
- **Against:** a schema addition — a new need in the starter pack, which changes every host's
  defaults and the `Needs` vector's length. Breaking for a host that indexes needs by position.

### 2. Gate `gossip` on having something to tell

`gossip`'s consideration becomes a script that scores 0 when the person knows no rumor any listener
would not already know. A person with nothing to say chats instead.

- **For:** no schema change; uses `ctx.knows` and `ctx.persons`, both already public; and it is
  *true* — people gossip about news.
- **Against:** a script consideration per decision has a cost, and a person who knows nothing would
  never gossip, which may be too strict (people do gossip without news).

### 3. Leave both, and document that a host should weight them

Hosts set `gossip`'s `weight` below `chat`'s in their own defs.

- **For:** no change at all.
- **Against:** the starter pack still ships a configuration where one activity can starve the other,
  and a host has no signal that it happened.

## Update: option 2 was implemented, and it does not fix the cluster

`ctx.tellable(a)` was added (2026-10-30-01) and the starter `gossip` now gates on it. Measured:

| | village | cluster |
|---|---|---|
| `chat` | 8746 → **9312** (+6.5%) | 128 → 140 |
| `gossip` | 15050 → 14160 | 3093 → 3186 |
| `snub` | 6004 → **1617** (−73%) | 2527 → 2188 |

The gate is correct and worth keeping — it is the literature's mechanism and it cuts snubbing by
nearly three quarters — but the cluster is unchanged. The reason is measured, not guessed: that
scenario produces **279 rumors**, so `tellable` is true for almost everybody and the gate never
closes. A gate on "having something to tell" can only differentiate where rumors are scarce.

**So option 1 is now the only remaining path** for the cluster's 1:24 ratio: a second need, with
`chat` serving `belonging` and `gossip` serving `social`. Option 3 stays rejected for the reason
given.

## Recommendation

**Option 2**, then **option 1** if it proves too strict. Option 2 needs no schema change, is
expressible today with `Input::Script`, and addresses the actual mechanism the literature names —
gossip carries *information*, and a person with none should fall back to conversation.

Until it is decided, `docs/research/rumor-and-reputation.md` records the starvation and this note
carries the measurement, so the next iteration starts from evidence rather than from the symptom.

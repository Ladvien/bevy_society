# Plan: give gossip something to say

## Why

The log's #1 was `chat` starving against `gossip` in a cluster. Last iteration escalated it to
`needs-decision/chat-and-gossip-share-one-need.md` with three options; the recommendation was
option 2 — gate gossip on having something to tell — because it needs no schema change and matches
the literature's account of what gossip *is*.

## Design

1. **`ctx.tellable(a)`** — a new `ctx` call: whether `a` knows any rumor it is still spreading.
   Added because the gate could not otherwise be expressed: `ctx.knows` needs a rumor id, and an
   authored file cannot name one.
2. **`gossip` gains a leading consideration** `Script("if ctx.tellable(actor) { 1.0 } else { 0.0 }")`
   through `Linear(1.0, 0.0)`, so a villager with nothing to say scores 0 and chats instead. The
   previous `weight: 1.0` line and its long comment are replaced by this, because the gate is now
   the thing that separates the two activities.

## What this is *not*

It is **not** a fix for the cluster's 1:24 starvation, and the measurements said so before I wrote
this section: the cluster generates 279 rumors, so `tellable` is true almost always. The gate is
correct and worth having on its own terms — it cuts `snub` by 73% in the village — but the ratio
needs option 1 (a second need), which is a schema change and stays in the note.

## Touched

- `src/snapshot.rs` (`SocietySnapshot::tellable`),
- `src/script.rs` (registering `ctx.tellable`),
- `assets/starter.society.ron` (`gossip`'s considerations),
- `tests/rumor.rs` (one test),
- `CHANGELOG.md`, `docs/guide/scripting.md`, `docs/loop/log.md`, the decision note.

No new `Tuning` field: the gate is a definition, not a rate.

# Research: the thing I could not tune, and the thing I could at least detect

The log's #1 was "`chat` is nearly unreachable: `Need(social) Linear(-1,1)` scores 0 at a full need
while gossip self-supplies it. Either the gate's curve or the `satisfies` balance needs work."

## What the measurement showed

Activity census, person-ticks over each scenario's run:

| scenario | chat | gossip | ratio |
|---|---|---|---|
| `village-evening` | 8746 | 15050 | 1 : 1.7 — healthy |
| `rumor-crosses-a-cluster` | **128** | **3093** | 1 : 24 — starved |

So the gate is not broken in general. The village chats constantly. Only the cluster starves, and
the cluster is where gossip is the best available option all day.

## Why it starves, and why two attempts to fix it failed

`chat` needs `Need(social) Linear(-1, 1)`, which is 0 at a full need and only overtakes gossip
below social ≈ 0.45. Measured, the cluster's social never falls below **0.754** — a mean of 0.895.

Two attempts, both measured and both reverted:

1. **Lower gossip's `satisfies` from 1.0/h to 0.3/h.** `chat` went from 128 to **106** — worse,
   because the interaction effect still supplied the need.
2. **Lower the gossip interaction's effect from 0.15 to 0.05.** `chat` went 128 → **153** — a 20%
   improvement that left social at 0.903.

Arithmetic for why: a gossip fires **~17 times per person per game-day** in that cluster, so
`17 × X ≥ 1.0/day` holds the need up for any `X ≥ 0.06`. A per-instance effect small enough to
balance the decay would make gossiping worthless, and the loop closes on itself: gossip supplies
social → social stays high → chat's gate stays shut → gossip stays the best option.

**This is not a rate that can be tuned.** It is two activities serving one need with no
differentiation between them, and separating them is a definitions-schema question (a second need,
or a `gossip` variant gated on having something to tell) that belongs in `needs-decision/`.

## What I chose instead, and why it still counts as the most valuable thing here

The log's #2 was "durations do not scale with the day". Chasing #1 for most of this iteration
demonstrated exactly why #2 matters: the bug that kept producing these findings is an activity
shorter than one tick, and it has *always been silent*.

Measured: at `seconds_per_day = 4.0` a tick is 0.094 game-hours, so `snub` (0.1 h) lasts **1.1
ticks**. A person re-decides on the tick their activity starts, and repeats whatever scored
highest — 38 snubs per person per game-day. The crate accepted that definitions file without a word.

That is a host-ergonomics defect of the first order: the host hands in a valid file, gets a
degenerate society, and has no signal. So the change is a **startup warning** naming every activity
shorter than two ticks, with the tick length and the activity's own arithmetic:

```
WARN bevy_society::behavior: bevy_society: 1 activities are shorter than two society ticks at
seconds_per_day 4 (one tick is 0.094 game-hours), so their people will re-decide every tick:
snub (0.1h = 1.1 ticks)
```

`SocietyDefs::activity_durations()` is exposed so a host can make the same check itself.

## The corpus

Searched `conversation vs gossip distinct social functions Dunbar` and
`gossip distinct from conversation reputation information not affiliation`. The one strong hit is
**Sommerfeld et al. 2007** (`10.1073/pnas.0704598104`, score **0.700**) — *Gossip as an alternative
for direct observation in games of indirect reciprocity*.

It was read and it supports the diagnosis at the level of *mechanism*: gossip substitutes for direct
observation, which is a different function from conversation. That is exactly the differentiation
the starter pack lacks. But the document converted to **one page — the abstract only** — so it gives
no parameter and nothing was lifted from it.

`docs/research/rumor-and-reputation.md` already cites it for the mechanism.

## Verdict

One real, testable, host-facing change (the startup warning plus a public accessor), and an honest
negative result on the log's #1: two failed attempts, both measured, both reverted, with the
arithmetic showing why no rate can fix it and the decision escalated instead.

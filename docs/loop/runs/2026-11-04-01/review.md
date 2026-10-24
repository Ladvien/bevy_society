# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: is the pair balanced?

Claim: every `ActivityEnded` now closes an `ActivityStarted` a host was told about.

**Evidence.** `every_activity_end_has_a_start`, written **red first**: one person, one place 50 m
away, no steering plugin, so nothing ever arrives.

| | starts | ends |
|---|---|---|
| before | **2** | **55** (53 unpaired) |
| after | 150 | 149 (one in flight at the cutoff) |

And in the narrative the review gate requires, counting lines rather than trusting the code:

| run | `started` | `finished` |
|---|---|---|
| `village-evening` before | 14 | **18** — unpaired |
| `village-evening` after | 27 | 26 — paired |
| `fresh-eyes` seed 99 / 40 people | 22 | 21 — paired |

**Critique.** The test's bound is `starts >= ends` with at most one unpaired, not equality. That is
weaker than the invariant sounds, and it is deliberate: a run can stop mid-activity. My first draft
asserted equality and went red at `150 / 149` for a *correct* implementation — a test that would have
forced a wrong fix. The narrative counts are the stronger check because they are whole transcripts.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the artifacts closely

**Critique.** `evidence/before-after.png` and `evidence/fresh-eyes/`. Four things a close read shows:

1. **The pairing is not cosmetic — behaviour moved at the same seed.** Rumors 28 → **53**, friend
   edges 91 → **118**, rumor reach 0.500 → 0.333. Nothing but message *timing* changed, which is the
   strongest evidence the pair is load-bearing for gossip (`ctx.tellable` moves when exchanges land at
   different instants).
2. **The two starvation symptoms are untouched, and that is correct.** `need2` and `need3` still flat
   at 0 in the plot. Measured separately: with zero travel, `need3` is at 0 for 28.2% of ticks rather
   than 78.2%, so walking is part of it but not the cause. The cause is activity *choice*: over 1 419
   starts, `snub` 589 and `gossip` 561 — 82% — and neither satisfies a need.
3. **The fresh-eyes narrative now reads correctly on the message axis**: `started work` at `d1 15h05`
   is followed by `finished work (interrupted)` at `d3 07h07`, 40 game-hours later.
4. **...and that is the escalated two-clocks decision, now visible as plain text.** A person walks to
   bed for 40 game-hours. `needs-decision/travel-time-in-two-clocks.md` already records it and
   recommends option 1; this iteration does not decide it, and the narrative is a much better
   argument for the decision than the file was.

**Critique of the fix's own risk.** Barks now fire at decision instead of arrival. A host reading
`work`'s bark "Back to it." now sees it as the person sets off, not on the doorstep. That is the
better reading, and it is documented on the message type, but it *is* a host-visible timing change —
noted rather than escalated because the alternative (barks for activities that never happen) is worse.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: a different seed and population

**Critique.** `evidence/fresh-eyes/` — seed 99, 40 people, 17 game-days. Narrative paired 22/21. The
day reads coherently to a stranger: work, gossip with V33, chat, sleep, and names recur (Hilda, Sable,
Dena) the way a small society's should.

**Critique of what it exposes.** All four needs of the tracked person still end at `0.0`, and the
society is 406 friend-edges with **0 rivals**. So at 40 people the village is friendlier and hungrier
than at 24. Both are the same recorded defect, not new ones — and the friend/rival split is worth
noting for the next iteration since `snub` fired 589 times in the probe yet no rival survives.

**Verdict: GREAT** for the change (paired at any seed and population); **NOT YET** for need
satisfaction, which is the next iteration's item with numbers already recorded.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 2/55 → 150/149 in the repro; narrative counts confirm it |
| Zoom | GREAT — pairing measurably moved gossip and friendship at a fixed seed |
| Fresh eyes | GREAT for the change at seed 99 / 40 people; NOT YET for needs |

## Top 5 remaining

1. **Need satisfaction via activity choice** — `snub` + `gossip` are 82% of starts and satisfy nothing;
   `need3` at 0 for 28% of ticks even with zero travel. Measured both ways.
2. **The two-clocks decision** — the fresh-eyes narrative shows a 40-game-hour walk to bed in plain
   text. `needs-decision/travel-time-in-two-clocks.md` recommends option 1; needs the operator.
3. **Rumor reach flat-lines** — 0.333 after tick 62; Daley–Kendall stifling should decay it, not freeze it.
4. **`Field` / `Gradient` wiring** — unchanged.
5. **`warn_unscored_fields` says "1 definitions"** — plural helper.

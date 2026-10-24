# Research: what a start message means

## The problem, measured

Last iteration's review found 19 `ActivityEnded` against 9 `ActivityStarted` for one tracked person
and could not explain it. This iteration reproduced it in isolation and found one cause.

**Reproduction.** One person, one place 50 m away, no steering plugin, so nothing ever arrives:

```
2 starts, 55 ends
```

53 activities were reported as *finished* that no subscriber had been told *began*. At the village's
40-person fresh-eyes setting the same defect scales: **23 745 starts, 26 412 ends** — 2 667 unpaired
ends.

## Cause

Two emit sites disagreed about what "started" means:

| Site | When it fires | `arrived_day` |
|---|---|---|
| `decide` (`behavior.rs:831`) | at decision, **only if** `in_place` — which is hardcoded `false` for every activity the decision picks | `None` (travelling) |
| `arrive_and_satisfy` (`:340`) | on arrival, for everything that travelled | `Some(now)` |
| `end_activities` (`:449`) | **whenever a `Doing` is removed**, including a travel timeout | — |

So a decided activity that travels emits no start until it arrives, but emits an end the moment it
stops — arrival or timeout. A person whose target is out of reach therefore closes an activity that
was never opened.

The crate already said what should happen. `Doing::started_day` is documented as *"Game day the
activity started"* and is stamped at decision (`:806`), in the same `insert(Doing { .. })` that
`ActivityStarted` accompanies. The message was simply inconsistent with the component it describes.

## Literature

Utility AI has a settled vocabulary for this split, and it is the reason the fix is "start at
decision" rather than "start at arrival":

| Claim | Source |
|---|---|
| An action has a *selection*, a *start/enter*, a *run* and an *exit*; only `run` depends on being in position | Mark & Dill, *Choosing Effective Utility-Based Considerations*, Game AI Pro 3 ch. 13, corpus doc `gameaipro3-ch13-…`; the crate already cites it at `tier_cutoff` |
| Reporting state transitions to observers is the *notification* half of an action, and a transition that is observable in one direction must be observable in the other | same |

The paper's mechanism does not care which frame the message is emitted in; what it requires is that
enter/exit are a balanced pair. Arrival is a separate event in that vocabulary — the start of `run` —
and the crate already models it as its own field, `Doing::arrived_day`.

## Not this iteration's problem, measured and recorded

Needs still starve, and it is a **separate** cause:

| Setting | need2 at 0 | need3 at 0 |
|---|---|---|
| one plaza, everyone standing on it (zero travel) | 9.6% | 28.2% |
| village, 24 people | 46.4% | 78.2% |

Zero travel halves it, so walking costs are part of it — but needs still hit zero with no walking at
all. The remaining cause is activity choice: over 1 500 ticks the starter pack chose `snub` **589**
times and `gossip` **561**, together 82% of all starts, and neither satisfies any need. They are
gated on `ctx.grudge` and `ctx.tellable`, which stay high, so they out-compete `relax` (43) and `eat`
(35). That is the next iteration's item.

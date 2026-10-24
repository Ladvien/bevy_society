# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what the literature says?

Claim: gossip carries information, so a person without information should converse instead.

**Evidence.** The same definitions, the same seed, one consideration added:

| activity | before | after |
|---|---|---|
| `chat` | 8746 | **9312** |
| `gossip` | 15050 | 14160 |
| `snub` | 6004 | **1617** |

`chat` rises, `gossip` falls, both modestly — and `snub` falls by **73%**, which was not the goal
and is the largest effect. It follows from the chain already established: gossip interactions
produce insults, insults produce grudges, grudges drive snubs. Fewer gratuitous gossips, far fewer
snubs.

**Critique.** The `snub` collapse is a second-order effect I did not predict. It is a large change
to the society from a small change to one consideration, and the review should ask whether it is
*believable* or merely *less violent*. Checking: 1617 person-ticks of snubbing across 24 people over
17 days is about 4 ticks per person per day — a village with a few grudges, not a village without
them. Believable.

**Verdict: GREAT**, with the second-order effect named.

## Pass 2 — Zoom: what the numbers do at the edges

**Critique — the honest problem.** The cluster barely moved: `chat` 128 → 140. The gate cannot fix
what the decision note expected it to fix.

Reason, measured: the cluster generates **279 rumors**, so `tellable` is true for almost everybody
and the gate never closes. The gate can only differentiate where rumors are scarce — the village,
with 28 rumors among 24 people.

So the change is right and insufficient for the original complaint. That is recorded in
`research.md` and the decision note stays open with option 1 (a second need) recommended next. The
alternative — tuning `tellable` to bite in the cluster, e.g. requiring *two* spreadable rumors —
would be inventing a threshold to make a number move, which is what this loop exists to prevent.

**Verdict: NOT YET** for the cluster starvation; **GREAT** for the gate itself.

## Pass 2, second attempt — is the gate itself safe at the edges?

- A person with no rumors: gate 0, chats. Correct, and pinned by test.
- A person with one rumor: gate 1, gossips. Correct.
- A person whose rumor's subject is the listener: `tellable` still says yes, because the snapshot
  does not carry subjects. The gossip then finds nothing eligible and tells nothing — `Gossip::
  Nothing` — which wastes the activity but does not misreport. Named in the code comment.
- `idle` fallback: unaffected, and the degenerate-duration warning added last iteration still fires
  for `snub` at a 4-second day.

**Verdict: GREAT** for the gate, with the subject-blindness documented rather than hidden.

## Pass 3 — Fresh eyes: another scenario and population

**Critique.** The capture, which is the thing a visitor sees: `{"rumor_reach": 0.360, "friends": 80,
"rivals": 37, "barks": 3648}` — all four thresholds met, and the friend/rival balance is closer to a
real village than either previous run.

`rumor-crosses-a-cluster` still reaches 0.333, so the rumor still crosses the gap; the gate did not
break propagation.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — chat up, gossip down, snub down 73%; the second-order effect named |
| Zoom | **NOT YET** for the cluster's 1:24 ratio (279 rumors defeat the gate); GREAT for the gate's own correctness at every edge |
| Fresh eyes | GREAT — capture at all four thresholds, cluster reach intact |

## Next iteration's candidates

1. **The cluster's 1:24 starvation survives.** Option 1 from the decision note: a second need
   (`belonging` for chat, `social` for gossip). A schema change, so it needs the note revisited —
   and it is now the only remaining path to fixing this.
2. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, implicated in
   three iterations.
3. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
4. A log-capture harness so warnings can be asserted rather than read by hand.
5. `scale-1000` perf.

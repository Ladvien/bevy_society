# 2026-10-30-29 — research: hunger that takes precedence (`urgent` rank)

## The item

Pass 28's top backlog item, a defect recorded in the log: **hunger is not prepotent**. With bread on every shelf the 60-person
town still held 37 % of person-ticks at `food` < 0.05 and the 24-person village 16-19 %, because `eat` is one draw among many
by `weight · score`. No open issue (`gh issue list` returns nothing); the parked branches (passes 20-40) are older than `main`
and ignored. The release is time-gated (`v0.7.0` is 3.9 h old at the start of this pass).

## What the harness found first (HEAD, before any change)

`examples/evidence` gained `floor.txt` (this pass): for every villager-tick from game-day 2 on with `food` under 0.05 it records
what the person was doing and whether the household held a meal (a quarter loaf, what `eat` consumes), and it measures each
**spell of hunger**, `food` under 0.35 (the midpoint of `eat`'s `Logistic(-10, 0.35)`) until it is back over, in game-hours.
Village, seed 7, 17 game-days, 24 people, default pack (`evidence/diagnosis/village-s7-default/floor.txt`):

| of the person-ticks at the floor | share |
|---|---|
| the household **held a meal** | **96.8 %** (so supply is not the cause) |
| asleep / walking to bed | 25.8 % / 4.7 % |
| walking to eat | 19.6 % |
| `work` / walking to work | 17.2 % / 6.5 % |
| `craft`, `relax`, `play`, `compliment`, `gossip`, `snub`, `share`, `chat`, `wander`, `bake` and their walks | `craft` and its walks 6.4 %; the social and leisure ones 19.2 % (`bake`, `wander` 0.5 %) |
| at the table (`eat`) | 0.6 % |

About half of the floor (`work`, `craft`, the social and leisure activities and their walks: 17.2 + 6.5 + 6.4 + 19.2 = 49.3 %) is a person at 0.05 or less **choosing something other than a meal that is on the shelf**. The spells
are long: 316 finished, mean 12.4 h, median 9.5 h, p90 27.5 h, longest 86 h. The overlay of pass 28 (`--provisioning`) has the
same shape (18.8 % at the floor, spells mean 11.8 h, p90 25.1 h), so provisioning fixed the supply and left the precedence.

## Sources

- **Maslow 1943, "A Theory of Human Motivation", DOI 10.1037/h0054346** (corpus doc `10.1037_h0054346`, passages checked).
  "the most prepotent goal will monopolize consciousness and will tend of itself to organize the recruitment of the various
  capacities of the organism. The less prepotent needs are minimized, even forgotten or denied." And: "a satisfied need is not a
  motivator. It must be considered for all practical purposes simply not to exist." Two consequences for a utility scorer:
  a need that is *low* dominates the choice outright, not as one more factor to weigh against flattery; and a need that is
  *not* low takes no part. The bucket below is both.
- **Graham, "An Introduction to Utility Theory", Game AI Pro 1 ch. 9** (corpus doc `gameaipro1-ch09-introduction-utility-theory`,
  passage checked): "On The Sims, motives are bucketed based on their utility value. The highest scoring motives are grouped
  into a bucket, and none of the motives below are considered. ... Thus, a starving Sim will never even consider watching TV
  unless they fail to find anything that can solve their hunger." The fall-through clause matters: the bucket is a preference
  for what can solve the need, not a command to do something that cannot.
- **Dill, "Dual-Utility Reasoning", Game AI Pro 2 ch. 3** (corpus doc `gameaipro2-ch03-dual-utility-reasoning`, passage checked).
  Absolute utility (the rank) "is used to divide the options into categories and ensure that only options from the most
  important or most relevant category will be selected", relative utility picks inside the category; in Zoo Tycoon 2 the ranks
  "were coordinated by a simple FSM". The crate already has the ranks (`ActivityDef::rank`, used by `sleep`); what it lacks is a
  rank that follows the situation.
- **Wang, Chiu & Chiu 2023, "Humanoid Agents", DOI 10.48550/arXiv.2310.05418** (corpus doc `10.48550_arxiv.2310.05418`). A
  simulation of agents with basic needs: when a need is set to zero the agents "adapt their activities most when the basic
  needs of health (156 %), energy (56 %) and fullness (35 %)" are low, which the authors read as Maslow's low-level needs
  being satisfied first. The crate's village at the floor on `food` 19 % of the day, doing what it would have done fed, is the
  opposite of that finding.

## The model

`ActivityDef::urgent: Option<UrgentDef { need, below, rank }>`. At decision time an activity's rank is its own `rank`, **raised
to `urgent.rank` while the actor's `need` is under `below`**. Selection is unchanged otherwise: the highest rank with any
option above zero wins, `tier_cutoff` trims the tier, and one option is drawn by `weight · score`. So:

- a hungry person's `eat` is in a tier above `work`, `compliment` and `play`, and a person who is not hungry sees the pack
  exactly as before;
- an urgent activity with no viable option (no table tagged `food`, the table just failed) leaves the lower tiers to decide,
  which is Graham's fall-through and costs no code;
- `sleep` already has rank 1; an `urgent` rank of 1 on `eat` puts the two in one tier, drawn by score (a tired, hungry person
  at night does either), and an author who wants hunger over sleep writes rank 2;
- the crate names no need and no number: `below` is authored, a design default per pack (Maslow gives an order, not a level).

Which activities are "urgent for hunger" is stated by the pack rather than inferred from `satisfies`, for a reason this pass
measured (below): the activity that *provisions* the need (`bake`, which satisfies nothing itself) has to be in the bucket too.

## What the measurement said about where it can be used

`--urgent BELOW` (harness flag) puts `eat` in the bucket on any pack, village, seed 7, 17 game-days:

| pack | `below` | floor | spell mean / median / p90 (h) | bread a head at the end | friends / rivals / insults |
|---|---|---|---|---|---|
| default (clock-driven `bake`) | off | 19.4 % | 12.4 / 9.5 / 27.5 | 3.24 | 41 / 36 / 273 |
| default | 0.15 | 18.1 % | 11.0 / 7.8 / 13.4 | 0.70 | 47 / 28 / 260 |
| default | 0.25 | **26.9 %** | 12.6 / 6.7 / 17.3 | 0.29 | 76 / 17 / 185 |
| default | 0.35 | **52.2 %** | 14.2 / 6.2 / 13.9 | 0.00 | 66 / 2 / 149 |
| overlay (`--provisioning`) | off | 18.8 % | 11.8 / 9.0 / 25.1 | 1.20 | 50 / 30 / 262 |
| overlay | 0.15 | 3.6 % | 7.4 / 7.1 / 11.4 | 1.65 | 40 / 40 / 329 |
| overlay | 0.25 | **2.7 %** | 6.8 / 6.4 / 10.5 | 1.41 | 37 / 39 / 338 |
| overlay | 0.35 | 1.8 % | 6.0 / 6.0 / 9.5 | 1.60 | 47 / 33 / 335 |

On the **default** pack a prepotent `eat` is worse the more prepotent it is: the pack prices an empty shelf instead of gating
it (`starter.society.ron`'s own comment: "a meal from an empty larder walks to the table and restores no `food`"), and its
`bake` is a one-hour dawn errand outside the bucket, so a hungry person on a bare shelf keeps choosing a meal that feeds
nothing and never bakes (bread a head 3.24 → 0.00, floor 19 → 52 %). That is Graham's clause in reverse: the bucket was
filled with an option that cannot solve the need. On the **overlay**, where the shelf is in `eat`'s score and `bake` answers a
hungry bare household, the same bucket takes the floor from 18.8 % to 2.7 % and the longest spell from 55 h to 24 h. So the
feature is an engine capability and its first user is the provisioning overlay; the starter pack does not take it (it would
change the default for every host into the failure above, until the default pack provisions itself, which a protected
assertion blocks: `needs-decision/households-provision-themselves.md`).

## What believable output looks like

A villager meets `food` < `below` roughly once a day, finishes its activity, and goes to eat; it does not spend a working day at
`food` 0.02 with bread on the shelf. The floor share falls to a few percent (the residue is sleeping through a night's
decay, and a walk of two game-hours at the harness's 64-second day), spells shorten toward walk + meal, the shelf is no emptier
than before, and the social fabric (friends, rivals, insults, shares) stays inside its seed-to-seed range.

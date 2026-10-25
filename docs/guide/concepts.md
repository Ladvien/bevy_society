# Concepts

Four ideas explain almost everything surprising about this crate: ticks, snapshots, one application
point for script writes, and deterministic order.

## Ticks and game-days

The society does not advance every frame. It advances on **society ticks**, one per
`SocietySettings::tick_secs` of fixed-step time (0.25 s by default, which is sixteen 64 Hz steps).

```mermaid
flowchart LR
    Fixed[FixedUpdate step] --> Clock[Clock every step]
    Clock -->|tick due| Perceive
    Perceive --> Mind --> Decide --> Interact --> Propagate --> Relate --> Rules
    Rules --> Apply[apply_script_actions]
```

| Set | What runs there | On every fixed step? |
|---|---|---|
| `Clock` | the clock, needs/opinion sizing, and the drains for game-written input | yes |
| `Perceive` | perception and the script snapshot | tick only |
| `Mind` | emotions, mood, needs, memories | tick only |
| `Decide` | choosing an activity | tick only |
| `Interact` | arrivals, needs satisfied, social exchanges | tick only |
| `Propagate` | opinions, conformity, rumors | tick only |
| `Relate` | ties, balance, labels, factions, status, contact gossip | tick only |
| `Rules` | triggers and their scripts | tick only |

Every rate in `Tuning` is **per game-day**, and every duration is converted to game-days through
`SocietyClock::dt_days`. `SocietySettings::seconds_per_day` (240 by default) is what makes a game-day
long or short; it changes how much happens per real second, never what happens.

### Where the clock's time comes from

`SocietySettings::clock` chooses between two sources. The tick **rate** is the same either way — one
tick per `tick_secs` of fixed-step time, so simulation cost stays bounded — and only how much game time
a tick represents changes:

| `clock` | `dt_days` each tick | use |
|---|---|---|
| `ClockSource::Fixed` (default) | `tick_secs / seconds_per_day` | a society whose day is a constant length |
| `ClockSource::Host` | the change in `HostClock::now_days` since the previous tick, never negative | a game whose day has phases of unequal length |

With `Host`, insert a `HostClock` resource and write `now_days` every frame — the crate never inserts
one. A host clock that stands still is a pause: `dt_days` is 0, so no need decays and no walk times out.
One that jumps forward (a phase skipped) applies the whole jump in one tick with no catch-up ticks, and
every `Hour` trigger the jump crossed fires once. One that goes backward (a reload) takes the host's
time and applies no negative `dt`, so nothing fires twice. A `Host` clock with **no** `HostClock`
freezes and warns once, rather than inventing time.

## Contact gossip

Two people who share a place talk even when neither has chosen the other as an activity target, but only
**when the speaker has news the listener does not have**. The gate is `Knowledge::best_telling`, so a pair
with nothing between them says nothing at all — a village where people share a tavern bench and simply
ignore each other, rather than one where standing near someone makes you friends. Goffman's *civil
inattention* (1963, via Cañigueral & Hamilton 2019, DOI 10.3389/fpsyg.2019.00560) is the shape: acknowledge
presence, do not treat it as special interest.

Pacing matters more than it looks. Every telling to a listener who **already knows** stifles the teller
(Daley & Kendall 1964, DOI 10.1038/2041118a0), so a contact pass that runs too often spends the village's
spreaders faster than it spreads news. `Tuning::contact_gossip_cooldown_days` (2.0 by default) is set by
measurement: on the shipped village, rumor reach is **0.875** at 2 days, **0.375** at half a day, and
**0.583** — exactly the no-contact-gossip number — at 6.

Because a society tick is many fixed steps apart, Bevy's message buffers (two swaps) would drop a
message written between ticks. So every **game-written input message** — `PlantRumor`, `Interact`,
`FireTrigger` — is drained by an ungated system in `Clock`. A game may write one at any time and it is
never lost.

## Snapshots

Scripts see the society as it stood at the **start of the tick**. `Perceive` freezes every person's
personality, mood, emotions, needs, signals, opinions, ties, knowledge, memberships, standing and
activity into one immutable snapshot, and every `ctx` call in that tick reads it.

Two consequences follow, and both are deliberate:

- A script cannot observe its own writes. `fx.adjust(a, b, "affinity", 1.0)` followed by
  `ctx.affinity(a, b)` in the same script still reads the old value.
- The order in which systems run cannot change what a script sees. Only the snapshot's contents
  matter, and the snapshot was built before any of them ran.

## One application point for script writes

A script never touches the world. `fx.*` queues a write; the queue is drained by **one exclusive
system after `Rules`**, once per tick. So:

- No script can change what another script in the same tick reads, even if the other script runs
  later.
- A write queued during `Update` (a dialogue node effect) lands at the same place as one queued during
  `FixedUpdate`: at the end of the next tick.
- A failed script applies **none** of its actions. A script cannot half-apply.

## Determinism

The same seed and the same spawn sequence give the same society, bit for bit. Three rules make that
true:

1. Every simulation loop walks `PersonIndex::sorted` (ascending `PersonId`, which is spawn order).
   Nothing iterates a `HashMap` or `HashSet`.
2. The only randomness is `SocietyRng`, a ChaCha8 stream seeded from `SocietySettings::seed`. Draws
   use `bevy_emerge_rng::DetRng` (`unit()`, `below(n)`).
3. `PersonId`s are handed out by an `On<Add<Person>>` observer, so they follow spawn order.

A script's `ctx.rand()` comes from a generator seeded once per tick from the same stream, so scripts
are deterministic too.

## Scale

Perception runs through a uniform grid: one cell per `perception_radius`, with a person's neighbours read
from the nine cells around its own. Measured with `examples/benchmark` on an M1 Max, release build, starter
pack, `tick_secs = 0.25`:

| persons | ms per society tick | amortised per 60 fps frame |
|---|---|---|
| 100 | 0.6 | 0.04 ms (0.2%) |
| 1 000 | **13.5** | 0.9 ms (5.3%) |
| 2 000 | 49.9 | 3.1 ms (19%) |
| 5 000 | **245.1** | 15.3 ms (**92%**) |

A thousand people costs about 5% of a 60 fps budget; five thousand costs most of a frame.

**The cost is the crowd, not the search.** An earlier version scanned every pair, and this section said the
pair count was the scale curve. It is not: measured with direct timers, the all-pairs scan was **36 ms of a
254 ms tick** at 5 000 people. Substituting the grid is bit-identical — same neighbours, same order, same
society — and saves 3-7%.

What scales is **scoring**. Every `Person`-targeted activity is scored against every neighbour: at 5 000
people on the benchmark's 40 m disc an 8 m radius holds **182** neighbours, so a tick runs roughly
`5 000 x 4 x 182 = 3.6M` consideration evaluations. The density is real, so no index removes those. A host
that needs thousands of people in contact wants fewer `Person`-targeted activities, a smaller
`perception_radius`, or a decision that samples neighbours rather than scoring them all.

`ms per tick` is the number that matters, not `ms per update`: a society tick happens once per `tick_secs`,
and everything else in the crate's cost scales with it.

## Walking and obstacles

A person with a `SocialMover` and a `Destination` walks toward that point, deflected by ORCA so two movers
pass each other with a gap. Anything round and solid is an `Obstacle`:

```rust
commands.spawn((Name::new("tavern"), Obstacle { radius: 2.0 }, Transform::from_xyz(6.5, 0.0, -5.0)));
```

The obstacle's centre and radius become a hard wall half-plane in the solver — a mover may slide along the
edge or walk away, never accelerate into it — and `push_out_of_obstacles` resolves contact afterwards, which
`bevy_orca` leaves to the caller. Without an `Obstacle` a person walks straight through: measured, a walker
crossed a block whose face was 1.5 m from its centre and reached **0.01 m**; with one it stopped at **2.35 m**.

**Give an activity's place a radius smaller than `interaction_radius`.** A place whose half-width exceeded
`interaction_radius` could not be used at all: being "arrived" required standing inside it. The demo scene
keeps its stalls at a 0.6 m half-width with a 1.3 m gather ring, so a person is outside the stall and within
reach at the same time.

## Ids

Needs, signals, topics, deeds, factions, activities and interactions are interned at compile time
into `NeedId(u16)`, `TopicId(u16)`, `SignalId(u16)` and so on, in definition order. Runtime state is a
`Vec` indexed by the id, so looking up a person's opinion on a topic is an array index. Scripts and
definitions name things by string; the snapshot and `SocietyDefs` map the name to the id, and an
unknown name is an error that names it.

## Why ties are a component, not Bevy relationships

A tie is directed (A may like B more than B likes A), weighted (four numbers), and many-to-many (every
person to every other person they have met). Bevy's relationships model an edge as a component on the
target, which fits none of those. So `Ties` is one component holding a `Vec<Tie>` sorted by the
target's bits: lookups are binary searches, iteration is deterministic, and the whole neighbourhood of
one person is one cache-friendly read.

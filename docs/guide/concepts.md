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
from the nine cells around its own.

Measured with `examples/benchmark` on an M1 Max, release build, starter pack:

| persons | ms per society tick | amortised per 60 fps frame |
|---|---|---|
| 100 | 0.74 | 0.04 ms (0.3%) |
| 1 000 | **28.4** | 1.8 ms (11%) |
| 2 000 | 100.6 | 6.3 ms (38%) |

**Quote the minimum, not the mean.** These are the *fastest* window out of 5-9 repeated runs, and that is
deliberate: on a machine shared with other work the mean measures the load, not the code. Measured on one
binary, one afternoon:

| persons | fastest window | median of 9 windows | spread |
|---|---|---|---|
| 100 | **0.74** | 1.08 | 103% |
| 1 000 | **28.4** | 60.4 | **100%** |
| 2 000 | **100.6** | 140.1 | 49% |

The fastest window reproduced to **0.4%** across separate invocations (100.55 and 100.93 ms at 2 000), while
the median moved 18%. A single number with no repetitions is not a measurement, which is why
`examples/benchmark` now takes `--reps` and prints `ms_per_tick_min`, `ms_per_tick_median` and `spread_pct`.
**If `spread_pct` is above about 20, the machine was too busy for any of these figures to mean anything.**

Re-checked 2026-10-25 on the same M1 Max with `--reps 3`: fastest windows **0.78**, **28.9** and **101.2** ms at 100, 1 000 and 2 000 people, within 5%, 2% and 1% of the table. The spreads were 17%, 44% and 31% (the machine was shared, so by the rule above those runs say little beyond the minimum), and the table is unchanged.

Re-measured 2026-10-30 on v0.6.0 (a 24-thread x86-64 Linux host, release build, starter pack, `--ticks 100 --settle 300 --reps 5`) after about twenty feature passes. The host's load average was 11 to 38 while it ran, so every spread is above the 20% line and these are upper bounds on the true cost, not a table to quote:

| persons | fastest window (ms) | median (ms) | spread |
|---|---|---|---|
| 100 | 0.86 | 1.10 | 28% |
| 1 000 | 43.5 | 57.4 | 55% |
| 2 000 | 155.8 | 206.1 | 134% |
| 5 000 | 534.8 | 702.8 | 40% |

Per million pairs the cost falls from 222 ms at 100 people to 56 ms at 5 000, so a tick is not quadratic in the crowd; at 1 000 people it is about 1.5 times the M1 Max figure above. A 5 000-person society costs about half a second per society tick on this host: budget it with a `tick_secs` of a second or more, or fewer `Person`-targeted activities. Re-run `examples/benchmark` on your own hardware before planning around any of these figures.

An earlier version of this table said **13.5 ms** at 1 000 and **245.1 ms** at 5 000. Those cannot be
reproduced on this hardware: the fastest window measured here at 1 000 is 28.4 ms, more than twice that figure,
and the discrepancy was carried for nine iterations because the only tool available reported a single mean and
there was never a quiet enough moment to trust it. The numbers above replace them.

A thousand people costs about a tenth of a 60 fps budget; two thousand more than a third.

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

**A crowd walking to one place needs room, and ORCA needs a symmetry kick to pack it.** When everyone shares a
`Home`, the whole village converges on one coordinate at evening. ORCA cannot decide whether to pass left or
right at an exactly head-on meeting, so without a nudge the walkers' wishes cancel and they stall short — measured,
**2.0–4.0 m** away, for the whole `travel_timeout_hours`. The crate's `NUDGE` breaks that symmetry; at the old
`0.05` it was too weak to, and a village of 24 interrupted **40** activities against **4** with steering switched
off entirely. It is now `0.30`, which brings it to **19**.

Those remaining 19 are geometry, not steering: twenty-four bodies of 0.35 m radius cannot all fit inside a 1.5 m
arrival circle. So **give a shared place enough room** — spread it out, or use several — because the arrival test
is "within `interaction_radius` of the place", and a place everyone shares is a place everyone must fit inside.

**Give an activity's place a radius smaller than `interaction_radius`.** A place whose half-width exceeded
`interaction_radius` could not be used at all: being "arrived" required standing inside it. The demo scene
keeps its stalls at a 0.6 m half-width with a 1.3 m gather ring, so a person is outside the stall and within
reach at the same time.

**A host that owns the path wants a stall timeout, not a travel timeout.** Under `Locomotion::Host` leave `travel_timeout_hours` as `None`: a walk across a town takes game-hours and a total limit would cut walks still being made. The cost is that a destination nobody can reach (a point inside a building's collision boxes, a ring of standing people the solver treats as a wall) never ends, and the person aims at it for the rest of the run. `Tuning::stall_timeout_hours: Some(h)` bounds the *absence of progress* instead: the crate tracks the closest the person has been to its target and when that last improved by more than `stall_epsilon` (0.25 m), and ends the activity interrupted, as a travel timeout does, once `h` game-hours pass with no improvement. A slow walker improves every few ticks and is never cut; a person queued behind a crowd is, after `h`, so pick `h` for how long a queue may be. It defaults to `None`. A save carries the tracker, so a restored walker keeps its window. `TravelFailed` is unaffected.

**A grievance can cost one blow per period.** `Tuning::retaliation_refractory_hours: Some(h)` stops a person who has just attempted a harmful interaction (a deed with `harm < 0`, accepted or refused) from choosing another harmful activity against the same target for `h` game-hours; without it a grudge, which outlives the insult it answers, re-fires the moment each exchange ends. It defaults to `Some(6.0)` (`None` turns it off), a host's `Interact` is never refused by it, and a save carries it.

**A grievance fades faster toward someone still trusted.** `ctx.grudge(a, b)`, which the starter pack's `snub` reads, weighs each of `a`'s bad memories about `b` at its age times `1 + forgiveness · trust(a → b)` (`Tuning::forgiveness`, default `12.0`; McCullough et al. 2010, DOI 10.1037/a0019349; Burnette et al. 2012, DOI 10.1177/0146167211424582; Strelan, Karremans & Krieg 2017, DOI 10.1111/bjso.12173). A slight from someone `a` trusts at 0.5 fades seven times as fast as on the plain 138-hour recency curve; one from someone who keeps offending, and so has lost `a`'s trust, fades at close to that plain rate. `0.0` turns it off. `Memories::grudge`, which reads no tie, is the unforgiving value.

**A refusal is remembered for a while.** `Tuning::rebuff_memory_hours: Some(h)` keeps a person who has just been turned down from choosing the person who refused them as the target of any affiliative `Person` activity (`chat`, `gossip`, `compliment`, `play`, `share`: anything whose deed does not harm) for `h` game-hours, so the next approach goes to someone else. Excluded people do not seek reconnection with the specific people who excluded them (Maner, DeWall, Baumeister & Schaller 2007, DOI 10.1037/0022-3514.92.1.42); without the memory a person's nearest neighbour who keeps refusing is chosen again at the next decision. It defaults to `Some(12.0)`, a design default (no paper gives an interval); `None` turns it off and is exactly the old behaviour. A harmful activity is held by the retaliation refractory instead, not by this; a host's `Interact` is never refused by it; a save carries it (`PersonSave::rebuffed`, by `PersonKey`). It changes whom a person asks, not how often they are refused: the refused share of approaches stays 35-37%. `docs/research/rebuff.md`.

**A place a walk failed to reach can be forgotten for a while.** A `Place` target is always the nearest tied place, so after a stall or timeout the person chose the same unreachable place again. `Tuning::failed_place_memory_hours: Some(h)` drops that place from the person's candidates for `h` game-hours, so the next choice is the next-nearest place, or another activity if none is left. It defaults to `None`; a save carries it (by `PlaceKey`). A completed walk, a `Person` target and a dormant person's suspended walk are never remembered.

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

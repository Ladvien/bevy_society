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
| `Relate` | ties, balance, labels, factions, status | tick only |
| `Rules` | triggers and their scripts | tick only |

Every rate in `Tuning` is **per game-day**, and every duration is converted to game-days through
`SocietyClock::dt_days`. `SocietySettings::seconds_per_day` (240 by default) is what makes a game-day
long or short; it changes how much happens per real second, never what happens.

Because a society tick is many fixed steps apart, Bevy's message buffers (two swaps) would drop a
message written between ticks. So every **game-written input message** — `PlantRumor`, `Interact`,
`FireTrigger` — is drained by an ungated system in `Clock`. A game may write one at any time and it is
never lost.

## Snapshots

Scripts see the society as it stood at the **start of the tick**. `Perceive` freezes every person's
personality, mood, emotions, needs, opinions, ties, knowledge, memberships, standing and activity into
one immutable snapshot, and every `ctx` call in that tick reads it.

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

Perception is brute force: every person against every other, once per tick. That is the chosen design,
and its cost is quadratic. Measured with `examples/benchmark` on an M1 Max, release build, starter
pack, `tick_secs = 0.25`:

| persons | ms per society tick | pairs per tick | amortised per 60 fps frame |
|---|---|---|---|
| 100 | 0.5 | 4 950 | 0.03 ms (0.2%) |
| 1 000 | **12.4** | 499 500 | 0.83 ms (4.9%) |
| 5 000 | **350.7** | 12 497 500 | 23.4 ms (**140%**) |

So a thousand people costs about 5% of a 60 fps budget — usable, but not free, and the amortisation
assumes one society tick every fifteen frames. Five thousand costs more than a frame: that is the
point at which a spatial index stops being an optimisation and becomes the only way to run it.

`ms per tick` is the number that matters, not `ms per update`: a society tick happens once per
`tick_secs`, and everything else in the crate's cost scales with it.

## Ids

Needs, topics, deeds, factions, activities and interactions are interned at compile time into
`NeedId(u16)`, `TopicId(u16)` and so on, in definition order. Runtime state is a `Vec` indexed by the
id, so looking up a person's opinion on a topic is an array index. Scripts and definitions name things
by string; the snapshot and `SocietyDefs` map the name to the id, and an unknown name is an error that
names it.

## Why ties are a component, not Bevy relationships

A tie is directed (A may like B more than B likes A), weighted (four numbers), and many-to-many (every
person to every other person they have met). Bevy's relationships model an edge as a component on the
target, which fits none of those. So `Ties` is one component holding a `Vec<Tie>` sorted by the
target's bits: lookups are binary searches, iteration is deterministic, and the whole neighbourhood of
one person is one cache-friendly read.

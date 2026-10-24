# Scripting

Triggers, conversation rules, node effects, interaction conditions and considerations can run Rhai.
A script reads the society through `ctx`, and writes to it through `fx`.

## What a script sees

Every evaluation gets these constants:

| Name | Type | Meaning |
|---|---|---|
| `actor` | `i64` | the event's actor's `PersonId`, or −1 |
| `target` | `i64` | the event's target's `PersonId`, or −1 |
| `subject` | `i64` | the rumor subject's `PersonId`, or −1 |
| `rumor` | `i64` | the rumor's id, or −1 |
| `accepted` | `bool` | bound **only** on an interaction trigger |

Absent ids are −1 rather than an optional type, so `actor >= 0` is the test. A `PersonId` is what
`ctx.persons()` returns and what every `ctx` / `fx` call takes; a script never names an entity.

Numbers are `f32` (the engine runs with `f32_float`), so write `0.5`, not `0.5f64`.

## `ctx`: reading the tick-start society

Every `ctx` call reads the snapshot frozen in `Perceive`, so a script sees the society as it stood
before any of this tick's writes. Names are lowercase strings and an unknown name is a runtime error
that names it.

| Call | Returns |
|---|---|
| `ctx.affinity(a, b)` | liking in `[-1, 1]`; 0 with no tie |
| `ctx.trust(a, b)` | trust in `[0, 1]`; 0.5 with no tie |
| `ctx.respect(a, b)` | respect in `[-1, 1]`; 0 with no tie |
| `ctx.familiarity(a, b)` | familiarity in `[0, 1]` at the tick start; 0 with no tie |
| `ctx.label(a, b)` | `"stranger"`, `"acquaintance"`, `"friend"` or `"rival"` |
| `ctx.grudge(a, b)` | weight of `a`'s bad memories about `b`, `[0, 1]` |
| `ctx.standing(a)` | respect conferred by familiar persons |
| `ctx.mood(a)` | `#{ p, a, d }` |
| `ctx.emotion(a, kind)` | summed intensity of one emotion, `[0, 1]` |
| `ctx.need(a, id)` | a need's level, `[0, 1]` |
| `ctx.opinion(a, topic)` | an opinion, `[-1, 1]` |
| `ctx.trait(a, name)` | a personality trait, `[0, 1]` |
| `ctx.knows(a, rumor)` | whether `a` knows that rumor |
| `ctx.tellable(a)` | whether `a` knows a rumor it is still spreading (the condition a gossip gate needs) |
| `ctx.member(a, faction)` | whether `a` belongs to that faction |
| `ctx.faction_size(faction)` | how many persons belong to it |
| `ctx.name(a)` | the person's `Name`, or `"someone"` |
| `ctx.doing(a)` | the activity's name, `""` when idle |
| `ctx.persons()` | every live person's id, ascending |
| `ctx.hour()` | the hour of the game-day, `[0, 24)` |
| `ctx.day()` | whole game-days elapsed |
| `ctx.rand()` | a uniform draw in `[0, 1)` |

Emotion names are `joy`, `distress`, `hope`, `fear`, `pride`, `shame`, `admiration`, `reproach`,
`gratitude`, `anger`, `liking`, `disliking`. Trait names are `openness`, `conscientiousness`,
`extraversion`, `agreeableness`, `neuroticism`.

## `fx`: queuing writes

`fx` never touches the world. Each call queues an action; one exclusive system applies the queue after
`Rules`, once per tick. A script that fails applies **none** of its actions.

| Call | Effect |
|---|---|
| `fx.adjust(a, b, field, delta)` | moves a tie value; `field` is `affinity`, `trust`, `respect` or `familiarity` |
| `fx.emote(a, kind, intensity)` | appraises an emotion |
| `fx.need(a, id, delta)` | moves a need, clamped to `[0, 1]` |
| `fx.opinion(a, topic, delta)` | moves an opinion, clamped to `[-1, 1]` |
| `fx.join(a, faction)` / `fx.leave(a, faction)` | changes a membership |
| `fx.rumor(origin, subject, deed, truth)` | starts a rumor, known by `origin` at full belief |
| `fx.interact(a, b, interaction)` | writes `Interact`, resolved next tick |
| `fx.bark(a, text)` / `fx.think(a, text)` | writes `SocietyBark` |
| `fx.remember(a, b, text, valence, importance)` | records a `Custom` memory; `b` may be `-1` |
| `fx.converse(npc, player)` | opens a conversation (needs the `dialogue` feature) |
| `fx.emit(name, data_map)` | writes `ScriptEvent` with the event's `actor` and `target` |

Clamps follow the model: affinity and respect to `[-1, 1]`, trust to `[0, 1]`. A familiarity delta
moves the familiarity **peak** and restamps the contact, so `ctx.familiarity` reads at least the
clamped peak on the next tick.

`fx.emit`'s payload may contain only booleans, integers, floats and strings. Any other Rhai type is a
runtime error at `emit` time, so a script cannot smuggle a struct into a game event.

## Writing a trigger

```ron
(
    id: "watch_stops_theft",
    on: NormViolated("steal"),
    if: Some("ctx.member(actor, \"watch\")"),
    then: "fx.bark(actor, \"Stop, thief!\")",
)
```

A condition that fails to evaluate counts as **false** and writes a `ScriptError`; it does not stop the
other triggers.

## Errors and limits

Every evaluation runs sandboxed: 10 000 operations, 16 call levels, 1 KiB strings, 256-element arrays
and 64-entry maps. A script that exceeds a limit fails rather than hanging the tick.

A failure — a runtime error, a wrong return type, or the operation limit — logs
`bevy_society: script <owner> failed: <message>` and writes `ScriptError { owner, message }`. A failed
condition counts as false, a failed consideration scores 0, and a failed effect applies none of its
queued actions.

## Firing your own events

Write a message:

```rust,ignore
app.world_mut().write_message(FireTrigger {
    name: "poke".to_owned(),
    actor: Some(villager),
    target: None,
});
```

and the `Custom("poke")` triggers run, with `actor` bound. `FireTrigger` is drained on every fixed
step, so a game may write it from `Update`.

## When triggers run

Events are reported in a fixed order, so the same inputs give the same society:

1. `Tick`, actor −1.
2. `Hour(h)` for each hour crossed since the previous tick, wrap-aware, actor −1.
3. Queued events (interactions, activities, dialogue nodes), in push order.
4. `TieLabelChanged`, in message order.
5. `NormViolated`, actor = actor, target = witness.
6. `NeedBelow`, persons in id order and needs in id order.
7. `RumorReach`, rumors in id order, the first time a living rumor of that deed reaches its fraction.

Within one event, triggers are evaluated in definition order. `NeedBelow` re-arms only once the need
is back above `below + 0.05`, so a level hovering at the threshold does not fire every tick.

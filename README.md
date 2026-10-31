# bevy_society

Give your Bevy world a living society: villagers who remember, gossip, take sides, fall out and make up — and who you can script.

[![CI](https://github.com/Ladvien/bevy_society/actions/workflows/ci.yml/badge.svg)](https://github.com/Ladvien/bevy_society/actions/workflows/ci.yml)

## The demo

![A rumor spreads through a village](https://github.com/Ladvien/bevy_society/raw/main/docs/media/village.gif)

One afternoon in a village of twenty-four people: a weaver tells the rumor that Bram stole bread, and in the six game-hours the clip covers (14:00 to 20:00) it passes to one more villager — the plate in the corner reads `known by 2/24` on the last frame. The same run recorded twice as long, twelve game-hours, is still at 2, and which weaver tells matters: of ten tellers tried, four passed it on inside six hours and six did not (`docs/DEMOS.md`), so this clip tells it from one of the four. A rumor crosses a village in game-days, not hours, because each telling is one choice among many and the Daley–Kendall rule stifles a teller whose listener already knew. Tie lines show friendships green; the magenta ring marks whoever knows the rumor.

Every clip below is the same village, same seed, same run — only the overlay changes, except **Goods**, which also opens with bread handed out and says so. None of them draws a rivalry: the plate reads `Rivalries 0` from first frame to last, because six game-hours is not long enough for one to form. A longer run does form them — 28 directed rival ties against 62 friend ties at seed 7 over 17 game-days (`examples/evidence`, `docs/loop/runs/2026-10-30-25/eval.md`). The clips were recorded on 2026-10-25, before `Tuning::forgiveness` (2026-10-30) changed how the settled village's ties form; they show the village as it was then.

### One clip per feature

| | |
|---|---|
| ![Mood](https://github.com/Ladvien/bevy_society/raw/main/docs/media/mood.gif) **Mood** — a halo on every villager. Green is pleased, amber neutral, red displeased; halo size is arousal. The colour scale is normalised to the ±0.45 this village actually spans (`MOOD_SPAN`), so a small mood shift is a visible colour change rather than every villager the same olive. | ![Opinion](https://github.com/Ladvien/bevy_society/raw/main/docs/media/opinions.gif) **Opinions** — one bar per villager on the honesty topic, green above the midpoint and red below, length by strength, all drawn from a fixed origin so a near-neutral opinion is still legible. |
| ![Needs](https://github.com/Ladvien/bevy_society/raw/main/docs/media/needs.gif) **Needs** — a full green halo is a fed villager, a small red one is hungry. | ![Ties](https://github.com/Ladvien/bevy_society/raw/main/docs/media/ties.gif) **Relationships** — the tie graph alone: green friendship, red rivalry, brightness by familiarity, and only for pairs already familiar. Every line in this clip is green — the village has no rivalry yet. |
| ![Factions](https://github.com/Ladvien/bevy_society/raw/main/docs/media/factions.gif) **Factions** — orange millers, green weavers, violet watch, no ties, so the group structure is the only thing on screen. | ![Goods](https://github.com/Ladvien/bevy_society/raw/main/docs/media/goods.gif) **Goods** — a gold ring is a villager holding bread, sized by how much; a small dark one holds none; a white line is a loaf being shared. The recording opens with every other villager holding four loaves (the village's own stores sat near zero when this was recorded, so a clip from the settled state had nothing to show; they no longer do — a 24-villager village holds about five loaves a head by its seventh game-day, and from this release every person starts with a loaf), and the plate reads `0 loaves shared · 12 hold bread` on the first frame and `5 loaves shared · 17 hold bread` on the last. The hero clip above is the same run with every overlay on. |

`docs/DEMOS.md` is the recipe: the scene each clip is recorded from, the camera, the encode, and what a visual review has caught in each rebuild.

## What you get

- **People with a mind** — Big Five personalities, moods on a pleasure/arousal/dominance axis, an OCC emotion set, needs that decay, and memories that fade. [personality-and-mood](https://github.com/Ladvien/bevy_society/blob/main/docs/research/personality-and-mood.md), [memory](https://github.com/Ladvien/bevy_society/blob/main/docs/research/memory.md)
- **Daily life and trades** — utility-AI decisions over tagged places and other people, with travel, arrival, durations and need satisfaction. A **task is not a skill**: an activity can declare a `profession`, and a `Skill("craft")` consideration reads the trait that capability comes from, so who does the skilled work is decided by comparative advantage rather than by assignment. Nothing is gated — an unskilled villager can still take the task, at a lower score. [needs-and-utility](https://github.com/Ladvien/bevy_society/blob/main/docs/research/needs-and-utility.md) A **full place costs** instead of vanishing: a place with no free seat keeps `1 - Tuning::crowding_penalty` of its score, so a villager with an alternative goes elsewhere and one with none still arrives, while `capacity: 0` is closed. Measured, this holds a village's rival density at 2.9-6.1% of pairs as it grows from 24 to 48 people in a 30-seat village, where the old veto took it to 23% and lost the working day. [capacity](https://github.com/Ladvien/bevy_society/blob/main/docs/loop/runs/2026-10-25-10/eval.md)
- **Relationships** — directed ties carrying affinity, trust, respect and familiarity, with power-law forgetting, structural balance, a Dunbar-style cap and friend/rival labels. Affinity is **clamped to `±Tuning::affinity_ceiling` (0.35)** and a `Friend` needs trust as well, so a large town with daily contact keeps friendship scarce: measured, 60 people over 30 game-days reach 155 friend ties of 3 540 ordered pairs (**4.4%**). [relationships](https://github.com/Ladvien/bevy_society/blob/main/docs/research/relationships.md), [the measurement](https://github.com/Ladvien/bevy_society/blob/main/docs/loop/needs-decision/friendship-has-no-diminishing-return.md)
- **Opinions** — bounded-confidence influence, stubborn anchors and conformity to a clear group majority. [opinion-dynamics](https://github.com/Ladvien/bevy_society/blob/main/docs/research/opinion-dynamics.md)
- **Rumors and reputation** — gossip with the Daley–Kendall stifling rule, belief scaled by trust, witnesses who judge deeds and hold grudges. [rumor-and-reputation](https://github.com/Ladvien/bevy_society/blob/main/docs/research/rumor-and-reputation.md)
- **Factions and status** — in-group affinity, standing as conferred respect, contests and intergroup contact. [groups-and-status](https://github.com/Ladvien/bevy_society/blob/main/docs/research/groups-and-status.md)
- **Crowd fields** — a stigmergy grid of presence and commotion per cell, readable as a `Field(Crowd)` consideration and walkable as a `Gradient` target, so an activity can be "join the crowd" without naming a place. [needs-and-utility](https://github.com/Ladvien/bevy_society/blob/main/docs/research/needs-and-utility.md)
- **Scripts** — Rhai triggers over a per-tick snapshot, with a `ctx` reader and an `fx` writer that cannot see its own writes. [scripting](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/scripting.md)
- **Dialogue** — barks and conversations through `bevy_speech_bubbles`. [dialogue](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/dialogue.md)
- **Decision traces** — why a person chose what they did: every option weighed, its score, and how far the chosen one fell short of the best. Insert a `DecisionTrace` and read it; recording changes no outcome. [decisions](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/decisions.md)
- **Obstacles** — solid buildings that walkers path around, as hard walls in the ORCA solver, with a resolver that pushes anyone who lands inside back out. [getting started](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/getting-started.md#solid-buildings-obstacle)
- **A basic economy** — goods as quantities rather than currency, a per-person store, `produces`/`consumes` on a task, a price per good **read from how full the village's stores are, per head** — empty is double, full is half, the shape the corpus's own game-economy model uses ([economics](https://github.com/Ladvien/bevy_society/blob/main/docs/research/economics.md)) and readable from a script as `ctx.price("bread")`, plus **goods that change hands**: an interaction can `actor_gives` / `target_gives` a good, and a `Surplus` consideration scores a transfer by the joint surplus of two stores. In the village that is sharing — measured over nine seeds, it cuts the spread of bread across the village by 38% — and the engine does a two-way swap too. Nothing gates: an empty store never stops anyone eating — but it stops the meal **feeding** them, because a task's `satisfies` is paid in proportion to what its `consumes` actually found, so a meal from an empty larder costs the walk and restores nothing while half a loaf feeds half. Measured, that closed a village fed by arithmetic: 4.80 loaves baked a game-day against 21.06 meals, ~77% of them out of an empty shelf. A person starts holding each good's authored `initial` (the starter pack gives one loaf), so a fresh village is fed at its first dawn; with every larder empty, 39% of its second game-day's villager-ticks were spent with `food` below 0.05. [economics](https://github.com/Ladvien/bevy_society/blob/main/docs/research/economics.md) · [authoring](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/authoring.md#goods-stores-and-trade)
- **Persistence** — the whole society captured and restored through `serde`: ties, memories, rumors, the clock, the random stream and every trigger's bookkeeping, keyed by your own ids, so the ticks after a reload are the ticks that would have followed. [persistence](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/persistence.md)

Every formula traces to a paper; every rate is a named field on `Tuning`; every default either comes from a paper or says it is a design default.

## Install

```toml
[dependencies]
bevy_society = "0.4"
```

## Quick start

This is the crate's own doctest, and `tests/readme.rs` fails if the two ever differ.

```rust,no_run
use bevy::prelude::*;
use bevy_society::{
    Autonomous, Home, Person, Place, PlaceTags, SocietyClock, SocietyDefs, SocietyPlugins,
    SocietySettings,
};

let defs = SocietyDefs::starter().expect("the starter pack is valid");
// A headless app has no frame clock, so `Time<Fixed>` never advances on its own. Without this the
// society takes zero ticks however many updates you run.
let step = std::time::Duration::from_secs_f64(1.0 / 64.0);

let mut app = App::new();
app.add_plugins((MinimalPlugins, TransformPlugin))
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(step))
    .insert_resource(SocietySettings::default())
    .insert_resource(defs);
// `DialoguePlugin` needs a speech-bubbles plugin over a camera, which this snippet has not.
let plugins = SocietyPlugins.build();
#[cfg(feature = "dialogue")]
let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
app.add_plugins(plugins);

let well = app
    .world_mut()
    .spawn((Name::new("well"), Place { capacity: 6 }, PlaceTags(vec!["gathering".into()]), Transform::default()))
    .id();
for name in ["Bram", "Alys", "Corin"] {
    app.world_mut()
        .spawn((Person, Autonomous, Name::new(name), Home(well), Transform::default()));
}

for _ in 0..600 {
    app.update();
}
// The society advanced: this line is what proves the fixed step was pinned.
assert!(app.world().resource::<SocietyClock>().tick > 0);
```

Run it headlessly with `cargo run --example quickstart`. The example sets a short 16-second game-day, so 600 updates are enough for the village to start talking; on this code it reports `229 social exchanges, 112 accepted`.

The full village, with balloons and the debug server:

```
cargo run --example village -- /System/Library/Fonts/Supplemental/Arial.ttf
```

Font paths: macOS `/System/Library/Fonts/Supplemental/Arial.ttf`; Linux `/usr/share/fonts/liberation/LiberationSans-Regular.ttf`; Windows `C:\Windows\Fonts\arial.ttf`.

## Cargo features

| feature | default | turns on |
|---|---|---|
| `dialogue` | yes | `DialoguePlugin`, `TalkTo`, barks and conversations through `bevy_speech_bubbles`. The host adds `SpeechBubblesPlugin` over its own camera. |
| `steering` | yes | `SteeringPlugin`: `SocialMover` walking under ORCA (`bevy_orca`), and `Obstacle`. |
| `fields` | yes | `SocialFieldPlugin`: crowd presence and commotion grids and per-faction presence grids (`bevy_stigmergy`), the `Field` consideration and the `Gradient` target. |
| `serde` | no | `SocietySave`, `capture` and `restore`: save and load the whole society. |
| none (`--no-default-features`) | | the core: mind, daily life, relationships, opinions, rumors, reputation, factions, scripts and `DecisionTrace`. |

## A tour

### People

```rust,ignore
app.world_mut().spawn((
    Person,            // brings personality, mood, emotions, needs, signals, ties, opinions, knowledge,
    Autonomous,        // memories, memberships and status
    Name::new("Bram"),
    Home(well),
    Transform::from_xyz(2.0, 0.0, 0.0),
));
```

A player character is a `Person` without `Autonomous`: it does only what the game tells it.

### Needs and daily life

```ron
(
    id: "sleep",
    target: Home,
    duration_hours: 8.0,
    rank: 1,
    considerations: [
        (input: Hour, curve: Hours(22.0, 6.0)),
        (input: Need("rest"), curve: Linear(-1.0, 1.0)),
    ],
    satisfies: [("rest", 0.15)],
)
```

Options are scored by compensated considerations, tiered by `rank`, and drawn by weight.

### Relationships

`Ties` holds one directed edge per person with affinity, trust, respect and familiarity. Labels are recomputed each tick and a change writes `TieLabelChanged`. **A label is a latitude, not a line**: entering a category and leaving it are different tests (`Tuning::label_hysteresis` at the low end, `Tuning::friend_band` for a friendship, `Tuning::rival_band` for a rivalry), so a value resting on a bar does not announce a relationship change every time the model nudges it. Measured on `village-evening` at seed 7, a friendship now lasts a median **1.01 game-days** instead of **0.36** (`CHANGELOG.md`, 0.3.0), and a rivalry **2.64** instead of **0.29** — before the rival band a `Rival` was released when its affinity climbed **0.0001** above the bar, so the village made rivalries it could not keep ([the sweep](https://github.com/Ladvien/bevy_society/blob/main/docs/loop/runs/2026-10-27-02/eval.md)).

### Opinions

An accepted chat pulls both sides' opinions toward each other inside their confidence bounds, and a confident group majority pulls an agreeable person further, one tick at a time.

### Rumors and reputation

```rust,ignore
app.world_mut().write_message(PlantRumor {
    origin: watchman,
    subject: bram,
    patient: None,
    deed: "steal".to_owned(),
    truth: false,
});
```

Every witness who knows it can pass it on; a listener who already knows it stifles the teller.

### Factions and status

```ron
(factions: [(id: "millers"), (id: "weavers"), (id: "watch", ingroup_bias: 0.3)])
```

Two co-members start a tie with in-group affinity, and standing is the respect conferred by everyone who knows a person well enough.

### Triggers and scripting

```ron
(
    id: "watch_stops_theft",
    on: NormViolated("steal"),
    if: Some("ctx.member(actor, \"watch\")"),
    then: "fx.bark(actor, \"Stop, thief!\")",
)
```

### Dialogue

```rust,ignore
app.add_plugins(bevy_speech_bubbles::SpeechBubblesPlugin::<MyCamera>::default());
app.world_mut().write_message(TalkTo { npc: bram, player });
```

## Built on

| Crate | Used for | Source |
|---|---|---|
| Bevy | the engine | <https://github.com/bevyengine/bevy> |
| bevy_speech_bubbles | barks and conversations (`dialogue`) | <https://crates.io/crates/bevy_speech_bubbles> |
| bevy_stigmergy | per-cell presence and commotion fields (`fields`) | <https://crates.io/crates/bevy_stigmergy> |
| bevy_orca | reciprocal collision avoidance for walkers (`steering`) | <https://crates.io/crates/bevy_orca> |
| bevy_emerge_rng | the seeded ChaCha8 stream | <https://crates.io/crates/bevy_emerge_rng> |
| Rhai | trigger and effect scripts | <https://github.com/rhaiscript/rhai> |
| serde | authored definitions | <https://github.com/serde-rs/serde> |
| RON | the definitions' file format | <https://github.com/ron-rs/ron> |
| rand_chacha | names the ChaCha8 generator the stream comes from | <https://github.com/rust-random/rand> |
| bevy_fairy | the agentic debugger in the village example | <https://github.com/Ladvien/bevy_fairy> |

## The science

| Module | Key papers | Note |
|---|---|---|
| Personality and mood | Mehrabian 1996 · Gebhard 2005 (ALMA) | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/personality-and-mood.md) |
| Needs and utility | Mark & Dill (Game AI Pro 3 ch. 13) · Dill 2015 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/needs-and-utility.md) |
| Relationships | Heider 1946 · Zajonc 1968 · Dunbar 1992 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/relationships.md) |
| Opinions | Deffuant et al. 2000 · Friedkin & Johnsen 1990 · Asch 1956 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/opinion-dynamics.md) |
| Rumors and reputation | Daley & Kendall 1964 · Nowak & Sigmund 1998 · Ohtsuki & Iwasa 2004 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/rumor-and-reputation.md) |
| Factions and status | Tajfel et al. 1971 · Pettigrew & Tropp 2006 · Bonabeau et al. 1995 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/groups-and-status.md) |
| Memory | Park et al. 2023 · Anderson & Schooler 1991 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/memory.md) |
| Social simulation in games | Game AI Pro chapters · McCoy et al. 2014 | [note](https://github.com/Ladvien/bevy_society/blob/main/docs/research/social-simulation-games.md) |

The papers were gathered with [home-still](https://github.com/Ladvien/bevy_society/blob/main/docs/research/corpus/README.md), a local corpus pipeline; `docs/research/README.md` lists a DOI and a corpus id for every mechanism.

## Bevy compatibility

| bevy_society | Bevy |
|---|---|
| 0.1, 0.2, 0.3 | 0.20 |

## Documentation

[Getting started](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/getting-started.md) · [Concepts](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/concepts.md) · [Authoring](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/authoring.md) · [Scripting](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/scripting.md) · [Dialogue](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/dialogue.md) · [Decisions](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/decisions.md) · [Persistence](https://github.com/Ladvien/bevy_society/blob/main/docs/guide/persistence.md) · [Research](https://github.com/Ladvien/bevy_society/blob/main/docs/research/README.md) · [Demos](https://github.com/Ladvien/bevy_society/blob/main/docs/DEMOS.md)

## Contributing

Issues and pull requests are welcome. Agents working in this repository should read `AGENTS.md` first: it lists the invariants (determinism, the message rule, the single script-application point) that a change must not break.

## License

MIT OR Apache-2.0, at your option.

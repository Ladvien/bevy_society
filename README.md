# bevy_society

Give your Bevy world a living society: villagers who remember, gossip, take sides, fall out and make up — and who you can script.

[![CI](https://github.com/Ladvien/bevy_society/actions/workflows/ci.yml/badge.svg)](https://github.com/Ladvien/bevy_society/actions/workflows/ci.yml)

![A rumor spreads through a village](docs/media/village.gif)

One evening in a village of twenty-four people: Bram steals bread, the watch sees it, and by nightfall the rumor has reached twenty-three of twenty-five. Tie lines show friendships green and rivalries red; the red ring marks whoever knows the rumor. Every clip below is the same village, same seed, same run — only the overlay changes.

### One clip per feature

| | |
|---|---|
| ![Mood](docs/media/mood.gif) **Mood** — a halo on every villager. Green is pleased, amber neutral, red displeased; halo size is arousal. The colour scale is normalised to the ±0.4 the model actually produces, so a small mood shift is visible. | ![Opinion](docs/media/opinions.gif) **Opinions** — one bar per villager on the honesty topic, green above the midpoint and red below, length by strength. |
| ![Needs](docs/media/needs.gif) **Needs** — a full green halo is a fed villager, a small red one is hungry. | ![Ties](docs/media/ties.gif) **Relationships** — the tie graph alone: green friendship, red rivalry, brightness by familiarity. |
| ![Factions](docs/media/factions.gif) **Factions** — orange millers, green weavers, violet watch, no ties, so the group structure is the only thing on screen. | The clip above is the same run with every overlay on. |

## What you get

- **People with a mind** — Big Five personalities, moods on a pleasure/arousal/dominance axis, an OCC emotion set, needs that decay, and memories that fade. [personality-and-mood](docs/research/personality-and-mood.md), [memory](docs/research/memory.md)
- **Daily life** — utility-AI decisions over tagged places and other people, with travel, arrival, durations and need satisfaction. [needs-and-utility](docs/research/needs-and-utility.md)
- **Relationships** — directed ties carrying affinity, trust, respect and familiarity, with power-law forgetting, structural balance, a Dunbar-style cap and friend/rival labels. [relationships](docs/research/relationships.md)
- **Opinions** — bounded-confidence influence, stubborn anchors and conformity to a clear group majority. [opinion-dynamics](docs/research/opinion-dynamics.md)
- **Rumors and reputation** — gossip with the Daley–Kendall stifling rule, belief scaled by trust, witnesses who judge deeds and hold grudges. [rumor-and-reputation](docs/research/rumor-and-reputation.md)
- **Factions and status** — in-group affinity, standing as conferred respect, contests and intergroup contact. [groups-and-status](docs/research/groups-and-status.md)
- **Scripts** — Rhai triggers over a per-tick snapshot, with a `ctx` reader and an `fx` writer that cannot see its own writes. [scripting](docs/guide/scripting.md)
- **Dialogue** — barks and conversations through `bevy_speech_bubbles`. [dialogue](docs/guide/dialogue.md)

Every formula traces to a paper; every rate is a named field on `Tuning`; every default either comes from a paper or says it is a design default.

## Quick start

```toml
[dependencies]
bevy_society = { git = "https://github.com/Ladvien/bevy_society" }
```

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

Run it headlessly with `cargo run --example quickstart`, which is this code. On a real run it reports
`59 social exchanges, 30 accepted` after nine game-days.

The full village, with balloons and the debug server:

```
cargo run --example village -- /System/Library/Fonts/Supplemental/Arial.ttf
```

Font paths: macOS `/System/Library/Fonts/Supplemental/Arial.ttf`; Linux `/usr/share/fonts/liberation/LiberationSans-Regular.ttf`; Windows `C:\Windows\Fonts\arial.ttf`.

## A tour

### People

```rust,ignore
app.world_mut().spawn((
    Person,            // brings personality, mood, emotions, needs, ties, opinions, knowledge,
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

`Ties` holds one directed edge per person with affinity, trust, respect and familiarity. Labels are recomputed each tick and a change writes `TieLabelChanged`.

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
| bevy_speech_bubbles | barks and conversations (`dialogue`) | <https://github.com/Ladvien/bevy_speech_bubbles> |
| bevy_stigmergy | per-cell presence and commotion fields (`fields`) | <https://github.com/Ladvien/bevy_stigmergy> |
| bevy_orca | reciprocal collision avoidance for walkers (`steering`) | <https://github.com/Ladvien/bevy_orca> |
| bevy_emerge_rng | the seeded ChaCha8 stream | <https://github.com/Ladvien/bevy_emerge> |
| Rhai | trigger and effect scripts | <https://github.com/rhaiscript/rhai> |
| serde | authored definitions | <https://github.com/serde-rs/serde> |
| RON | the definitions' file format | <https://github.com/ron-rs/ron> |
| rand_chacha | names the ChaCha8 generator the stream comes from | <https://github.com/rust-random/rand> |
| bevy_fairy | the agentic debugger in the village example | <https://github.com/Ladvien/bevy_fairy> |

## The science

| Module | Key papers | Note |
|---|---|---|
| Personality and mood | Mehrabian 1996 · Gebhard 2005 (ALMA) | [note](docs/research/personality-and-mood.md) |
| Needs and utility | Mark & Dill (Game AI Pro 3 ch. 13) · Dill 2015 | [note](docs/research/needs-and-utility.md) |
| Relationships | Heider 1946 · Zajonc 1968 · Dunbar 1992 | [note](docs/research/relationships.md) |
| Opinions | Deffuant et al. 2000 · Friedkin & Johnsen 1990 · Asch 1956 | [note](docs/research/opinion-dynamics.md) |
| Rumors and reputation | Daley & Kendall 1964 · Nowak & Sigmund 1998 · Ohtsuki & Iwasa 2004 | [note](docs/research/rumor-and-reputation.md) |
| Factions and status | Tajfel et al. 1971 · Pettigrew & Tropp 2006 · Bonabeau et al. 1995 | [note](docs/research/groups-and-status.md) |
| Memory | Park et al. 2023 · Anderson & Schooler 1991 | [note](docs/research/memory.md) |
| Social simulation in games | Game AI Pro chapters · McCoy et al. 2014 | [note](docs/research/social-simulation-games.md) |

The papers were gathered with [home-still](docs/research/corpus/README.md), a local corpus pipeline; `docs/research/README.md` lists a DOI and a corpus id for every mechanism.

## Bevy compatibility

| bevy_society | Bevy |
|---|---|
| 0.1 | 0.20 |

## Documentation

[Getting started](docs/guide/getting-started.md) · [Concepts](docs/guide/concepts.md) · [Authoring](docs/guide/authoring.md) · [Scripting](docs/guide/scripting.md) · [Dialogue](docs/guide/dialogue.md) · [Research](docs/research/README.md) · [Demos](docs/DEMOS.md)

## Contributing

Issues and pull requests are welcome. Agents working in this repository should read `AGENTS.md` first: it lists the invariants (determinism, the message rule, the single script-application point) that a change must not break.

## License

MIT OR Apache-2.0, at your option.

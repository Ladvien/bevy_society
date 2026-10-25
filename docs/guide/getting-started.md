# Getting started

`bevy_society` gives a Bevy world people who remember, gossip, take sides and fall out. This page is
the shortest path from nothing to a village that visibly does something.

## Install

```toml
[dependencies]
bevy_society = { git = "https://github.com/Ladvien/bevy_society" }
```

The crate is `publish = false` while its ladvien dependencies are git dependencies, so it is consumed
by git. Pin a revision in a real project:

```toml
bevy_society = { git = "https://github.com/Ladvien/bevy_society", rev = "<commit>" }
```

## Add the plugins

```rust,no_run
use bevy::prelude::*;
use bevy_society::{SocietyDefs, SocietyPlugins, SocietySettings};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(SocietySettings::default())
        .insert_resource(SocietyDefs::starter().expect("the starter pack is valid"));
    // `DialoguePlugin` needs a speech-bubbles plugin over a camera (see
    // `docs/guide/dialogue.md`). Until the app has one, leave it out:
    let plugins = SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins).run();
}
```

`SocietySettings` and `SocietyDefs` must be inserted **before** `SocietyPlugins`. The group installs
every plugin the crate has; each one also works alone after `SocietyCorePlugin`, so a host that wants
only opinions and rumors can take those two.

`SocietyPlugins` includes `DialoguePlugin`, which **panics in `finish`** without
`bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera>`. Either disable it as above until the app has
a camera, or add the speech-bubbles plugin first; see `docs/guide/dialogue.md`.

## Spawn people and a place

```rust,ignore
let well = commands
    .spawn((
        Name::new("well"),
        Place { capacity: 6 },
        PlaceTags(vec!["gathering".to_owned()]),
        Transform::default(),
    ))
    .id();

for name in ["Bram", "Alys", "Corin"] {
    commands.spawn((
        Person,
        Autonomous,
        Name::new(name),
        Home(well),
        Transform::from_xyz(2.0, 0.0, 0.0),
    ));
}
```

`Person` brings every per-person component with it: personality, mood, emotions, needs, ties,
opinions, knowledge, memories, memberships and status. `Autonomous` is what makes a person choose its
own activities; a player-controlled person is a `Person` without the marker.

`Place` and `PlaceTags` are how activities find somewhere to go. A `Place`'s `capacity` limits how
many people may use it at once, so a crowded tavern pushes the next villager to the market.

## Read what happens

Everything the society does is a message. A game reads the ones it cares about:

```rust,ignore
fn watch(mut resolved: MessageReader<InteractionResolved>) {
    for exchange in resolved.read() {
        info!("{} -> {}: {}", exchange.actor, exchange.target, exchange.interaction);
    }
}
```

The messages worth knowing:

| Message | When |
|---|---|
| `InteractionResolved` | A social exchange finished, accepted or refused. |
| `ActivityStarted` / `ActivityEnded` | A person began or finished a daily-life activity. |
| `SocietyBark` | A line a person said or thought, ready to display. |
| `TieLabelChanged` | A relationship became a friendship, a rivalry, and so on. |
| `RumorLearned` | A person heard a rumor. |
| `NormViolated` | A witness judged someone's deed harshly. |
| `ScriptEvent` | A Rhai script emitted a named event. |
| `FireTrigger` | **You** write this to fire the `Custom` triggers with that name. |

### "What is this person doing?"

For a HUD or a log, read the `Doing` component and ask it:

```rust,ignore
fn hud(people: Query<(&Name, &Doing)>, defs: Res<SocietyDefs>) {
    for (name, doing) in &people {
        info!("{name}: {}", doing.label(&defs));
    }
}
```

`doing.label(&defs)` gives `"travelling:sleep"` while the person is still walking to bed, and `"sleep"` once it
has arrived and the activity has actually begun. That distinction matters: a person's needs and its activity's
duration start counting only on arrival, so a walker labelled `sleep` is not being paid anything.

`doing.activity` alone names the activity that was **chosen**, which is why the raw field over-reports whatever
people travel to — measured on the shipped village, sleep reads 36.8% of all person-ticks against the 33.3% one
8-hour sleep a day implies. Use `label` unless you specifically want the target.

## Give it a font, if you want balloons

With the `dialogue` feature (on by default) the crate turns barks into `bevy_speech_bubbles` balloons.
The host adds the speech-bubbles plugin itself, because it is generic over the camera marker:

```rust,ignore
app.add_plugins(bevy_speech_bubbles::SpeechBubblesPlugin::<MyCamera>::default());
```

Run the full example:

```
cargo run --example village -- /System/Library/Fonts/Supplemental/Arial.ttf
```

Font paths per host are in [`../DEMOS.md`](../DEMOS.md).

## Next

- [`concepts.md`](concepts.md): ticks, game-days, snapshots, determinism, and why the input messages
  are drained every fixed step.
- [`authoring.md`](authoring.md): every definition field, the starter pack annotated, and the error
  messages.
- [`scripting.md`](scripting.md): the Rhai `ctx` and `fx` API.
- [`dialogue.md`](dialogue.md): conversations with the player.

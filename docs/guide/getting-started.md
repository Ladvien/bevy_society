# Getting started

`bevy_society` gives a Bevy world people who remember, gossip, take sides and fall out. This page is
the shortest path from nothing to a village that visibly does something.

## Install

```toml
[dependencies]
bevy_society = "0.3"
```

It is on crates.io, and its ladvien dependencies (`bevy_speech_bubbles`, `bevy_stigmergy`,
`bevy_orca`, `bevy_emerge_rng`) are crates.io versions too, so a consumer needs no `[patch]`. Pin an
exact version in a real project (`bevy_society = "=0.3.0"`), since a pre-1.0 minor bump may break.
`bevy_fairy` is only a **dev**-dependency of this repository's examples and never reaches a consumer.

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

`Person` brings every per-person component with it: personality, mood, emotions, needs, signals,
ties, opinions, knowledge, memories, memberships and status. `Autonomous` is what makes a person
choose its own activities; a player-controlled person is a `Person` without the marker.

Two markers take a person further out of the simulation, and they are different things:

| marker | effect | keeps |
|---|---|---|
| `Dormant` | out of daily life *for now*: no crate-chosen activity, in nobody's perception, its running activity and any activity targeting it end, no exchange with it | ties, memories, opinions, knowledge, memberships |
| no `Person` | the person is gone: `on_person_removed` deletes every tie to and from them | nothing |

`Dormant` is the one you want for a night watch, a spell indoors, an expedition or a conversation: the
person is simply not living their daily life, and removing the marker lets them re-decide. And
`EndActivity { person }` cancels one running activity without any of that, for a place that closed or a
shift that ended.

`Place` and `PlaceTags` are how activities find somewhere to go. A `Place`'s `capacity` limits how
many people may use it at once, so a crowded tavern pushes the next villager to the market.

### Solid buildings: `Obstacle`

With the `steering` feature (on by default) a `Person` that also carries `SocialMover` walks to its destination under ORCA, so two walkers passing each other keep a gap. A building they must not walk through is an `Obstacle`, a round solid with a `radius` in metres on the plane, put on the same entity as the `Place` or on any other entity with a `Transform`:

```rust,ignore
commands.spawn((
    Name::new("tavern"),
    Place { capacity: 10 },
    PlaceTags(vec!["gathering".to_owned(), "food".to_owned()]),
    Obstacle { radius: 2.0 },
    Transform::from_xyz(6.5, 0.0, -5.0),
));
```

The obstacle's centre and radius are fed to ORCA as **hard wall half-planes**: a walker may slide along the edge or turn away, but is never accelerated into it. A final resolver also pushes any mover that has ended up inside one (a spawn placed there, a fast arrival) out to its surface along the line from the centre, so nobody is drawn half-buried in a building. The push needs no random draw, so a replay matches. Only entities with `SocialMover` are steered or pushed; a person you move yourself under `Locomotion::Host` is yours to keep outside. The default radius is `1.0`. The measurement behind it is in [`concepts.md`](concepts.md#walking-and-obstacles).

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
| `RumorLearned` | A person heard a rumor; `from` is the teller when they were told, `None` when they saw it themselves or the host planted it. |
| `ReportDeed` (write) | A host reports a deed its own rules saw: actor, patient, deed id and the witnesses **by name**, so reputation and rumor enter as seen by the people the game names. |
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

`Doing` says what a person is doing now. For *why they chose it* and what they passed over, insert a `DecisionTrace` and read it as in [`decisions.md`](decisions.md).

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
- [`decisions.md`](decisions.md): `DecisionTrace`, for "why did they do that?".
- [`persistence.md`](persistence.md): saving and restoring the whole society (feature `serde`).

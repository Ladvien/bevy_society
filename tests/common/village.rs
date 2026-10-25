//! A village shaped like the project's own, for any test that needs a society rather than a pair.
//!
//! Four places, an 8-wide column per faction, and one seed driving every spawn draw. `tests/determinism.rs`
//! and `tests/friendship_bound.rs` both build it, which is why it lives here: the friendship bound is a
//! statement about a village, and a second copy of the village would let the two drift.

#![allow(dead_code)]

use bevy::app::PluginGroup;
use bevy::prelude::*;
use bevy_emerge_rng::{DetRng, seeded};
use bevy_society::{
    Autonomous, DefsFile, Home, Person, Personality, Place, PlaceTags, SocietyDefs, SocietyPlugins,
    SocietySettings,
};

/// The village's four places: (name, position, tags, capacity).
///
/// **These must match `examples/evidence.rs`**, which builds the village the README's clip is rendered from.
/// They had drifted — `(8.0, -6.0)` here against `(6.5, -5.0)` there — so a test built on this layout was
/// asserting things about a village nobody watches. The farthest pair is **16.4 m** here, which at the
/// scenario's 64 s/day needs a `travel_timeout_hours` of at least 4.4; a 21.3 m layout needed 5.7 and made the
/// village unwalkable at its own declared settings.
pub const PLACES: &[(&str, Vec2, &[&str], u32)] = &[
    ("well", Vec2::new(0.0, 0.0), &["gathering"], 6),
    ("tavern", Vec2::new(6.5, -5.0), &["gathering", "food"], 10),
    ("market", Vec2::new(-6.5, -5.0), &["food", "work"], 8),
    ("workshop", Vec2::new(6.5, 5.0), &["work"], 6),
];

/// The village's factions, one per column of villagers.
pub const FACTIONS: &[&str] = &["millers", "weavers", "watch"];

/// The starter pack plus the village's own additions, compiled.
pub fn village_defs() -> SocietyDefs {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    let additions = DefsFile::from_ron(include_str!("../../examples/assets/village.society.ron"))
        .expect("the village additions parse");
    file.merge(additions)
        .expect("the village adds to the starter");
    SocietyDefs::compile(file).expect("the village compiles")
}

/// Builds a village of `people` villagers on the crate's default day, one column per faction.
///
/// The layout: villager `i` stands at column `i / 8` and row `i % 8`, spaced 14 m and 4 m apart, in
/// `FACTIONS[column % 3]`. With 24 people that is three full columns; with 60 it is seven columns and
/// four rows of the eighth, so the faction rotates exactly as it does in a real town.
pub fn village(defs: SocietyDefs, seed: u64, people: usize) -> App {
    let settings = SocietySettings {
        seed,
        ..SocietySettings::default()
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(0.25),
        ))
        .insert_resource(settings.clone())
        .insert_resource(defs);
    // No balloons in a headless village: the dialogue plugin needs SpeechBubblesPlugin, which needs a
    // camera. `headless` in `tests/common` disables it the same way.
    let plugins = SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();

    // Places first, so villagers can find them by tag.
    let mut homes = Vec::new();
    for &(name, pos, tags, capacity) in PLACES {
        let entity = app
            .world_mut()
            .spawn((
                Name::new(name.to_owned()),
                Transform::from_xyz(pos.x, 0.0, pos.y),
                Place { capacity },
                PlaceTags(tags.iter().map(|t| (*t).to_owned()).collect()),
            ))
            .id();
        homes.push(entity);
    }
    let home = homes[0];

    // One RNG for the spawn draws, so the whole village is a function of the seed.
    let mut rng = seeded(42);
    for i in 0..people {
        let column = i / 8;
        let row = i % 8;
        let faction = FACTIONS[column % FACTIONS.len()];
        let pos = Vec2::new(-14.0 + 14.0 * column as f32, -14.0 + 4.0 * row as f32);
        let honesty = if faction == "watch" {
            0.7
        } else {
            -0.2 + 0.8 * rng.unit() as f32
        };
        let name = format!("V{i:02}");
        spawn_villager(&mut app, &name, pos, home, faction, honesty, &mut rng);
    }
    app
}

/// Spawns one villager with a personality, a faction and two opinions.
pub fn spawn_villager(
    app: &mut App,
    name: &str,
    pos: Vec2,
    home: Entity,
    faction: &str,
    honesty: f32,
    rng: &mut rand_chacha::ChaCha8Rng,
) -> Entity {
    let defs = app.world().resource::<SocietyDefs>();
    let memberships = defs
        .memberships([faction])
        .expect("the village faction is defined");
    let opinions = defs
        .opinions([
            ("honesty", honesty),
            ("kindness", -0.2 + 0.8 * rng.unit() as f32),
        ])
        .expect("the village topics are defined");
    app.world_mut()
        .spawn((
            Person,
            bevy::ecs::name::Name::new(name.to_owned()),
            Autonomous,
            Home(home),
            Transform::from_xyz(pos.x, 0.0, pos.y),
            Personality::sample(rng),
            memberships,
            opinions,
        ))
        .id()
}

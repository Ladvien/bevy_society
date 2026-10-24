//! Determinism (plan D12): the same seed and the same spawn sequence give the same society, bit for
//! bit, and a different seed gives a different one.
//!
//! The hash walks `PersonIndex.sorted` and reads each person's ties, opinions, knowledge and needs,
//! so it covers every model the crate keeps per person. It hashes raw `f32` bits rather than a
//! rounded value: a determinism guarantee that survives rounding is not the one the crate makes.

mod common;

use bevy::app::PluginGroup;
use bevy::prelude::*;
use bevy_emerge_rng::{DetRng, seeded};
use bevy_society::{
    Autonomous, DefsFile, Home, Knowledge, Needs, Opinions, Person, PersonId, PersonIndex,
    Personality, Place, PlaceTags, SocietyClock, SocietyDefs, SocietyPlugins, SocietySettings,
    Ties,
};

/// The village's four places, as the F1 scene spawns them: (name, position, tags, capacity).
const PLACES: &[(&str, Vec2, &[&str], u32)] = &[
    ("well", Vec2::new(0.0, 0.0), &["gathering"], 6),
    ("tavern", Vec2::new(8.0, -6.0), &["gathering", "food"], 10),
    ("market", Vec2::new(-8.0, -6.0), &["food", "work"], 8),
    ("workshop", Vec2::new(8.0, 8.0), &["work"], 6),
];

/// The village's factions, one per column of the 24 villagers.
const FACTIONS: &[&str] = &["millers", "weavers", "watch"];

/// FNV-1a 64 over the whole society. Order is `PersonIndex.sorted`, so the hash is stable only when
/// the society is.
fn hash(app: &App) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn mix(state: &mut u64, value: u64) {
        for byte in value.to_le_bytes() {
            *state ^= u64::from(byte);
            *state = state.wrapping_mul(PRIME);
        }
    }

    let mut state = OFFSET;
    let index = app.world().resource::<PersonIndex>();
    let ids: &[(PersonId, Entity)] = &index.sorted;
    mix(&mut state, ids.len() as u64);
    for &(id, entity) in ids {
        mix(&mut state, u64::from(id.0));
        if let Some(ties) = app.world().get::<Ties>(entity) {
            mix(&mut state, ties.0.len() as u64);
            for tie in &ties.0 {
                let target = ids
                    .iter()
                    .find(|(_, e)| *e == tie.target)
                    .map_or(u64::MAX, |(id, _)| u64::from(id.0));
                mix(&mut state, target);
                mix(&mut state, u64::from(tie.affinity.to_bits()));
                mix(&mut state, u64::from(tie.trust.to_bits()));
                mix(&mut state, u64::from(tie.respect.to_bits()));
                mix(&mut state, u64::from(tie.familiarity_peak.to_bits()));
            }
        }
        if let Some(opinions) = app.world().get::<Opinions>(entity) {
            mix(&mut state, opinions.0.len() as u64);
            for opinion in &opinions.0 {
                mix(&mut state, u64::from(opinion.value.to_bits()));
            }
        }
        if let Some(knowledge) = app.world().get::<Knowledge>(entity) {
            mix(&mut state, knowledge.0.len() as u64);
            for known in &knowledge.0 {
                mix(&mut state, u64::from(known.rumor.0));
                mix(&mut state, u64::from(known.belief.to_bits()));
                mix(&mut state, known.spreader as u64);
            }
        }
        if let Some(needs) = app.world().get::<Needs>(entity) {
            mix(&mut state, needs.0.len() as u64);
            for level in &needs.0 {
                mix(&mut state, u64::from(level.to_bits()));
            }
        }
        // Personality is hashed too: it is the one model the seed reaches directly, through
        // `Personality::sample`, so a hash that skipped it could not tell two seeds apart until the
        // society had run long enough for a draw to change something else.
        if let Some(personality) = app.world().get::<Personality>(entity) {
            for trait_value in personality.as_array() {
                mix(&mut state, u64::from(trait_value.to_bits()));
            }
        }
        // Position and the current activity are hashed as well. A `Wander` decision draws its point
        // from `SocietyRng`, so position is the first place a seed difference shows up: a hash
        // limited to ties, opinions, knowledge and needs would call two different seeds identical
        // while their people were standing in different places doing different things.
        if let Some(transform) = app.world().get::<Transform>(entity) {
            mix(&mut state, u64::from(transform.translation.x.to_bits()));
            mix(&mut state, u64::from(transform.translation.z.to_bits()));
        }
        if let Some(doing) = app.world().get::<bevy_society::Doing>(entity) {
            mix(&mut state, u64::from(doing.activity.0));
            mix(&mut state, u64::from(doing.arrived_day.is_some()));
        }
    }
    state
}

/// Builds the village: four places, 24 villagers on a 6×4 grid spaced 2 m, eight per faction.
fn village(defs: SocietyDefs, seed: u64) -> App {
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
    // No balloons in a headless village: the dialogue plugin needs SpeechBubblesPlugin, which needs
    // a camera. `headless` in `tests/common` disables it the same way.
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
    for i in 0..24 {
        let column = i / 8;
        let row = i % 8;
        let faction = FACTIONS[column];
        let pos = Vec2::new(-14.0 + 14.0 * column as f32, -14.0 + 4.0 * row as f32);
        let honesty = if faction == "watch" {
            0.7
        } else {
            -0.2 + 0.8 * rng.unit() as f32
        };
        let name = format!("V{i:02}");
        let entity = spawn_villager(&mut app, &name, pos, home, faction, honesty, &mut rng);
        let _ = entity;
    }
    app
}

fn spawn_villager(
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
    let entity = app
        .world_mut()
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
        .id();
    #[cfg(feature = "steering")]
    {
        app.world_mut()
            .entity_mut(entity)
            .insert(bevy_society::SocialMover::default());
    }
    entity
}

fn village_defs() -> SocietyDefs {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    let additions = DefsFile::from_ron(include_str!("../examples/assets/village.society.ron"))
        .expect("the village additions parse");
    file.merge(additions)
        .expect("the village adds to the starter");
    SocietyDefs::compile(file).expect("the village compiles")
}

fn run(app: &mut App, ticks: usize) -> u64 {
    for _ in 0..ticks {
        app.update();
    }
    app.world().resource::<SocietyClock>().tick;
    hash(app)
}

#[test]
fn the_same_seed_gives_the_same_society() {
    let mut first = village(village_defs(), 7);
    let mut second = village(village_defs(), 7);
    let a = run(&mut first, 3000);
    let b = run(&mut second, 3000);
    assert_eq!(
        a, b,
        "the same seed and spawn sequence give the same society"
    );
    assert_ne!(a, 0, "the hash covers a non-empty society");
}

#[test]
fn a_different_seed_gives_a_different_society() {
    // The seed reaches the hashed state through the per-tick draws: personality sampling at spawn
    // and, once the behaviour plugin decides activities, one draw per decision. Until then two
    // villages of the same spawn sequence hash alike, so this asserts the whole path is seeded.
    let mut seven = village(village_defs(), 7);
    let mut eight = village(village_defs(), 8);
    let a = run(&mut seven, 3000);
    let b = run(&mut eight, 3000);
    assert_ne!(a, b, "the seed is part of the society");
}

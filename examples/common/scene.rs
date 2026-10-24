//! The village both `village` and `capture` build: one scene, so the GIF shows what the example runs.
//!
//! Forty metres of green ground, four tagged places, twenty-four villagers in three factions of
//! eight, and the player. Everything the two examples decide on their own is their own camera, input
//! and timeline; everything they must agree on lives here.

use bevy::ecs::name::Name;
use bevy::prelude::*;
use bevy_emerge_rng::{DetRng, seeded};
use bevy_society::{Autonomous, Home, Person, Personality, Place, PlaceTags, SocietyDefs};

/// Half the ground's side, in metres.
pub const GROUND_HALF: f32 = 20.0;

/// One village place: name, plane position, tags and how many may use it at once.
pub const PLACES: &[(&str, Vec2, &[&str], u32)] = &[
    ("well", Vec2::new(0.0, 0.0), &["gathering"], 6),
    ("tavern", Vec2::new(8.0, -6.0), &["gathering", "food"], 10),
    ("market", Vec2::new(-8.0, -6.0), &["food", "work"], 8),
    ("workshop", Vec2::new(8.0, 8.0), &["work"], 6),
];

/// The three factions, one per column of villagers.
pub const FACTIONS: &[(&str, Color)] = &[
    ("millers", Color::srgb(0.88, 0.69, 0.25)),
    ("weavers", Color::srgb(0.23, 0.65, 0.63)),
    ("watch", Color::srgb(0.42, 0.48, 0.66)),
];

/// The first villager's name, and the subject of the planted rumor. The rest follow from the list.
pub const VILLAGERS_PER_FACTION: usize = 8;

/// Fixed names, so the scene is readable in a frame and a debug command can name a villager.
pub const NAMES: &[&str] = &[
    "Bram", "Alys", "Corin", "Dena", "Edric", "Fenna", "Garet", "Hilda", // millers
    "Ivo", "Jena", "Kell", "Lira", "Milo", "Nera", "Oswin", "Perrin", // weavers
    "Quill", "Rhea", "Sable", "Tomas", "Ulric", "Vesna", "Wren", "Ysolde", // watch
];

/// The name of the rumor's subject: a miller, so the watch has someone to suspect and the millers
/// have someone to defend.
pub const RUMOR_SUBJECT: &str = "Bram";

/// The seed every spawn draw uses. Not the `SocietySettings::seed`: this one decides who a villager
/// is, and that must not change when a capture changes when the society runs.
pub const VILLAGE_SEED: u64 = 42;

/// What `build_village` needs from Bevy to draw the scene.
pub struct VillageMaterials {
    /// The colour of the ground.
    pub ground: Handle<StandardMaterial>,
    /// One material per place, in `PLACES` order.
    pub places: Vec<Handle<StandardMaterial>>,
    /// One material per faction, in `FACTIONS` order.
    pub factions: Vec<Handle<StandardMaterial>>,
    /// The player's material.
    pub player: Handle<StandardMaterial>,
}

/// The entities a scene build produced, for the examples to keep looking at.
///
/// A resource because a `Startup` system is the only place with both `Commands` and the mesh and
/// material assets: `Commands` borrows the world, so the two cannot be taken in the same expression.
#[derive(Resource, Debug, Clone)]
pub struct Village {
    /// The player person.
    pub player: Entity,
    /// The places, in `PLACES` order.
    pub places: Vec<Entity>,
    /// The villagers, in spawn order, so `NAMES[i]` is `villagers()[i]`.
    pub villagers: Vec<Entity>,
}

/// Builds the ground plane and returns its entity.
pub fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) -> Entity {
    commands
        .spawn((
            Name::new("ground"),
            Mesh3d(
                meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(GROUND_HALF * 2.0, GROUND_HALF * 2.0),
                ),
            ),
            MeshMaterial3d(material),
            Transform::default(),
        ))
        .id()
}

/// Spawns the four places as coloured cuboids with their tags, and returns them in `PLACES` order.
pub fn spawn_places(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &[Handle<StandardMaterial>],
) -> Vec<Entity> {
    let mut out = Vec::with_capacity(PLACES.len());
    for (i, &(name, pos, tags, capacity)) in PLACES.iter().enumerate() {
        let material = materials.get(i).cloned().unwrap_or_default();
        let height = 1.0 + 0.5 * capacity as f32;
        out.push(
            commands
                .spawn((
                    Name::new(name),
                    Place { capacity },
                    PlaceTags(tags.iter().map(|t| (*t).to_owned()).collect()),
                    Mesh3d(meshes.add(Cuboid::new(3.0, height, 3.0))),
                    MeshMaterial3d(material),
                    Transform::from_xyz(pos.x, height / 2.0, pos.y),
                ))
                .id(),
        );
    }
    out
}

/// Spawns the twenty-four villagers: eight per faction, in three columns.
///
/// Every draw comes from one seeded generator, so the village is a function of [`VILLAGE_SEED`] and
/// nothing else. The watch is honest and the others are not, which is what makes the theft rumor
/// believable to one faction and an insult to another.
pub fn spawn_villagers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &[Handle<StandardMaterial>],
    defs: &SocietyDefs,
    home: Entity,
) -> Vec<Entity> {
    let capsule = meshes.add(Capsule3d::new(0.3, 1.2));
    let mut rng = seeded(VILLAGE_SEED);
    let mut out = Vec::with_capacity(NAMES.len());
    for (i, name) in NAMES.iter().enumerate() {
        let column = i / VILLAGERS_PER_FACTION;
        let row = i % VILLAGERS_PER_FACTION;
        let (faction, _) = FACTIONS[column];
        let material = materials.get(column).cloned().unwrap_or_default();
        let pos = Vec2::new(-14.0 + 14.0 * column as f32, -14.0 + 4.0 * row as f32);
        let honesty = if faction == "watch" {
            0.7
        } else {
            -0.2 + 0.8 * rng.unit() as f32
        };
        let kindness = -0.2 + 0.8 * rng.unit() as f32;
        let memberships = defs.memberships([faction]).unwrap_or_default();
        let opinions = defs
            .opinions([("honesty", honesty), ("kindness", kindness)])
            .unwrap_or_default();
        let entity = commands
            .spawn((
                Name::new(*name),
                Person,
                Autonomous,
                Home(home),
                Transform::from_xyz(pos.x, 0.8, pos.y),
                Mesh3d(capsule.clone()),
                MeshMaterial3d(material),
                Personality::sample(&mut rng),
                memberships,
                opinions,
            ))
            .id();
        #[cfg(feature = "steering")]
        commands
            .entity(entity)
            .insert(bevy_society::SocialMover::default());
        out.push(entity);
    }
    out
}

/// Spawns the player: a white capsule at the south of the village, with no [`Autonomous`] marker so it
/// only does what the game tells it to.
pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) -> Entity {
    commands
        .spawn((
            Name::new("You"),
            Person,
            Transform::from_xyz(0.0, 0.8, 10.0),
            Mesh3d(meshes.add(Capsule3d::new(0.32, 1.3))),
            MeshMaterial3d(material),
        ))
        .id()
}

/// Builds the whole village. `materials` must already be built from the asset handles.
pub fn build_village(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &VillageMaterials,
    defs: &SocietyDefs,
) -> Village {
    spawn_ground(commands, meshes, materials.ground.clone());
    let places = spawn_places(commands, meshes, &materials.places);
    let home = places.first().copied().unwrap_or(Entity::PLACEHOLDER);
    let villagers = spawn_villagers(commands, meshes, &materials.factions, defs, home);
    let player = spawn_player(commands, meshes, materials.player.clone());
    Village {
        player,
        places,
        villagers,
    }
}

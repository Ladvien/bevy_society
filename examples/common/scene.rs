//! The village both `village` and `capture` build: one scene, so the GIF shows what the example runs.
//!
//! Forty metres of green ground, four tagged places, twenty-four villagers in three factions of
//! eight, and the player. Everything the two examples decide on their own is their own camera, input
//! and timeline; everything they must agree on lives here.

use bevy::ecs::name::Name;
use bevy::prelude::*;
use bevy_emerge_rng::{DetRng, seeded};
#[cfg(feature = "steering")]
use bevy_society::Obstacle;
use bevy_society::{Autonomous, Home, Needs, Person, Personality, Place, PlaceTags, SocietyDefs};

/// Half the ground's side, in metres.
///
/// **40, not 20.** The camera has to cover the village plus `Wander(6.0)` of drift on every side, which
/// is 27.6 x 19.6 m; a 40 m plane left its edge showing as a blue wedge in the top-left corner of the
/// tie clip. At 80 m the plane is never in shot at any framing the demos use, and the sky is not.
pub const GROUND_HALF: f32 = 40.0;

/// Height of a place's stall, in metres. Low, so the crowd around it is the subject of the frame.
pub const PLACE_HEIGHT: f32 = 1.2;

/// Half-width of a place's stall, in metres.
///
/// **0.6 m, not 1.5.** A stall 3 m across could not be used at all: `interaction_radius` is 1.5 m, so
/// a person had to stand within 1.5 m of the place's centre to be "arrived" — which is *inside* a 1.5 m
/// half-width block. The two constraints were contradictory, so villagers were drawn standing in a
/// building. At 1.2 m across, a person on the 1.1 m gather ring is outside the stall and within reach.
pub const PLACE_HALF: f32 = 0.6;

/// How far a walker is kept from a place's centre.
///
/// The stall's **corner** diagonal is `PLACE_HALF * sqrt(2) = 0.85 m`, and a body is 0.35 m, so 1.25 m
/// is the smallest radius that keeps a capsule clear of the box at every angle. The old 0.95 m used the
/// half-width rather than the corner, and a villager standing diagonally still overlapped it.
#[cfg(feature = "steering")]
pub const PLACE_REACH: f32 = 1.25;

/// Radius of the ring villagers stand on around their place, in metres: outside the stall, and inside
/// `interaction_radius` (1.5 m) so they start arrived and the duration is what limits an activity.
pub const GATHER_RADIUS: f32 = 1.3;

/// One village place: name, plane position, tags and how many may use it at once.
pub const PLACES: &[(&str, Vec2, &[&str], u32)] = &[
    ("well", Vec2::new(0.0, 0.0), &["gathering"], 6),
    ("tavern", Vec2::new(6.5, -5.0), &["gathering", "food"], 10),
    ("market", Vec2::new(-6.5, -5.0), &["food", "work"], 8),
    ("workshop", Vec2::new(6.5, 5.0), &["work"], 6),
];

/// The three factions, one per column of villagers.
///
/// Chosen for **separation**, not taste. The old teal `(0.23, 0.65, 0.63)` and blue `(0.42, 0.48, 0.66)`
/// differ by 0.19 in red and 0.17 in blue — close enough that under warm key light they read as one
/// colour, and a viewer could not tell the weavers from the watch at the GIF's 720 px. These are spread
/// across the hue circle (orange, green, violet) and stay distinct after a 256-colour palette.
pub const FACTIONS: &[(&str, Color)] = &[
    ("millers", Color::srgb(0.95, 0.62, 0.10)),
    ("weavers", Color::srgb(0.15, 0.72, 0.35)),
    ("watch", Color::srgb(0.45, 0.30, 0.90)),
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
        // A low stall, not a tower. These used to be `1.0 + 0.5 * capacity` tall — up to 6 m for the
        // tavern — and villagers stood *inside* them, so the README clips showed half-bodies emerging
        // from a block. A 1.2 m stall reads as the thing a crowd has gathered around instead.
        let height = PLACE_HEIGHT;
        out.push(
            commands
                .spawn((
                    Name::new(name),
                    Place { capacity },
                    PlaceTags(tags.iter().map(|t| (*t).to_owned()).collect()),
                    Mesh3d(meshes.add(Cuboid::new(PLACE_HALF * 2.0, height, PLACE_HALF * 2.0))),
                    MeshMaterial3d(material),
                    Transform::from_xyz(pos.x, height / 2.0, pos.y),
                    #[cfg(feature = "steering")]
                    Obstacle {
                        radius: PLACE_REACH,
                    },
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
    homes: &[Entity],
) -> Vec<Entity> {
    let capsule = meshes.add(Capsule3d::new(0.3, 1.2));
    let mut rng = seeded(VILLAGE_SEED);
    let mut out = Vec::with_capacity(NAMES.len());
    for (i, name) in NAMES.iter().enumerate() {
        let column = i / VILLAGERS_PER_FACTION;
        let row = i % VILLAGERS_PER_FACTION;
        let (faction, _) = FACTIONS[column];
        let material = materials.get(column).cloned().unwrap_or_default();
        // 2.5 m spacing. A walker covers `speed / 64` metres per tick, so a 14 m village put every
        // place 640 ticks away and nobody ever arrived; at this spacing a neighbour walk is ~110.
        // Villagers start within `interaction_radius` of a place: a walk is real-time
        // (`speed / 64` m per tick) while a duration is game-time, so a 3.5 m grid walk takes 160
        // ticks and a 0.3 game-hour chat lasts 3 — nobody would ever arrive before their activity
        // ended. Starting arrived makes the duration the thing that limits an activity.
        let anchor = PLACES[column % PLACES.len()].1;
        // **Outside** the stall, not on top of it. These used to stand 0.6 m from the place's centre
        // while the place was a 3 m cube, so every villager was rendered inside a building.
        let angle = row as f32 * std::f32::consts::TAU / VILLAGERS_PER_FACTION as f32;
        let pos = anchor + Vec2::new(GATHER_RADIUS * angle.cos(), GATHER_RADIUS * angle.sin());
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
                // Live at one place, work at another, so there is a commute to watch. All twenty-four
                // used to live at the place they started at, which meant a person never had anywhere
                // to go: measured, the clips showed the same clusters in every frame.
                Home(homes[column % homes.len()]),
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

/// Starts the demo's villagers **fed**.
///
/// The starter pack declares needs with no `initial`, so every person begins at 0.0 and the decision
/// system has to claw its way up from nothing. A clip recorded after 3 800 settle ticks therefore opened
/// with the whole village starving — the `needs` demo showed 24 identical red halos from its first frame
/// to its last, which demonstrates a constant, not a scale.
///
/// Filling them at spawn means the clip opens fed and the need decays *during* the recording, so a viewer
/// can watch hunger arrive. It changes only the demo's starting state, never the model.
pub fn feed_villagers(
    mut commands: Commands,
    villagers: Query<Entity, (With<Autonomous>, Without<Needs>)>,
) {
    for entity in &villagers {
        commands.entity(entity).insert(Needs(vec![0.85; 4]));
    }
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
    // Each faction's people live at **their own** place. All twenty-four used to share one `Home`
    // while starting at their faction's place, so every sleep was a cross-village walk that could not
    // be finished inside the travel timeout: measured, 25 interrupted sleeps against 4 completed.
    let villagers = spawn_villagers(commands, meshes, &materials.factions, defs, &places);
    let player = spawn_player(commands, meshes, materials.player.clone());
    Village {
        player,
        places,
        villagers,
    }
}

//! The village: twenty-four people living a day while you watch, and whom you can talk to.
//!
//! ```text
//! cargo run --example village -- /System/Library/Fonts/Supplemental/Arial.ttf
//! ```
//!
//! Keys: click a villager to talk to them, `R` to plant the rumor, `T` to toggle the tie lines.
//!
//! The debug server listens on `127.0.0.1:15702`. `society/inspect` returns one villager's ties,
//! mood, needs, opinions and knowledge as JSON; `society/rumor` plants a rumor. Both exist so an
//! agent can drive the scene without a mouse.

mod common;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_society::{
    Person, PersonIndex, PlantRumor, SocietyDefs, SocietyPlugins, SocietySettings, TalkTo,
};
use bevy_speech_bubbles::{BubbleAssets, SpeechBubblesPlugin};
use common::scene::{self, VillageMaterials};
use common::visuals::{self, RumorMarkers, ShowTies, SunClock, spawn_hud, update_hud};

/// The camera the speech balloons anchor to, and the only camera in the scene.
#[derive(Component)]
struct VillageCamera;

/// The player's entity, so input can address it.
#[derive(Resource)]
struct Player(Entity);

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(font_path) = args.next() else {
        eprintln!(
            "village: needs a font path.\n  \
             cargo run --example village -- /System/Library/Fonts/Supplemental/Arial.ttf"
        );
        std::process::exit(1);
    };
    let font_bytes = match std::fs::read(&font_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("village: cannot read {font_path}: {err}");
            std::process::exit(1);
        }
    };
    let defs = match society_defs() {
        Ok(defs) => defs,
        Err(err) => {
            eprintln!("village: the definitions do not compile: {err}");
            std::process::exit(1);
        }
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevy_society — the village".to_owned(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: "examples/assets".to_owned(),
                ..default()
            }),
    )
    .add_plugins(SpeechBubblesPlugin::<VillageCamera>::default())
    .insert_resource(SocietySettings::default())
    .insert_resource(defs)
    .add_plugins(SocietyPlugins)
    .insert_resource(ShowTies(true))
    .insert_resource(RumorMarkers(true))
    .init_resource::<visuals::PlantedRumor>()
    .insert_resource(SunClock::default())
    .insert_resource(ClearColor(Color::srgb(0.45, 0.55, 0.68)))
    .insert_resource(FontBytes(font_bytes))
    .add_systems(Startup, setup_fonts)
    .add_systems(Startup, setup_scene)
    .add_systems(Update, (update_hud, visuals::move_sun).chain())
    .add_systems(Update, visuals::draw_ties)
    .add_systems(
        Update,
        (visuals::track_planted_rumor, visuals::draw_rumor_rings).chain(),
    )
    .add_systems(Update, (click_to_talk, plant_rumor_key, toggle_ties));
    app.run();
}

/// The ground, the places, the villagers, the player, the sun and the camera.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    defs: Res<SocietyDefs>,
) {
    let built = VillageMaterials {
        ground: materials.add(Color::srgb(0.22, 0.35, 0.22)),
        places: vec![
            materials.add(Color::srgb(0.62, 0.58, 0.50)),
            materials.add(Color::srgb(0.52, 0.36, 0.24)),
            materials.add(Color::srgb(0.55, 0.50, 0.35)),
            materials.add(Color::srgb(0.40, 0.42, 0.45)),
        ],
        factions: vec![
            materials.add(Color::srgb(0.88, 0.69, 0.25)),
            materials.add(Color::srgb(0.23, 0.65, 0.63)),
            materials.add(Color::srgb(0.42, 0.48, 0.66)),
        ],
        player: materials.add(Color::srgb(0.92, 0.92, 0.92)),
    };
    let village = scene::build_village(&mut commands, &mut meshes, &built, &defs);
    commands.insert_resource(Player(village.player));
    visuals::spawn_sun(&mut commands);
    commands.spawn((
        VillageCamera,
        Camera3d::default(),
        Transform::from_xyz(18.0, 18.0, 22.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

/// The font bytes the host passed on the command line, moved into the app as a resource so a system
/// can read it.
#[derive(Resource)]
struct FontBytes(Vec<u8>);

/// Builds the balloon assets and the HUD from the one font the host passed in.
///
/// `BubbleAssets` keeps its own `FontArc`; Bevy's text needs the same bytes as a `Font`. One file,
/// two wrappers, so the two never drift.
fn setup_fonts(
    bytes: Res<FontBytes>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    match BubbleAssets::from_font_bytes(quad, bytes.0.clone()) {
        Ok(assets) => commands.insert_resource(assets),
        Err(err) => error!("village: the font is unusable: {err}"),
    }
    let font = fonts.add(Font::from_bytes(bytes.0.clone()));
    spawn_hud(&mut commands, font);

    // The three conversations the definition file names. Validated here, so a broken script is a loud
    // error at start rather than a conversation that never opens.
    let text = include_str!("assets/village.dialogue.ron");
    match ron::from_str::<bevy_speech_bubbles::DialogueScript>(text) {
        Ok(script) => {
            if let Err(err) = bevy_speech_bubbles::validate_script(&script) {
                error!("village: the dialogue script is invalid: {err}");
            }
            commands.insert_resource(script);
        }
        Err(err) => error!("village: the dialogue script does not parse: {err}"),
    }
}

/// Clicking a villager writes `TalkTo` with the player.
fn click_to_talk(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<VillageCamera>>,
    people: Query<(Entity, &GlobalTransform), With<Person>>,
    player: Option<Res<Player>>,
    mut talk: MessageWriter<TalkTo>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let (Some(player), Ok(window), Ok((camera, camera_transform))) =
        (player, windows.single(), camera.single())
    else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    // The nearest villager within a capsule's width of the ray, ignoring the player itself.
    let mut best: Option<(f32, Entity)> = None;
    for (entity, transform) in &people {
        if entity == player.0 {
            continue;
        }
        let offset = transform.translation() - ray.origin;
        let along = offset.dot(*ray.direction);
        if along <= 0.0 {
            continue;
        }
        let miss = (offset - *ray.direction * along).length();
        if miss < 0.6 && best.is_none_or(|(d, _)| along < d) {
            best = Some((along, entity));
        }
    }
    if let Some((_, npc)) = best {
        talk.write(TalkTo {
            npc,
            player: player.0,
        });
    }
}

/// `R` plants the rumor about Bram, told by the first weaver.
fn plant_rumor_key(
    keys: Res<ButtonInput<KeyCode>>,
    index: Res<PersonIndex>,
    books: Res<bevy_society::RumorBook>,
    mut planted: ResMut<visuals::PlantedRumor>,
    mut writer: MessageWriter<PlantRumor>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    // Villagers spawn in NAMES order: index 0 is Bram the miller, index 8 the first weaver.
    let (Some(&(_, subject)), Some(&(_, origin))) = (index.sorted.first(), index.sorted.get(8))
    else {
        return;
    };
    planted.request(&books, origin, subject);
    writer.write(PlantRumor {
        origin,
        subject,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
}

/// `T` toggles the tie lines.
fn toggle_ties(keys: Res<ButtonInput<KeyCode>>, mut show: ResMut<ShowTies>) {
    if keys.just_pressed(KeyCode::KeyT) {
        show.0 = !show.0;
    }
}

/// The starter pack plus the village's additions.
fn society_defs() -> Result<SocietyDefs, String> {
    let mut file = bevy_society::DefsFile::starter().map_err(|e| e.to_string())?;
    let additions = bevy_society::DefsFile::from_ron(include_str!("assets/village.society.ron"))
        .map_err(|e| e.to_string())?;
    file.merge(additions).map_err(|e| e.to_string())?;
    SocietyDefs::compile(file).map_err(|e| e.to_string())
}

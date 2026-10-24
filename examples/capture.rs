//! Records the village to PNGs for the hero GIF.
//!
//! Not interactive: it builds the same scene `village` builds, runs the timeline and exits, printing
//! one JSON line a script can assert on. The frame timeline and the acceptance thresholds are in
//! `docs/DEMOS.md`.
//!
//! ```text
//! cargo +1.98.1 run --release --example capture -- \
//!     --font /System/Library/Fonts/Supplemental/Arial.ttf --out frames/hero --frames 480
//! ```

mod common;

use bevy::app::PluginGroup;
use bevy::prelude::*;
use bevy_society::{
    Knowledge, PersonIndex, PlantRumor, SocietyDefs, SocietyPlugins, SocietySettings, TieLabel,
    Ties,
};
use common::recorder::Recorder;
use common::scene::{self, VillageMaterials};
use common::visuals::{
    RumorMarkers, ShowTies, SunClock, draw_rumor_rings, draw_ties, move_sun, spawn_hud, update_hud,
};

/// The recorded frame at which the rumor is planted.
const PLANT_FRAME: u32 = 30;
/// Unrecorded frames before recording, so ties have formed and the village is not a static field.
const SETTLE_STEPS: u32 = 600;
/// Frames drawn and discarded, so the first *recorded* frame is not the blank still a GIF shows.
const WARM_UP: u32 = 3;
/// Frame width and height of the recording.
const WIDTH: u32 = 720;
const HEIGHT: u32 = 405;

/// The font bytes, moved into the app so a `Startup` system can read them.
#[derive(Resource)]
struct FontBytes(Vec<u8>);

/// The HUD and the sun, from the one font the host passed in.
fn setup_fonts(bytes: Res<FontBytes>, mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(bytes.0.clone()));
    spawn_hud(&mut commands, font);
    common::visuals::spawn_sun(&mut commands);
}

/// The villages, its places and the player.
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
    commands.insert_resource(village);
}

fn main() {
    let args = Args::parse();
    let font_bytes = match std::fs::read(&args.font) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("capture: cannot read {}: {err}", args.font);
            std::process::exit(1);
        }
    };
    let defs = match society_defs() {
        Ok(defs) => defs,
        Err(err) => {
            eprintln!("capture: the definitions do not compile: {err}");
            std::process::exit(1);
        }
    };

    // The society goes in through `new_with`: `Recorder::new` finishes and cleans the app, and after
    // that Bevy will not accept another plugin.
    let Some(mut rec) = Recorder::new_with(WIDTH, HEIGHT, orbit(0.0), &args.out, move |app| {
        app.insert_resource(SocietySettings {
            seconds_per_day: 64.0,
            tick_secs: 0.1,
            start_hour: 15.0,
            ..SocietySettings::default()
        });
        app.insert_resource(defs);
        // The capture has no balloons: it draws its own HUD and rings, so the dialogue plugin is
        // disabled rather than adding a speech-bubbles plugin the frames would never show.
        let plugins = SocietyPlugins.build();
        #[cfg(feature = "dialogue")]
        let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
        app.add_plugins(plugins);
        app.insert_resource(ShowTies(true));
        app.insert_resource(RumorMarkers(true));
        app.insert_resource(SunClock::default());
        app.insert_resource(ClearColor(Color::srgb(0.45, 0.55, 0.68)));
        app.add_systems(Update, (update_hud, move_sun).chain());
        app.add_systems(Update, draw_ties);
        app.add_systems(Update, draw_rumor_rings);
        app.insert_resource(FontBytes(font_bytes.clone()));
        app.add_systems(Startup, (setup_fonts, setup_scene));
    }) else {
        std::process::exit(1);
    };

    // `Startup` has not run until the app is pumped once, so the village exists only after this.
    rec.step();
    let Some(village) = rec.world().get_resource::<scene::Village>().cloned() else {
        eprintln!("capture: the scene did not build");
        std::process::exit(1);
    };

    rec.warm_up(SETTLE_STEPS + WARM_UP);

    let mut barks = 0u32;
    for frame in 0..args.frames {
        if frame == PLANT_FRAME {
            plant(&mut rec, village.villagers.first().copied());
        }
        // The camera orbits across the clip rather than jumping, so the GIF reads as one shot.
        let fraction = frame as f32 / args.frames.max(1) as f32;
        if let Some(camera) = camera_entity(&mut rec) {
            let shot = orbit(fraction);
            if let Some(mut transform) = rec.world().get_mut::<Transform>(camera) {
                *transform = shot;
            }
        }
        rec.shoot();
        barks += drain_barks(&mut rec);
    }
    // `finish` consumes the recorder, so the numbers come first.
    let (reach, friends, rivals) = stats(rec.world(), &village);
    let written = rec.finish();
    println!(
        "{{\"frames\": {written}, \"rumor_reach\": {reach:.3}, \"friends\": {friends}, \"rivals\": {rivals}, \"barks\": {barks}}}"
    );
    if written < args.frames {
        eprintln!("capture: wrote {written} frames, expected {}", args.frames);
        std::process::exit(2);
    }
}

/// The recorder's camera: the one it spawned looking through the off-screen target.
fn camera_entity(rec: &mut Recorder) -> Option<Entity> {
    rec.world()
        .query_filtered::<Entity, With<Camera3d>>()
        .iter(rec.world())
        .next()
}

/// Plants the rumor about Bram, told by the first weaver.
fn plant(rec: &mut Recorder, subject: Option<Entity>) {
    let (Some(subject), Some(origin)) = (subject, weaver(rec)) else {
        return;
    };
    rec.world().write_message(PlantRumor {
        origin,
        subject,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
}

/// The first weaver: villagers spawn in `NAMES` order, eight per faction, so index 8 is the first
/// weaver. It is deliberately not the subject, and not a miller.
fn weaver(rec: &mut Recorder) -> Option<Entity> {
    rec.world()
        .resource::<PersonIndex>()
        .sorted
        .get(8)
        .map(|&(_, entity)| entity)
}

/// Counts the barks written since the last frame.
fn drain_barks(rec: &mut Recorder) -> u32 {
    rec.world()
        .resource_mut::<bevy::ecs::message::Messages<bevy_society::SocietyBark>>()
        .drain()
        .count() as u32
}

/// `(rumor reach, friendships, rivalries)` over the villagers, each friendship counted once.
fn stats(world: &mut World, village: &scene::Village) -> (f32, u32, u32) {
    let (mut friends, mut rivals) = (0u32, 0u32);
    for &entity in &village.villagers {
        let Some(ties) = world.get::<Ties>(entity) else {
            continue;
        };
        for tie in &ties.0 {
            match tie.label {
                TieLabel::Friend => friends += 1,
                TieLabel::Rival => rivals += 1,
                _ => {}
            }
        }
    }
    let living = world.resource::<PersonIndex>().sorted.len().max(1);
    let knowers = village
        .villagers
        .iter()
        .filter(|&&entity| {
            world
                .get::<Knowledge>(entity)
                .is_some_and(|known| known.get(bevy_society::RumorId(0)).is_some())
        })
        .count();
    (knowers as f32 / living as f32, friends / 2, rivals / 2)
}

/// A camera orbiting the origin: `fraction` 0..1 of the clip maps 20°..50° at radius 24, height 16.
fn orbit(fraction: f32) -> Transform {
    let degrees = 20.0 + 30.0 * fraction;
    let angle = degrees.to_radians();
    Transform::from_xyz(24.0 * angle.sin(), 16.0, 24.0 * angle.cos())
        .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y)
}

/// The command line: `--font <path>`, `--out <dir>`, `--frames <n>`.
struct Args {
    font: String,
    out: String,
    frames: u32,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            font: String::new(),
            out: "frames/hero".to_owned(),
            frames: 480,
        };
        let mut argv = std::env::args().skip(1);
        while let Some(flag) = argv.next() {
            match flag.as_str() {
                "--font" => args.font = argv.next().unwrap_or_default(),
                "--out" => args.out = argv.next().unwrap_or_default(),
                "--frames" => {
                    args.frames = argv
                        .next()
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(args.frames);
                }
                other => {
                    eprintln!("capture: unknown argument {other}");
                    std::process::exit(1);
                }
            }
        }
        if args.font.is_empty() {
            eprintln!("capture: --font <path.ttf> is required");
            std::process::exit(1);
        }
        args
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

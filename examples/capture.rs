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
use common::scenes;
use common::visuals::{
    RumorMarkers, ShowTies, SunClock, draw_rumor_rings, draw_ties, move_sun, spawn_hud, update_hud,
};

/// The recorded frame at which the rumor is planted.
const PLANT_FRAME: u32 = 30;
/// Unrecorded frames before recording, so ties have formed and the village is not a static field.
const SETTLE_STEPS: u32 = 3800;
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
    let scene = args.scene;
    let Some(mut rec) = Recorder::new_with(WIDTH, HEIGHT, orbit(0.0), &args.out, move |app| {
        app.insert_resource(SocietySettings {
            seconds_per_day: 8.0,
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
        // Each scene turns on the overlays its feature needs, and only those.
        // Ties only in the clips that are about ties. The factions clip is about who belongs with whom,
        // and lines over the crowd hid the colours that are its whole subject.
        app.insert_resource(ShowTies(matches!(scene, Scene::Village | Scene::Ties)));
        app.insert_resource(RumorMarkers(matches!(scene, Scene::Village)));
        app.insert_resource(scenes::OpinionBar(0));
        app.insert_resource(scenes::NeedRing(NEED_CLIP));
        app.insert_resource(common::visuals::SceneLabel(scene_label(scene).to_owned()));
        // The sun follows the clock in every scene: a clip lit differently from the rest reads as
        // a different simulation rather than a different view of the same one.
        app.insert_resource(SunClock(true));
        match scene {
            Scene::Mood => {
                app.add_systems(Update, scenes::draw_mood_discs);
            }
            Scene::Opinions => {
                app.add_systems(Update, scenes::draw_opinion_bars);
            }
            Scene::Needs => {
                app.add_systems(Update, scenes::draw_need_rings);
            }
            Scene::Village | Scene::Ties | Scene::Factions => {}
        }
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

    // The settle steps run the society too, and that is when most of the talking happens, so barks
    // are counted from the start. Counting only the recorded frames reported `barks 0` for a village
    // that had been gossiping for ten game-minutes.
    let mut barks = 0u32;
    for _ in 0..(SETTLE_STEPS + WARM_UP) {
        rec.step();
        barks += drain_barks(&mut rec);
    }

    for frame in 0..args.frames {
        if frame == PLANT_FRAME {
            plant(&mut rec, village.villagers.first().copied());
        }
        // The camera orbits across the clip rather than jumping, so the GIF reads as one shot.
        let fraction = frame as f32 / args.frames.max(1) as f32;
        if let Some(camera) = camera_entity(&mut rec) {
            // Each scene frames what it is about: the village wide, the tie graph close enough to
            // read individual lines, the overlay scenes close enough to read a 0.5 m ring.
            let shot = match scene {
                Scene::Village => orbit(fraction),
                Scene::Ties => scenes::ties_camera(fraction),
                _ => scenes::close_camera(fraction),
            };
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

/// The line the HUD shows under the counters, naming the feature the clip is about.
fn scene_label(scene: Scene) -> &'static str {
    match scene {
        Scene::Village => "a rumor spreads · ties form",
        Scene::Mood => "mood: green is pleased, red is displeased · ring size is arousal",
        Scene::Opinions => "opinion on honesty: green above the middle, red below",
        Scene::Needs => "food: a full green ring is fed, a small red one is hungry",
        Scene::Ties => "ties: green is friendship, red is rivalry",
        Scene::Factions => "faction colour · amber millers, teal weavers, blue watch",
    }
}

/// Which need the needs clip rings. `food` (index 1 in the starter pack), because hunger cycles
/// fastest and is the need a viewer can see a village fail at.
const NEED_CLIP: usize = 1;

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
    // The **newest** alive rumor, which is the one `plant` writes. Reading `RumorId(0)` reported a
    // different rumor: the first the run ever made, which for this clip is a deed from the settle
    // steps and not the theft the caption names.
    let newest = world
        .resource::<bevy_society::RumorBook>()
        .rumors
        .iter()
        .enumerate()
        .rev()
        .find(|(_, rumor)| rumor.alive)
        .map(|(index, _)| bevy_society::RumorId(index as u32));
    let knowers = newest.map_or(0, |id| {
        village
            .villagers
            .iter()
            .filter(|&&entity| {
                world
                    .get::<Knowledge>(entity)
                    .is_some_and(|known| known.get(id).is_some())
            })
            .count()
    });
    (knowers as f32 / living as f32, friends / 2, rivals / 2)
}

/// A camera orbiting the origin: `fraction` 0..1 of the clip maps 20°..50° at radius 24, height 16.
fn orbit(fraction: f32) -> Transform {
    let degrees = 20.0 + 30.0 * fraction;
    let angle = degrees.to_radians();
    // Framed on the crowd at roughly the radius `scenes::close_camera` uses. The ground plane is 40 m
    // across and the villagers stand in 1.2 m clusters round the well and the tavern, so the earlier
    // 19 m / 13.5 m orbit — chosen when the village was a 12 m square — put two thirds of every frame
    // on empty grass.
    let focus = Vec3::new(3.2, 0.9, -2.4);
    Transform::from_xyz(
        focus.x + 8.0 * angle.sin(),
        7.0,
        focus.z + 8.0 * angle.cos(),
    )
    .looking_at(focus, Vec3::Y)
}

/// Which feature the clip is about. One GIF per feature, so each scene turns on only the overlays
/// that feature needs and says what it is showing in the caption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    /// The village at dusk: a rumor spreads, ties form. The README hero.
    Village,
    /// Mood: PAD colour on every villager, so a mood swing is visible as a colour change.
    Mood,
    /// Opinions: a bar per villager on one topic, narrowing as they talk.
    Opinions,
    /// Relationships: tie lines only, close up.
    Ties,
    /// Factions: one colour per faction, with the intergroup boundary.
    Factions,
    /// Needs: a ring per villager whose fullness is the need level.
    Needs,
}

impl Scene {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "village" | "hero" => Self::Village,
            "mood" => Self::Mood,
            "opinions" => Self::Opinions,
            "ties" => Self::Ties,
            "factions" => Self::Factions,
            "needs" => Self::Needs,
            _ => return None,
        })
    }
}

/// The command line: `--font <path>`, `--out <dir>`, `--frames <n>`, `--scene <name>`.
struct Args {
    font: String,
    out: String,
    frames: u32,
    scene: Scene,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            font: String::new(),
            out: "frames/hero".to_owned(),
            frames: 480,
            scene: Scene::Village,
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
                "--scene" => {
                    let name = argv.next().unwrap_or_default();
                    match Scene::parse(&name) {
                        Some(scene) => args.scene = scene,
                        None => {
                            eprintln!("capture: unknown scene '{name}'");
                            std::process::exit(1);
                        }
                    }
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

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
    PlantedRumor, RumorMarkers, ShowTies, SunClock, draw_rumor_rings, draw_ties, move_sun,
    spawn_hud, track_planted_rumor, update_hud,
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
            materials.add(Color::srgb(0.95, 0.62, 0.10)),
            materials.add(Color::srgb(0.15, 0.72, 0.35)),
            materials.add(Color::srgb(0.45, 0.30, 0.90)),
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
            // **32 s a day, not 8.** A walk is real-time (`walk_speed` m per second) while an
            // activity's `duration_hours` is game-time, so `seconds_per_day` decides how far a person
            // can get before its activity ends. At 8 s/day a 3 m walk cost 6.4 game-hours against a
            // 0.3-hour chat, so nobody ever arrived anywhere and the clips showed twenty-four people
            // standing exactly where they spawned for the whole run. At 32 s/day the same walk is
            // 1.6 game-hours: still long enough to watch, short enough to finish inside a clip.
            seconds_per_day: 32.0,
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
        app.init_resource::<PlantedRumor>();
        // **`honesty` is index 1, not 0.** The starter pack declares `kindness` (0) then `honesty` (1);
        // this was 0, so the clip captioned "opinion of honesty" drew the village's kindness opinions.
        // The watch faction is the honest one (0.7 against -0.2), so with the right topic a third of the
        // bars go red — which is what makes the clip show a *contrast* rather than one colour.
        app.insert_resource(scenes::OpinionBar(1));
        app.insert_resource(scenes::NeedRing(NEED_CLIP));
        // Fixed daylight, deliberately. Following the clock sounded right and looked wrong: the
        // village's run starts at 15:00 and reaches 22:00 inside a clip, so the last third of every
        // GIF went dark blue and the feature it was recorded to show became unreadable. A README clip
        // has to be legible in every frame, and the clock is in the HUD anyway.
        app.insert_resource(SunClock(false));
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
        app.add_systems(Update, (track_planted_rumor, draw_rumor_rings).chain());
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

    // **Fed at the moment recording starts, not at spawn.** The settle steps run the society for a long
    // time, and the starter pack's needs have no `initial`, so by frame 0 the whole village had decayed
    // to zero — the `needs` clip showed 24 identical red halos from first frame to last, a constant
    // rather than a scale. Filling them here means the recording opens fed and a viewer watches hunger
    // arrive. It touches only the demo's starting state, never the model or the run that produced the
    // numbers printed above.
    feed(&mut rec, &village);
    spread_opinions(&mut rec, &village);

    for frame in 0..args.frames {
        if frame == PLANT_FRAME {
            plant(&mut rec, village.villagers.first().copied());
        }
        // The camera orbits across the clip rather than jumping, so the GIF reads as one shot.
        let fraction = frame as f32 / args.frames.max(1) as f32;
        if let Some(camera) = camera_entity(&mut rec) {
            // Each scene frames what it is about: the village wide, the tie graph close enough to
            // read individual lines, the overlay scenes close enough to read a 0.5 m ring.
            // Every scene is framed on **where the crowd actually is** this tick. A fixed frame lost a
            // faction the moment it wandered: measured, `factions` showed two colours while claiming
            // three. `Village` and `Ties` keep their own orbit so the clips differ in feel.
            let (centre, spread) = scenes::crowd(rec.world());
            // Same framing everywhere: each clip differs by what it *draws*, not by where the camera
            // stands. They used to differ by framing too, which meant `village` and `ties` became
            // near-identical once both were framed on the crowd.
            let shot = scenes::crowd_camera(centre, spread, fraction);
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

/// Fills every villager's needs to a comfortable level, so the recorded clip shows them fall.
fn feed(rec: &mut Recorder, village: &scene::Village) {
    for &entity in &village.villagers {
        if let Some(mut needs) = rec.world().get_mut::<bevy_society::Needs>(entity) {
            for level in needs.0.iter_mut() {
                *level = 0.85;
            }
        }
    }
}

/// Gives the villagers **opposing** opinions on honesty at the moment recording starts.
///
/// The settle steps run opinion dynamics to convergence: by frame 0 only about three of twenty-four
/// villagers still held a negative view, so the `opinions` clip — captioned "green high, red low" — showed
/// an almost uniformly green village and demonstrated a constant instead of a scale. Seeding a spread
/// means the recording opens divided and a viewer watches the influence model pull the two sides
/// together, which is what the feature actually is.
///
/// Only the demo's starting state changes; the model is untouched.
fn spread_opinions(rec: &mut Recorder, village: &scene::Village) {
    let topics = rec
        .world()
        .resource::<bevy_society::SocietyDefs>()
        .topic_id("honesty")
        .map(|id| usize::from(id.0));
    let Some(topic) = topics else {
        return;
    };
    for (i, &entity) in village.villagers.iter().enumerate() {
        if let Some(mut opinions) = rec.world().get_mut::<bevy_society::Opinions>(entity) {
            // Alternating strong positions, so the opening frame has both colours at full length.
            let value = if i % 3 == 2 { 0.6 } else { -0.6 };
            if let Some(slot) = opinions.0.get_mut(topic) {
                *slot = bevy_society::Opinion::at(value);
            }
        }
    }
}

/// Which need the needs clip rings.
///
/// **`food` is index 2, not 1.** The starter pack declares `social` (0), `rest` (1), `food` (2),
/// `fun` (3). This was `1`, so every "hunger" clip ringed `rest` while its caption and the HUD's own
/// label said food — a demo that showed the wrong feature and named it confidently.
const NEED_CLIP: usize = 2;

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
    rec.world()
        .resource_scope(|world, mut planted: Mut<PlantedRumor>| {
            planted.request(world.resource::<bevy_society::RumorBook>(), origin, subject);
        });
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
///
/// A busier teller was tried — the villager with the most accepted `gossip` exchanges over the settle
/// steps — and measured: the planted rumor reached **2** villagers in both cases, at 240 and at 480
/// frames, so the extra code bought nothing and was removed.
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
    // The planted rumor: the one `plant` writes and the rings and the HUD describe. Neither `RumorId(0)`
    // nor the newest alive rumor is it: the first is the settle steps' first deed, and the newest is
    // whichever deed the village last made a story of, which changes under the count.
    let newest = world.resource::<PlantedRumor>().id;
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
    let degrees = 16.0 + 12.0 * fraction;
    let angle = degrees.to_radians();
    // Framed on the crowd at roughly the radius `scenes::close_camera` uses. The ground plane is 40 m
    // across and the villagers stand in 1.2 m clusters round the well and the tavern, so the earlier
    // 19 m / 13.5 m orbit — chosen when the village was a 12 m square — put two thirds of every frame
    // on empty grass.
    // Whole village, same focus as the per-feature shots.
    let focus = Vec3::new(0.0, 0.4, -2.5);
    Transform::from_xyz(
        focus.x + 5.0 * angle.sin(),
        17.0,
        focus.z + 5.0 * angle.cos(),
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

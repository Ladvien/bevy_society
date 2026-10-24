//! What the village example and the capture draw on top of the simulation: tie lines, rumor rings, a
//! HUD and a sun that follows the clock.
//!
//! None of this is part of the crate. It reads the crate's components and draws them, which is the
//! point: a game keeps its own presentation and `bevy_society` keeps the society.

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Knowledge, Person, PersonIndex, RumorBook, RumorId, SocietyClock, SocietySettings,
    TieLabel, Ties,
};

/// Whether the tie lines are drawn. The village example toggles it with `T`; the capture leaves it on.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShowTies(pub bool);

const FRIEND_COLOR: Color = Color::srgb(0.25, 0.75, 0.35);
const RIVAL_COLOR: Color = Color::srgb(0.85, 0.25, 0.25);

/// Height above the ground the tie lines and rings are drawn at, clear of the ground plane.
pub const MARK_HEIGHT: f32 = 0.06;

/// A ring drawn under everyone who knows the planted rumor.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct RumorMarkers(pub bool);

/// Draws a line for every friendship and rivalry, in `PersonIndex` order so the picture is stable.
///
/// Only labels that mean something are drawn, and only for ties the pair is actually familiar: a line
/// between two strangers would be a lie about the relationship.
#[allow(clippy::too_many_arguments)]
pub fn draw_ties(
    show: Res<ShowTies>,
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    ties: Query<&Ties>,
    villagers: Query<(), (With<Autonomous>, With<Person>)>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    if !show.0 {
        return;
    }
    let now = clock.now_days as f32;
    let tuning = &settings.tuning;
    for &(_, entity) in &index.sorted {
        // **Villagers only.** `PersonIndex` includes the player, who has ties to nobody it can see but
        // still appeared in the hero clip as a line running off the bottom of the frame to an
        // off-screen capsule. A demo line should join two people a viewer can point at.
        if villagers.get(entity).is_err() {
            continue;
        }
        let (Ok(owned), Ok(mine)) = (ties.get(entity), transforms.get(entity)) else {
            continue;
        };
        let from = mine.translation();
        for tie in &owned.0 {
            let color = match tie.label {
                TieLabel::Friend => FRIEND_COLOR,
                TieLabel::Rival => RIVAL_COLOR,
                TieLabel::Stranger | TieLabel::Acquaintance => continue,
            };
            let familiarity = tie.familiarity(now, tuning);
            if familiarity < 0.1 {
                continue;
            }
            let Ok(theirs) = transforms.get(tie.target) else {
                continue;
            };
            let to = theirs.translation();
            // Chest height, not ground level: at `MARK_HEIGHT` the lines run under the capsules and
            // the ground plane's depth hides most of them. Measured on the ties clip, where a
            // ground-level graph showed two short segments instead of a network.
            gizmos.line(
                Vec3::new(from.x, from.y + 0.6, from.z),
                Vec3::new(to.x, to.y + 0.6, to.z),
                color.with_alpha(0.15 + 0.55 * familiarity),
            );
        }
    }
}

/// A ring under every villager who knows the planted rumor with belief at least 0.3.
pub fn draw_rumor_rings(
    markers: Res<RumorMarkers>,
    books: Res<RumorBook>,
    index: Res<PersonIndex>,
    knowledge: Query<&Knowledge>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    if !markers.0 {
        return;
    }
    // **The newest alive rumor, not `RumorId(0)`.** This read `books.rumors.first()` — the first rumor
    // the run ever produced, which is not the one the clip plants. The hero clip therefore drew rings for
    // a rumor nobody in frame had heard, and showed no rings at all while the HUD counted 10/24. The
    // correct version already existed in `scenes::draw_newest_rumor_rings` and was never called; that is
    // the one below now.
    let Some((index_of, _)) = books
        .rumors
        .iter()
        .enumerate()
        .rev()
        .find(|(_, rumor)| rumor.alive)
    else {
        return;
    };
    let id = RumorId(index_of as u32);
    for &(_, entity) in &index.sorted {
        let (Ok(known), Ok(transform)) = (knowledge.get(entity), transforms.get(entity)) else {
            continue;
        };
        let Some(entry) = known.get(id) else {
            continue;
        };
        if entry.belief < 0.3 {
            continue;
        }
        let pos = transform.translation();
        gizmos.circle(
            Isometry3d::new(
                Vec3::new(pos.x, pos.y + 1.05, pos.z),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            0.7,
            Color::srgb(0.95, 0.15, 0.55),
        );
    }
}

/// The HUD's text node.
#[derive(Component)]
pub struct Hud;

/// A per-clip line under the HUD, naming the feature the GIF is about.
///
/// One GIF per feature needs one sentence per GIF, or a reader has to infer what the colours mean.
#[derive(Resource, Debug, Clone, Default)]
pub struct SceneLabel(pub String);

/// Builds the HUD: a text node in the top-left, with the font the host passed in.
pub fn spawn_hud(commands: &mut Commands, font: Handle<Font>) -> Entity {
    commands
        .spawn((
            Hud,
            Text::new("Day 0 · 00:00"),
            TextFont {
                font: bevy::text::FontSource::Handle(font),
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::WHITE),
            bevy::text::LineHeight::Px(15.0),
            Node {
                position_type: PositionType::Absolute,
                // Bottom-left, not top-left. The top of the frame is where the village's far faction
                // stands, so a HUD plate there covered the very people the clip was recorded to show.
                //
                // **Two lines, not six.** The plate used to stack day, friends, rivals, the rumor line and
                // the scene label: a 430 px-wide slab covering a third of a 720 px frame, which forced
                // every camera back to keep the village out from behind it. The rumor line now sits beside
                // the counters rather than under them, and the scene label is the burned-in caption.
                bottom: px(6),
                left: px(8),
                max_width: px(360),
                // A dark plate behind the text. The village is bright grass and pale buildings, and
                // white text over it is unreadable at the GIF's 640 px; the plate costs nothing and
                // keeps every line legible whatever the camera is pointed at.
                padding: UiRect::all(px(7)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.72)),
        ))
        .id()
}

/// Updates the HUD: the day and time, the rumor's reach, and how many friendships and rivalries exist.
///
/// Each friendship is two directed ties, so the counts are halved; a rivalry likewise.
pub fn update_hud(
    clock: Res<SocietyClock>,
    books: Res<RumorBook>,
    index: Res<PersonIndex>,
    ties: Query<&Ties>,
    label: Option<Res<SceneLabel>>,
    villagers: Query<(), (With<Autonomous>, With<Person>)>,
    mut hud: Query<&mut Text, With<Hud>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let hour = clock.hour();
    let minutes = (hour.fract() * 60.0) as u32;
    let (mut friends, mut rivals) = (0u32, 0u32);
    for &(_, entity) in &index.sorted {
        let Ok(owned) = ties.get(entity) else {
            continue;
        };
        for tie in &owned.0 {
            match tie.label {
                TieLabel::Friend => friends += 1,
                TieLabel::Rival => rivals += 1,
                _ => {}
            }
        }
    }
    // The **newest** alive rumor, not `RumorId(0)`. The first rumor a run makes is not the one the
    // clip plants, and reading it reported a claim about the wrong one — the same mistake the
    // evidence harness made twice and the capture's ring overlay made once.
    let reach =
        books
            .rumors
            .iter()
            .rev()
            .find(|rumor| rumor.alive)
            .map_or(String::new(), |rumor| {
                // `PersonIndex::sorted` includes every `Person` — the player too — while `known_by`
                // counts the ones holding the rumor. Against the raw index the HUD read "17/25" in a
                // village of 24 villagers, which looks like an off-by-one bug to anyone reading it.
                let villagers = villagers.iter().count().max(1);
                format!(
                    "\n“Bram stole bread” — known by {}/{}",
                    rumor.known_by, villagers
                )
            });
    // The scene label is **not** in the HUD. Every GIF carries it as a burned-in caption, so printing it
    // twice cost a third line of the plate and covered the village it described.
    let _ = label;
    **text = format!(
        "Day {} · {:02}:{:02}   Friendships {} · Rivalries {}{reach}",
        clock.day(),
        hour as u32,
        minutes,
        friends / 2,
        rivals / 2,
    );
}

/// Whether the sun follows the clock. On in both examples; a host that lights its own scene turns it
/// off.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SunClock(pub bool);

impl Default for SunClock {
    fn default() -> Self {
        Self(true)
    }
}

/// The village sun. Moved and coloured by [`move_sun`] from `SocietyClock::hour`.
#[derive(Component)]
pub struct VillageSun;

/// Spawns the directional light the village is lit by, plus a little ambient so the faces turned
/// away from the sun stay readable instead of going black.
pub fn spawn_sun(commands: &mut Commands) -> Entity {
    // The crate-wide ambient: `AmbientLight` itself requires a `Camera` and overrides this per camera.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.80, 0.86, 1.0),
        brightness: 900.0,
        affects_lightmapped_meshes: true,
    });
    commands
        .spawn((
            VillageSun,
            DirectionalLight {
                illuminance: 16_000.0,
                shadow_maps_enabled: true,
                // Soft, small shadows. The default cascade covers a huge area, so a 1.4 m capsule's
                // shadow rasterised into almost nothing and then smeared — at the GIF's 720 px the
                // village read as a field of grey streaks. Narrowing the cascade concentrates the
                // shadow map's texels on the 20 m the camera actually sees.
                shadow_depth_bias: 0.02,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.6, 0.0)),
        ))
        .id()
}

/// Lerps the sun's colour and illuminance by the hour, so an evening capture actually darkens.
///
/// Noon is warm white, dusk orange, night a dim blue. The numbers are the example's, not the crate's.
pub fn move_sun(
    clock: Res<SocietyClock>,
    sun_clock: Res<SunClock>,
    mut suns: Query<(&mut DirectionalLight, &mut Transform), With<VillageSun>>,
) {
    let hour = clock.hour();
    // Daylight is 06:00 to 20:00; the sun's elevation follows a half sine across it.
    let day_fraction = ((hour - 6.0) / 14.0).clamp(0.0, 1.0);
    let elevation = (day_fraction * std::f32::consts::PI).sin();
    let (color, illuminance) = if !sun_clock.0 {
        (Color::WHITE, 16_000.0)
    } else if elevation > 0.15 {
        // Dusk tints the light orange as the elevation falls.
        let warmth = (1.0 - elevation).clamp(0.0, 1.0);
        (
            Color::srgb(1.0, 0.95 - 0.25 * warmth, 0.85 - 0.45 * warmth),
            6_000.0 + 22_000.0 * elevation,
        )
    } else {
        (Color::srgb(0.35, 0.42, 0.65), 900.0)
    };
    for (mut light, mut transform) in &mut suns {
        light.color = color;
        light.illuminance = illuminance;
        // **High enough that shadows do not swallow the frame.** The old `-0.25 - 1.1 * elevation`
        // put the sun near the horizon, and a 1.4 m capsule threw a shadow several metres long — at 240
        // px of a 405 px frame the clip was mostly shadow. A steep key light keeps each villager's shadow
        // tight under it and every body lit.
        let pitch = -1.35 - 0.20 * elevation;
        *transform = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, pitch, 0.6, 0.0));
    }
}

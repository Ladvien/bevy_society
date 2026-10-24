//! What the village example and the capture draw on top of the simulation: tie lines, rumor rings, a
//! HUD and a sun that follows the clock.
//!
//! None of this is part of the crate. It reads the crate's components and draws them, which is the
//! point: a game keeps its own presentation and `bevy_society` keeps the society.

use bevy::prelude::*;
use bevy_society::{
    Knowledge, PersonIndex, RumorBook, RumorId, SocietyClock, SocietySettings, TieLabel, Ties,
};

/// Whether the tie lines are drawn. The village example toggles it with `T`; the capture leaves it on.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShowTies(pub bool);

const FRIEND_COLOR: Color = Color::srgb(0.25, 0.75, 0.35);
const RIVAL_COLOR: Color = Color::srgb(0.85, 0.25, 0.25);

/// Height above the ground the tie lines and rings are drawn at, clear of the ground plane.
const MARK_HEIGHT: f32 = 0.06;

/// A ring drawn under everyone who knows the planted rumor.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct RumorMarkers(pub bool);

/// Draws a line for every friendship and rivalry, in `PersonIndex` order so the picture is stable.
///
/// Only labels that mean something are drawn, and only for ties the pair is actually familiar: a line
/// between two strangers would be a lie about the relationship.
pub fn draw_ties(
    show: Res<ShowTies>,
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    ties: Query<&Ties>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    if !show.0 {
        return;
    }
    let now = clock.now_days as f32;
    let tuning = &settings.tuning;
    for &(_, entity) in &index.sorted {
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
            gizmos.line(
                Vec3::new(from.x, MARK_HEIGHT, from.z),
                Vec3::new(to.x, MARK_HEIGHT, to.z),
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
    let Some(rumor) = books.rumors.first() else {
        return;
    };
    if !rumor.alive {
        return;
    }
    let id = RumorId(0);
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
                Vec3::new(pos.x, MARK_HEIGHT, pos.z),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            0.55,
            Color::srgb(0.95, 0.2, 0.2),
        );
    }
}

/// The HUD's text node.
#[derive(Component)]
pub struct Hud;

/// Builds the HUD: a text node in the top-left, with the font the host passed in.
pub fn spawn_hud(commands: &mut Commands, font: Handle<Font>) -> Entity {
    commands
        .spawn((
            Hud,
            Text::new("Day 0 · 00:00"),
            TextFont {
                font: bevy::text::FontSource::Handle(font),
                font_size: bevy::text::FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: px(10),
                left: px(12),
                ..default()
            },
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
    let reach = books.rumors.first().map_or(String::new(), |rumor| {
        format!(
            "\n“Bram stole bread” — known by {}/{}",
            rumor.known_by,
            index.sorted.len().max(1)
        )
    });
    **text = format!(
        "Day {} · {:02}:{:02}\nFriendships {} · Rivalries {}{reach}",
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
        // The sun swings from east to west; below the horizon it keeps a low angle so the scene is not
        // pitch black.
        let pitch = -0.25 - 1.1 * elevation;
        *transform = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, pitch, 0.6, 0.0));
    }
}

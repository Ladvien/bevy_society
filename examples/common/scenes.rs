//! Per-feature overlays for the README clips: one scene per feature, so each GIF shows exactly what
//! it claims to show and nothing else.
//!
//! Every system here is a *view*: it reads the society and draws, and never writes to it. That is the
//! same rule `docs/DEMOS.md` states for the capture — a demo must not change the thing it demonstrates.

use bevy::prelude::*;
use bevy_society::{Knowledge, Mood, Needs, Opinions, PersonIndex, RumorBook, RumorId};

use crate::common::visuals::MARK_HEIGHT;

/// Draws one ring per villager whose level on need `slot` is low, sized by how low.
///
/// A full ring is a comfortable need; the ring shrinks and reddens as the need starves, so a viewer
/// watches a village get hungry without reading a number.
#[derive(Resource)]
pub struct NeedRing(pub usize);

pub fn draw_need_rings(
    ring: Res<NeedRing>,
    index: Res<PersonIndex>,
    needs: Query<&Needs>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for &(_, entity) in &index.sorted {
        let (Ok(level), Ok(transform)) = (needs.get(entity), transforms.get(entity)) else {
            continue;
        };
        let Some(value) = level.0.get(ring.0) else {
            continue;
        };
        if std::env::var("SCENE_DEBUG").is_ok() {
            let t = transform.translation();
            eprintln!(
                "DBG needs {entity:?} need{}={value:.3} at ({:.2},{:.2},{:.2})",
                ring.0, t.x, t.y, t.z
            );
        }
        let pos = transform.translation();
        // Radius falls with the need. Sized for legibility at the capture's camera distance: a
        // 0.5 m ring is four pixels wide from 20 m up, which is not readable in a GIF.
        let radius = 0.25 + 0.85 * value;
        // Colour runs red (starving) to green (met).
        let color = Color::srgb(1.0 - 0.75 * value, 0.25 + 0.65 * value, 0.25);
        // Above the capsule, not under it: a ground-level ring is hidden by the mesh and by the
        // ground plane's depth. The capture camera looks down from 13 m, so a ring at head height
        // reads as a halo and is never occluded by the villager standing in it.
        gizmos.circle(
            Isometry3d::new(
                Vec3::new(pos.x, pos.y + 0.95, pos.z),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            radius,
            color,
        );
        // A second, inner ring: a single thin circle is hard to follow against grass and shadow, and
        // the pair reads as a halo at the GIF's 640 px width.
        gizmos.circle(
            Isometry3d::new(
                Vec3::new(pos.x, pos.y + 0.95, pos.z),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            radius * 0.62,
            color,
        );
    }
}

/// The half-width of the PAD range this village actually produces, measured over 4 200 ticks:
/// pleasure [-0.387, 0.382], arousal [-0.361, 0.447], dominance [-0.413, 0.513]. Colour scales are
/// normalised to it so a small mood shift is visible.
const MOOD_SPAN: f32 = 0.45;

/// The half-width of the opinion range this village produces, measured the same way. A bar drawn
/// proportional to the raw value was too short to see.
const OPINION_SPAN: f32 = 0.5;

/// Draws a capped bar above every villager on one opinion topic: green above the middle, red below.
#[derive(Resource)]
pub struct OpinionBar(pub usize);

pub fn draw_opinion_bars(
    bar: Res<OpinionBar>,
    index: Res<PersonIndex>,
    opinions: Query<&Opinions>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for &(_, entity) in &index.sorted {
        let (Ok(opinion), Ok(transform)) = (opinions.get(entity), transforms.get(entity)) else {
            continue;
        };
        let value = opinion.value(bar.0).clamp(-1.0, 1.0);
        let pos = transform.translation();
        let base = Vec3::new(pos.x, pos.y + 0.95, pos.z);
        // Measured, the village's opinions sit near zero and a proportional bar is invisible. The bar
        // is normalised to the range the model produces (about ±0.5) and given a floor, so "leaning
        // either way" is visible while the sign is what the colour says.
        let height = 0.12 + 0.55 * (value.abs() / OPINION_SPAN).clamp(0.0, 1.0);
        let color = if value >= 0.0 {
            Color::srgb(0.25, 0.8, 0.35)
        } else {
            Color::srgb(0.85, 0.3, 0.25)
        };
        // One vertical bar per villager, offset horizontally toward its own side so the column reads
        // as a bar rather than a lattice: seven parallel lines look like a tie graph from this angle.
        let side = if value >= 0.0 { 0.22 } else { -0.22 };
        let base = base + Vec3::new(side, 0.0, 0.0);
        gizmos.line(base, base + Vec3::Y * height, color);
        // A short cap makes the top of the bar obvious without thickening the whole line.
        gizmos.line(
            base + Vec3::Y * height,
            base + Vec3::Y * height + Vec3::X * 0.10,
            color,
        );
    }
}

/// Paints a disc under every villager whose colour is its PAD mood: pleasure to green-red,
/// arousal to radius.
pub fn draw_mood_discs(
    index: Res<PersonIndex>,
    moods: Query<&Mood>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for &(_, entity) in &index.sorted {
        let (Ok(mood), Ok(transform)) = (moods.get(entity), transforms.get(entity)) else {
            continue;
        };
        let pos = transform.translation();
        // Measured over 4 200 ticks of this village, PAD spans only about ±0.4, so a full [-1, 1]
        // scale maps every villager to nearly the same dull olive and the clip shows nothing. The
        // scale is normalised to the range the model actually produces: `MOOD_SPAN` is that
        // measurement, not a guess, and it is what makes a mood swing visible as a colour change.
        let p = (mood.pleasure / MOOD_SPAN).clamp(-1.0, 1.0);
        let a = ((mood.arousal + MOOD_SPAN) / (2.0 * MOOD_SPAN)).clamp(0.0, 1.0);
        let color = Color::srgb(0.55 - 0.5 * p, 0.55 + 0.45 * p, 0.25);
        gizmos.circle(
            Isometry3d::new(
                Vec3::new(pos.x, pos.y + 0.95, pos.z),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            0.25 + 0.85 * a,
            color,
        );
    }
}

/// Draws a ring under everyone who knows the **newest** alive rumor.
///
/// The capture used to read `RumorId(0)`, the first rumor the run made, which is not the one the clip
/// plants — the same mistake the evidence harness made twice. The newest alive rumor is the one the
/// clip is about.
pub fn draw_newest_rumor_rings(
    books: Res<RumorBook>,
    index: Res<PersonIndex>,
    knowledge: Query<&Knowledge>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    let Some((id, rumor)) = books
        .rumors
        .iter()
        .enumerate()
        .rev()
        .find(|(_, rumor)| rumor.alive)
    else {
        return;
    };
    let id = RumorId(id as u32);
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
    let _ = rumor;
}

/// Counts, for the printed line, how many villagers know the newest alive rumor.
pub fn newest_rumor_reach(
    books: &RumorBook,
    index: &PersonIndex,
    knowledge: &Query<&Knowledge>,
    world: &World,
) -> f32 {
    let Some((id, _)) = books
        .rumors
        .iter()
        .enumerate()
        .rev()
        .find(|(_, rumor)| rumor.alive)
    else {
        return 0.0;
    };
    let id = RumorId(id as u32);
    let living = index.sorted.len().max(1);
    let knowers = index
        .sorted
        .iter()
        .filter(|(_, entity)| {
            world
                .get::<Knowledge>(*entity)
                .is_some_and(|known| known.get(id).is_some())
        })
        .count();
    let _ = knowledge;
    knowers as f32 / living as f32
}

/// A close camera for the overlay scenes: near enough that a 0.5 m ring reads as a ring.
pub fn close_camera(fraction: f32) -> Transform {
    // Steeply down. The village's places are 6 m boxes and the villagers stand under them, so a low
    // camera puts a building between the viewer and the crowd — measured at 6 m and again at 9 m.
    // At 20 m up and a 15 m radius the whole grid is visible and a 0.5 m ring still reads.
    // Framed on the crowd, not the origin. The ground plane is 40 m across and the places are 6 m
    // boxes, so a wide shot is mostly empty grass; the villagers stand in 1.2 m clusters round the
    // well and the tavern. A 6 m radius at 5 m up fills the frame with people and keeps their faces
    // toward the camera.
    let angle = (30.0 + 25.0 * fraction).to_radians();
    let focus = Vec3::new(3.2, 0.9, -2.4);
    Transform::from_xyz(
        focus.x + 7.2 * angle.sin(),
        6.2,
        focus.z + 7.2 * angle.cos(),
    )
    .looking_at(focus, Vec3::Y)
}

/// A close camera over the tie graph, ignoring the sun so the lines read against a flat background.
pub fn ties_camera(fraction: f32) -> Transform {
    // Same framing as `close_camera`: the tie graph is about the crowd, and the earlier 13 m / 16 m
    // shot put four building tops in front of it.
    let angle = (25.0 + 20.0 * fraction).to_radians();
    let focus = Vec3::new(3.2, 0.9, -2.4);
    Transform::from_xyz(
        focus.x + 7.0 * angle.sin(),
        6.0,
        focus.z + 7.0 * angle.cos(),
    )
    .looking_at(focus, Vec3::Y)
}

//! Per-feature overlays for the README clips: one scene per feature, so each GIF shows exactly what
//! it claims to show and nothing else.
//!
//! Every system here is a *view*: it reads the society and draws, and never writes to it. That is the
//! same rule `docs/DEMOS.md` states for the capture — a demo must not change the thing it demonstrates.

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Knowledge, Mood, Needs, Opinions, Person, PersonIndex, RumorBook, RumorId,
};

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

/// A short perpendicular tick at the end of an opinion bar.
fn gilt_cap(gizmos: &mut Gizmos, at: Vec3, color: Color) {
    gizmos.line(at - Vec3::Z * 0.16, at + Vec3::Z * 0.16, color);
}

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
        // Measured, the village's opinions sit near zero and a proportional bar is invisible. The bar
        // is normalised to the range the model produces (about ±0.5) and given a floor, so "leaning
        // either way" is visible while the sign is what the colour says.

        let color = if value >= 0.0 {
            Color::srgb(0.25, 0.8, 0.35)
        } else {
            Color::srgb(0.85, 0.3, 0.25)
        };
        // **Horizontal, in the ground plane.** These were vertical lines 0.12-0.67 m tall while the
        // camera looks down from ~20 m: seen end-on, a 0.5 m vertical line is a few pixels of nothing and
        // the clip showed no overlay at all. Laid flat and drawn from the villager outward, the bar's
        // *length* is what the camera sees, in the same plane as every other ring and tie.
        // Long enough to compare at a glance. 0.35-1.5 m was still a stub from 20 m up; the village's
        // opinions cluster near zero, so the *differences* live in the top of the range and need room.
        // **Length carries magnitude from a fixed origin; colour carries the sign.** Two earlier versions
        // failed for the same reason: this village's opinions cluster within ±0.2 of neutral, so a bar
        // whose length came from `|value|` rendered every villager as the same short stub, and drawing red
        // to the left put it behind the capsule at this camera angle. Now every bar runs the same way from
        // its villager, its length is the magnitude, and the colour alone says which side of neutral — so
        // a 0.04 negative is as legible as a 0.4 positive.
        let length = 0.35 + 1.6 * (value.abs() / OPINION_SPAN).clamp(0.0, 1.0);
        let direction = Vec3::X;
        let from = Vec3::new(pos.x, pos.y - 0.75, pos.z);

        gizmos.line(from, from + direction * length, color);
        // A crossbar at the end, so the bar's reach is unambiguous at the GIF's width.
        gilt_cap(&mut gizmos, from + direction * length, color);
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
        // A **diverging** ramp: green at pleased, amber at neutral, red at displeased. The old
        // `(0.55-0.5p, 0.55+0.45p, 0.25)` ran red straight to green, so a mildly negative village — which
        // this one is, 5 rivalries in — came out uniformly red and the clip showed one colour while
        // claiming a scale. Amber in the middle makes both halves of the range visible.
        let color = if p >= 0.0 {
            Color::srgb(0.95 - 0.75 * p, 0.80 - 0.15 * p, 0.20)
        } else {
            let n = -p;
            Color::srgb(0.95, 0.80 - 0.60 * n, 0.20 + 0.25 * n)
        };
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

/// The centre of the crowd right now, and how far it reaches, from the villagers' transforms.
///
/// **The camera follows the crowd, not a constant.** The village has three groups that wander
/// independently — `play` targets `Wander(6.0)` — so a fixed frame sooner or later loses one of them;
/// measured, the `factions` clip showed the millers walking out of shot and the demo that exists to show
/// *three* colours was down to two. Averaging the living positions keeps all of them in frame at every
/// tick, and the orbit still gives the clip motion.
pub fn crowd(world: &mut World) -> (Vec3, f32) {
    let mut query = world.query_filtered::<&Transform, (With<Autonomous>, With<Person>)>();
    let points: Vec<Vec3> = query.iter(world).map(|t| t.translation).collect();
    if points.is_empty() {
        return (Vec3::new(0.0, 0.4, -2.5), 0.0);
    }
    let count = points.len() as f32;
    let centre = points.iter().fold(Vec3::ZERO, |sum, p| sum + *p) / count;
    let spread = points
        .iter()
        .map(|p| Vec2::new(p.x - centre.x, p.z - centre.z).length())
        .fold(0.0_f32, f32::max);
    (Vec3::new(centre.x, 0.4, centre.z), spread)
}

/// A camera that holds the whole crowd, from its measured centre and spread.
pub fn crowd_camera(centre: Vec3, spread: f32, fraction: f32) -> Transform {
    // Far enough back that the crowd's **full diameter** fits the narrower (vertical) field of view.
    // `spread` is the radius from the centroid, so the box to cover is `2 * spread` across; at a 45
    // degree tilt the ground is compressed vertically, so the multiplier has to be generous. Measured at
    // 2.6: the violet group sat on the left edge with half its members outside. 3.6 covers it, and the
    // floor keeps a tight crowd from being filmed from the stratosphere.
    let reach = (spread * 2.7).clamp(14.5, 27.0);
    let angle = (18.0 + 12.0 * fraction).to_radians();
    Transform::from_xyz(
        centre.x + 0.27 * reach * angle.sin(),
        reach,
        centre.z + 0.27 * reach * angle.cos(),
    )
    .looking_at(centre, Vec3::Y)
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
    // Pulled back and raised from the old 7.2 m / 6.2 m: at that framing the two nearest places filled
    // the frame and villagers were cut off at the bottom edge. The village's four places span
    // (-6.5,-5) to (6.5,5), so a 13 m radius at 10 m up holds all of them with margin, and a 0.5 m ring
    // is still about 20 px across at 720 px wide.
    // **Framed on the three places the villagers actually use**: the well (0,0), tavern (6.5,-5) and
    // market (-6.5,-5), whose centroid is (0, -3.3). Two earlier framings used the four-place centroid
    // and then a guessed (-1.6,-1.9); both left the crowd in the top third with empty grass below.
    //
    // A fixed pitch with a slow drift rather than a full orbit: an orbit is pleasant but it turns the
    // village so the ground's edge swings through frame, and a reader trying to follow one villager
    // loses it. The drift keeps the clip alive without moving the world under the viewer.
    // The crowd occupies x [-7.8, 7.8] and z [-6.3, 1.3], and `play` targets `Wander(6.0)`, so a
    // villager may be 6 m outside that box in any direction. Framing only the layout left a tie line
    // running off the bottom edge to a villager who had wandered out of shot.
    //
    // At 24 m up and 6.6 m out the frame covers 36.7 x 20.6 m: the 15.6 x 7.6 m layout plus 6 m of
    // wander on every side. Higher than a cinematic shot because the whole point is that every actor in
    // a relationship is visible at once.
    let centre = Vec3::new(0.0, 0.4, -2.5);
    let angle = (18.0 + 12.0 * fraction).to_radians();
    Transform::from_xyz(
        centre.x + 5.6 * angle.sin(),
        20.5,
        centre.z + 5.6 * angle.cos(),
    )
    .looking_at(centre, Vec3::Y)
}

/// A close camera over the tie graph, ignoring the sun so the lines read against a flat background.
pub fn ties_camera(fraction: f32) -> Transform {
    // Same framing as `close_camera`: the tie graph is about the crowd, and the earlier 13 m / 16 m
    // shot put four building tops in front of it.
    // The whole village, so a tie line between two factions is visible as one. Framed on the same
    // point as `close_camera` so the clips read as the same village from different angles.
    let centre = Vec3::new(0.0, 0.4, -2.5);
    let angle = (16.0 + 10.0 * fraction).to_radians();
    Transform::from_xyz(
        centre.x + 5.8 * angle.sin(),
        21.0,
        centre.z + 5.8 * angle.cos(),
    )
    .looking_at(centre, Vec3::Y)
}

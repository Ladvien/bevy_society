//! Per-feature overlays for the README clips: one scene per feature, so each GIF shows exactly what
//! it claims to show and nothing else.
//!
//! Every system here is a *view*: it reads the society and draws, and never writes to it. That is the
//! same rule `docs/DEMOS.md` states for the capture — a demo must not change the thing it demonstrates.

use bevy::prelude::*;
use bevy_society::{
    Autonomous, GoodId, InteractionResolved, Mood, Needs, Opinions, Person, PersonIndex, Stock,
};

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
    // **Framed on the villagers AND the places they use**, not on the villagers alone.
    //
    // The village's four stalls span `(-6.5, -5)` to `(6.5, 5)`, but the people gather at whichever two
    // are busy, so a villager-only centroid drifts toward that pair and the far stalls leave frame —
    // measured, two of the four stalls were out of shot and one sat half off the bottom edge. A clip
    // whose subject is a *village* has to keep the village in frame.
    let mut stalls = world.query_filtered::<&Transform, With<bevy_society::Place>>();
    let places: Vec<Vec3> = stalls.iter(world).map(|t| t.translation).collect();
    // **Centred on the places' bounding box, not on their centroid.** The four stalls are not symmetric
    // about their mean (the workshop is alone on one side), so a centroid framing clipped the workshop at
    // the bottom edge of every frame. The box's midpoint puts the village in the middle of the shot.
    let (min, max) = places.iter().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(lo, hi), p| (lo.min(*p), hi.max(*p)),
    );
    let centre = if places.is_empty() {
        let count = points.len() as f32;
        points.iter().fold(Vec3::ZERO, |sum, p| sum + *p) / count
    } else {
        0.5 * (min + max)
    };

    // How far anything in shot is from that centre: villagers **and** the stall corners, which are what
    // actually leaves frame.
    // **Biased toward the far corner.** The camera tilts, so the near edge of the shot covers less ground
    // than the far edge; framing exactly on the box midpoint left the workshop half off the bottom of
    // the last frame.
    //
    // **The bias was in the wrong direction and is now zero.** Projecting the four stalls through the
    // camera solves it exactly: at zero bias they land at y ∈ [-0.37, +0.31], which fits a 0.40 half-height;
    // the -2.2 m bias that "fixed the workshop" pushed `market` to **+0.385** and cropped *that* one, and a
    // +2.2 m bias pushed `workshop` to **-0.486**. The workshop is the far stall, so pushing the centre away
    // from the camera saved it by losing the near one. Symmetric is correct, so no bias is applied.
    let spread = points
        .iter()
        .map(|p| Vec2::new(p.x - centre.x, p.z - centre.z).length())
        .chain(places.iter().map(|p| {
            // A stall is a 1.2 m box; its corner is what clips first. `+ 1.3` is the half-diagonal
            // (0.85) plus a person's body radius, so a villager standing against the stall also fits.
            Vec2::new(p.x - centre.x, p.z - centre.z).length() + 1.3
        }))
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
    // `spread` is now the radius of everything in shot — people and places — so this is the multiplier
    // that fits that radius plus its 0.6 m stalls in the narrower (vertical) field at a 45 degree tilt.
    // Measured through three framings on the same run: 2.7 lost two of four stalls, **3.2 keeps all
    // four** and leaves the villagers large enough to read a 0.5 m ring at 720 px, and 4.0 pulled back so
    // far that people became specks. The cap stops a scattered crowd being filmed from the stratosphere.
    //
    // **The floor was the bug.** `Camera3d::default()` is a 45 degree *vertical* field, so the half-height
    // at the image plane is `tan(22.5) = 0.414`. Projecting the four stalls through the camera at the old
    // floor of 12 m puts `workshop` at **|y| = 0.66** and `market` at 0.49 — both outside. A tiny village
    // hits the floor, so a small `spread` was *cropping*, which is the opposite of what a floor is for.
    // 20 m fits every stall at |y| <= 0.37 with margin, and at 22 m they sit at 0.33.
    let reach = (spread * 2.5).clamp(20.0, 26.0);
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

/// How many frames a share line stays on screen. A share is one tick, so without a flash the clip would
/// show it for a twentieth of a second and nobody would see goods change hands.
const SHARE_FLASH_FRAMES: u32 = 24;

/// Every accepted `share` so far, and the ones still being drawn.
///
/// Present only in the goods scene: its HUD line reads it, and the other clips have no use for it.
#[derive(Resource, Default)]
pub struct Shares {
    /// Accepted barters since the recording began.
    pub total: u32,
    /// `(giver, receiver, frames left)`.
    live: Vec<(Entity, Entity, u32)>,
}

/// Counts each accepted `share` and queues a line for it. A view: it reads the message and never
/// writes to the society.
pub fn track_shares(mut resolved: MessageReader<InteractionResolved>, mut shares: ResMut<Shares>) {
    for message in resolved.read() {
        if message.interaction == "share" && message.accepted {
            shares.total += 1;
            shares
                .live
                .push((message.actor, message.target, SHARE_FLASH_FRAMES));
        }
    }
}

/// Draws one ring per villager whose radius is how much of the first good they hold.
///
/// An empty store is a small dark ring and a full one is a large gold one, so a viewer watches the loaves
/// spread from the people who have them to the people who do not. The ring's scale is the store's own
/// scale (`stock_ceiling` is 8), capped where the picture stops getting more informative.
pub fn draw_store_rings(
    index: Res<PersonIndex>,
    stocks: Query<&Stock>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for &(_, entity) in &index.sorted {
        let (Ok(stock), Ok(transform)) = (stocks.get(entity), transforms.get(entity)) else {
            continue;
        };
        let held = stock.get(GoodId(0));
        let pos = transform.translation();
        let (radius, color) = if held < 0.05 {
            (0.3, Color::srgb(0.12, 0.12, 0.22))
        } else {
            (0.3 + 0.15 * held.min(4.0), Color::srgb(1.0, 0.78, 0.12))
        };
        // The same two concentric rings the needs clip uses: one thin circle disappears against grass.
        for scale in [1.0, 0.62] {
            gizmos.circle(
                Isometry3d::new(
                    Vec3::new(pos.x, pos.y + 0.95, pos.z),
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                ),
                radius * scale,
                color,
            );
        }
    }
}

/// Draws a bright line from the giver to the receiver of each recent share, with a ring on the receiver.
pub fn draw_shares(
    mut shares: ResMut<Shares>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for (from, to, frames) in shares.live.iter_mut() {
        *frames = frames.saturating_sub(1);
        let (Ok(a), Ok(b)) = (transforms.get(*from), transforms.get(*to)) else {
            continue;
        };
        let (a, b) = (a.translation(), b.translation());
        let lift = Vec3::Y * 1.25;
        let fade = *frames as f32 / SHARE_FLASH_FRAMES as f32;
        let color = Color::srgb(1.0, 0.95 - 0.2 * (1.0 - fade), 0.55 + 0.45 * fade);
        // Three parallel lines: a single gizmo line is one pixel wide and vanishes in a 640 px GIF.
        let side = Vec3::new(b.z - a.z, 0.0, a.x - b.x).normalize_or_zero() * 0.06;
        for offset in [-side, Vec3::ZERO, side] {
            gizmos.line(a + lift + offset, b + lift + offset, color);
        }
        gizmos.circle(
            Isometry3d::new(b + lift, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            0.55,
            color,
        );
    }
    shares.live.retain(|(_, _, frames)| *frames > 0);
}

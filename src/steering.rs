//! Social steering: walking persons avoid each other with ORCA.
//!
//! A person with a [`SocialMover`] and a [`Destination`] walks toward that point at `speed`,
//! deflected by ORCA (van den Berg et al. 2011, DOI 10.1109/TRO.2011.2120810) so that two movers
//! passing each other keep a gap instead of walking through.
//!
//! **Solid things are [`Obstacle`]s.** Attach one to anything round that a person must not walk
//! through, and its centre and radius are fed to ORCA as hard wall half-planes: the solver may slide
//! along the edge or walk away, but never accelerate into it. A final resolver also pushes a mover that
//! has ended up inside an obstacle out to its surface, so a person is never *drawn* inside a building
//! even when a step lands there.
//!
//! Without that, a host with a tavern had no way to say so and the people walked through it. The
//! README clips showed exactly that: villagers standing inside a 3 m block with half a body hidden.
//!
//! Movers must be root entities: the system reads and writes `Transform` (the local transform),
//! which is the value the simulation owns. `GlobalTransform` lags behind between fixed steps.
//!
//! **Neighbour search is [`bevy_orca::Crowd`]'s.** It owns the spatial hash, the scratch buffers and
//! the deterministic tie-break, so a crowd of a few hundred persons costs no per-tick allocation and
//! does not depend on the order the movers happen to be read in. Before 0.2 this module hand-rolled an
//! O(n²) all-pairs scan, which is the shape every ORCA host writes and none of them needs to.

use bevy::prelude::*;
use bevy_orca::Crowd;
use bevy_orca::{CrowdMember, CrowdParams};

use crate::behavior::Destination;
use crate::core::require_core;
use crate::{PersonIndex, SocietyPlane, SocietySettings, SocietySystems};

/// Movers closer than this (metres on the plane) are ORCA neighbours.
const NEIGHBOUR_RANGE: f32 = 3.0;
/// Seconds ahead that agent-agent collisions are anticipated.
const TIME_HORIZON: f32 = 2.0;
/// At most this many neighbours per mover, nearest first.
const MAX_NEIGHBOURS: usize = 10;
/// The symmetry kick, as a fraction of a mover's wish. ORCA has no reason to prefer left over right at
/// an exactly head-on meeting, so it stalls; `Crowd` nudges each member to a side hashed from its key.
const NUDGE: f32 = 0.05;
/// Below this speed (m/s) a mover keeps its facing instead of turning to its velocity.
const FACE_MIN_SPEED: f32 = 0.05;
/// Metres ahead that a solid thing starts constraining a mover's velocity.
const WALL_RANGE: f32 = 2.0;

/// Something round and solid that walkers must go around, in metres on the society plane.
///
/// Its `Transform` is the centre; `radius` is how far its surface is from that centre. A person is
/// both steered around it (as a hard wall half-plane, so ORCA never accelerates into it) and pushed
/// out of it if a step lands inside.
///
/// ```ignore
/// commands.spawn((Name::new("tavern"), Obstacle { radius: 2.0 }, Transform::from_xyz(6.5, 0.0, -5.0)));
/// ```
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct Obstacle {
    /// Surface distance from the centre, in metres on the plane.
    pub radius: f32,
}

impl Default for Obstacle {
    fn default() -> Self {
        Self { radius: 1.0 }
    }
}

/// Opt-in walking for a person. Add it to a `Person` that should be steered toward its
/// [`Destination`]; a `Person` without it is moved by the game.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[require(SocialVelocity)]
pub struct SocialMover {
    /// Preferred walking speed in metres per second.
    pub speed: f32,
    /// Body radius on the plane in metres, used for avoidance.
    pub radius: f32,
    /// Rotate about the plane's up axis to face the direction of travel.
    pub face_motion: bool,
}

impl Default for SocialMover {
    fn default() -> Self {
        Self {
            speed: 1.4,
            radius: 0.35,
            face_motion: true,
        }
    }
}

/// The velocity last computed for a [`SocialMover`], in metres per second on the plane.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Default)]
#[reflect(Component)]
pub struct SocialVelocity(pub Vec2);

/// Adds ORCA steering for [`SocialMover`]s. Requires [`crate::SocietyCorePlugin`] (or
/// [`crate::SocietyPlugins`]).
///
/// Steering runs in `FixedUpdate` after [`SocietySystems::Rules`] on every fixed step, not only on
/// society ticks, so motion stays smooth while the simulation ticks slowly.
pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SocialMover>()
            .register_type::<SocialVelocity>()
            .register_type::<Obstacle>()
            .add_systems(
                FixedUpdate,
                (steer_movers, push_out_of_obstacles)
                    .chain()
                    .after(SocietySystems::Rules),
            );
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "SteeringPlugin");
    }
}

/// Pushes any mover that has ended up inside an [`Obstacle`] back out to its surface.
///
/// ORCA's wall half-planes stop a mover *accelerating* into a solid, and the comment above them in
/// `bevy_orca` says so: *"Actual contact is still the caller's collision resolver."* This is that
/// resolver. Without it a step that lands inside a block — a spawn placed there, a fast arrival, or
/// the first step after an obstacle moves — leaves the person drawn inside it, which is what the README
/// clips showed.
///
/// The push is along the line from the obstacle's centre, so it is deterministic and needs no RNG. A
/// mover exactly on the centre has no direction to be pushed along; it is offset along `+X`, the same
/// arbitrary-but-fixed choice every time, so a replay matches.
fn push_out_of_obstacles(
    settings: Res<SocietySettings>,
    obstacles: Query<(&Obstacle, &Transform), Without<SocialMover>>,
    mut movers: Query<(&SocialMover, &mut Transform)>,
) {
    let plane = &settings.plane;
    let solids: Vec<(Vec2, f32)> = obstacles
        .iter()
        .map(|(obstacle, transform)| (to_plane(transform.translation, plane), obstacle.radius))
        .collect();
    if solids.is_empty() {
        return;
    }
    for (mover, mut transform) in &mut movers {
        let mut at = to_plane(transform.translation, plane);
        let mut moved = false;
        for (centre, radius) in &solids {
            let reach = radius + mover.radius;
            let away = at - *centre;
            let distance = away.length();
            if distance >= reach {
                continue;
            }
            let direction = if distance > f32::EPSILON {
                away / distance
            } else {
                Vec2::X
            };
            at = *centre + direction * reach;
            moved = true;
        }
        if moved {
            let height = transform.translation.y;
            transform.translation = match plane {
                SocietyPlane::XZ => Vec3::new(at.x, height, at.y),
                SocietyPlane::XY => Vec3::new(at.x, at.y, height),
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn steer_movers(
    index: Res<PersonIndex>,
    settings: Res<SocietySettings>,
    time: Res<Time<Fixed>>,
    obstacles: Query<(&Obstacle, &Transform), Without<SocialMover>>,
    mut movers: Query<(
        &SocialMover,
        &mut SocialVelocity,
        &mut Transform,
        Option<&Destination>,
    )>,
    // The batch solver, and the members slice it is handed, both persist between ticks:
    // `Crowd` clears its buffers rather than dropping them and `members` is a `Vec` reused the same
    // way, so a steady crowd allocates nothing per tick.
    mut crowd: Local<Crowd>,
    mut members: Local<Vec<CrowdMember>>,
) {
    let dt = time.delta_secs();
    let plane = &settings.plane;
    let stop_radius = 0.5 * settings.interaction_radius;

    // The solid things, read once: a walker's wall list is the same for everyone this step.
    let solids: Vec<(Vec2, f32)> = obstacles
        .iter()
        .map(|(obstacle, transform)| (to_plane(transform.translation, plane), obstacle.radius))
        .collect();

    // Build one member per mover, in sorted order. A mover without a destination is still a member:
    // its `pref_vel` is `None`, which means "another system moves this one this step", so everyone
    // takes the full avoidance against it rather than walking through it — the same meaning the
    // hand-written code gave a zero velocity.
    members.clear();
    members.reserve(index.sorted.len());
    for &(id, entity) in &index.sorted {
        let Ok((mover, velocity, transform, dest)) = movers.get(entity) else {
            continue;
        };
        let pos = to_plane(transform.translation, plane);
        let wish = dest.map(|d| {
            let to_dest = d.0 - pos;
            if to_dest.length() <= stop_radius {
                Vec2::ZERO
            } else {
                to_dest.normalize_or_zero() * mover.speed
            }
        });
        members.push(CrowdMember {
            key: u64::from(id.0),
            pos,
            vel: if wish.is_some() {
                velocity.0
            } else {
                Vec2::ZERO
            },
            radius: mover.radius,
            max_speed: mover.speed,
            pref_vel: wish,
        });
    }
    if members.is_empty() {
        return;
    }

    let params = CrowdParams {
        time_horizon: TIME_HORIZON,
        neighbour_range: NEIGHBOUR_RANGE,
        max_neighbours: MAX_NEIGHBOURS,
        nudge: NUDGE,
    };
    // Every mover is a neighbour of every other: society has no layers or teams to hide one from
    // another. Walls are `(b, approach)`, where `b` points from the mover at the solid and `approach`
    // is the fastest it may still close on it this step — the gap it has left, divided by `dt`. Only
    // solids within `WALL_RANGE` matter; a distant block cannot be reached this step.
    let solved = crowd.solve(
        &members,
        &params,
        dt,
        |_, _| true,
        |i, walls| {
            walls.clear();
            let Some(me) = members.get(i) else {
                return;
            };
            for (centre, radius) in &solids {
                let to_solid = *centre - me.pos;
                let gap = to_solid.length() - radius - me.radius;
                if gap > WALL_RANGE {
                    continue;
                }
                // Already overlapping: no closing at all, which lets it slide or back off.
                walls.push((
                    to_solid.normalize_or_zero(),
                    if gap <= 0.0 { 0.0 } else { gap / dt },
                ));
            }
        },
    );
    let velocities: Vec<Option<Vec2>> = solved.to_vec();

    // Store each velocity, move by it over this step, and face the direction of travel.
    for (&(_, entity), velocity) in index.sorted.iter().zip(velocities) {
        let Ok((mover, mut stored, mut transform, dest)) = movers.get_mut(entity) else {
            continue;
        };
        // A mover with no destination keeps its stored zero and does not move; `Crowd` reports `None`
        // for it, and a `Some` that came back non-finite is a solver bug rather than a position.
        let velocity = velocity.filter(|v| v.is_finite()).unwrap_or(Vec2::ZERO);
        if dest.is_none() {
            stored.0 = Vec2::ZERO;
            continue;
        }
        stored.0 = velocity;
        step_on_plane(&mut transform, velocity * dt, plane);
        if mover.face_motion && velocity.length() > FACE_MIN_SPEED {
            transform.rotation = facing(velocity, plane);
        }
    }
}

/// Project a world position onto the society plane as a 2-D point.
fn to_plane(p: Vec3, plane: &SocietyPlane) -> Vec2 {
    match plane {
        SocietyPlane::XZ => Vec2::new(p.x, p.z),
        SocietyPlane::XY => Vec2::new(p.x, p.y),
    }
}

/// Move a transform by a 2-D displacement on the society plane, leaving the normal axis alone.
fn step_on_plane(transform: &mut Transform, delta: Vec2, plane: &SocietyPlane) {
    match plane {
        SocietyPlane::XZ => {
            transform.translation.x += delta.x;
            transform.translation.z += delta.y;
        }
        SocietyPlane::XY => {
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        }
    }
}

/// The rotation that turns a model's forward axis (−Z on XZ, +X on XY) toward `velocity`.
fn facing(velocity: Vec2, plane: &SocietyPlane) -> Quat {
    match plane {
        // Rotating forward (0, 0, -1) about Y by θ gives (-sin θ, 0, -cos θ).
        SocietyPlane::XZ => Quat::from_rotation_y((-velocity.x).atan2(-velocity.y)),
        SocietyPlane::XY => Quat::from_rotation_z(velocity.y.atan2(velocity.x)),
    }
}

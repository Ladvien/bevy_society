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
//! Neighbour search is brute force over the movers, so it is O(n²) in the number of movers.

use bevy::prelude::*;
use bevy_orca::{Agent, new_velocity};

use crate::behavior::Destination;
use crate::core::require_core;
use crate::{PersonIndex, SocietyPlane, SocietySettings, SocietySystems};

/// Movers closer than this (metres on the plane) are ORCA neighbours.
const NEIGHBOUR_RANGE: f32 = 3.0;
/// Seconds ahead that agent-agent collisions are anticipated.
const TIME_HORIZON: f32 = 2.0;
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

/// One mover's state read before any velocity is computed, so that neighbours see the same
/// positions regardless of iteration order.
struct Snap {
    entity: Entity,
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    speed: f32,
    face: bool,
    dest: Option<Vec2>,
}

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
) {
    let dt = time.delta_secs();
    let plane = &settings.plane;
    let stop_radius = 0.5 * settings.interaction_radius;

    // The solid things, read once: a walker's wall list is the same for everyone this step.
    let solids: Vec<(Vec2, f32)> = obstacles
        .iter()
        .map(|(obstacle, transform)| (to_plane(transform.translation, plane), obstacle.radius))
        .collect();

    // Pass 1: read every mover in sorted order. A mover without a destination reports zero
    // velocity, so it is an obstacle that stands still.
    let mut snaps: Vec<Snap> = Vec::with_capacity(index.sorted.len());
    for &(_, entity) in &index.sorted {
        let Ok((mover, velocity, transform, dest)) = movers.get(entity) else {
            continue;
        };
        snaps.push(Snap {
            entity,
            pos: to_plane(transform.translation, plane),
            vel: if dest.is_some() {
                velocity.0
            } else {
                Vec2::ZERO
            },
            radius: mover.radius,
            speed: mover.speed,
            face: mover.face_motion,
            dest: dest.map(|d| d.0),
        });
    }

    // Pass 2: compute every velocity against the pass-1 snapshot.
    let mut velocities: Vec<Vec2> = Vec::with_capacity(snaps.len());
    for (i, me) in snaps.iter().enumerate() {
        let Some(dest) = me.dest else {
            velocities.push(Vec2::ZERO);
            continue;
        };
        let to_dest = dest - me.pos;
        let pref = if to_dest.length() <= stop_radius {
            Vec2::ZERO
        } else {
            to_dest.normalize_or_zero() * me.speed
        };
        let agent = Agent {
            pos: me.pos,
            vel: me.vel,
            radius: me.radius,
            avoids: true,
        };
        let neighbours: Vec<Agent> = snaps
            .iter()
            .enumerate()
            .filter(|(j, other)| *j != i && other.pos.distance(me.pos) <= NEIGHBOUR_RANGE)
            .map(|(_, other)| Agent {
                pos: other.pos,
                vel: other.vel,
                radius: other.radius,
                avoids: true,
            })
            .collect();
        // Walls, as `(b, approach)`: `b` points from the mover at the solid, and `approach` is the
        // fastest it may still close on it this step — the gap it has left, divided by `dt`. Only
        // solids within `WALL_RANGE` matter; a distant block cannot be reached this step.
        let walls: Vec<(Vec2, f32)> = solids
            .iter()
            .filter_map(|(centre, radius)| {
                let to_solid = *centre - me.pos;
                let gap = to_solid.length() - radius - me.radius;
                if gap > WALL_RANGE {
                    return None;
                }
                let b = to_solid.normalize_or_zero();
                // Already overlapping: no closing at all, which lets it slide or back off.
                Some((b, if gap <= 0.0 { 0.0 } else { gap / dt }))
            })
            .collect();
        velocities.push(new_velocity(
            &agent,
            pref,
            &neighbours,
            &walls,
            TIME_HORIZON,
            dt,
            me.speed,
        ));
    }

    // Pass 3: store each velocity, move by it over this step, and face the direction of travel.
    for (me, velocity) in snaps.iter().zip(velocities) {
        let Ok((_, mut stored, mut transform, _)) = movers.get_mut(me.entity) else {
            continue;
        };
        stored.0 = velocity;
        step_on_plane(&mut transform, velocity * dt, plane);
        if me.face && velocity.length() > FACE_MIN_SPEED {
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

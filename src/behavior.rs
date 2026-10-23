//! Daily-life data types: what a person is doing and where it is walking to.
//!
//! The decision and arrival systems that fill these components belong to `BehaviorPlugin`. This
//! module only defines the shared shapes, so the steering plugin and game code can read them
//! without depending on the behaviour systems.

use bevy::prelude::*;

use crate::defs::ActivityId;

/// The target of a [`Doing`] activity.
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum DoingTarget {
    /// The activity needs no place or person (for example `idle`, or `sleep` at home).
    Itself,
    /// A point on the society plane, chosen by a wander option.
    Point(Vec2),
    /// A place entity, chosen by a tag option.
    Place(Entity),
    /// Another person, chosen by a person option.
    Person(Entity),
}

/// The activity a person is currently performing.
///
/// A person re-decides only when its `Doing` ends. While `arrived_day` is `None` the person is
/// travelling; the day it is set, the activity's needs and duration start counting.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct Doing {
    /// The activity definition this person is following.
    pub activity: ActivityId,
    /// What the activity is directed at.
    pub target: DoingTarget,
    /// Game day the activity started.
    pub started_day: f64,
    /// Game day the person reached the target, or `None` while still travelling.
    pub arrived_day: Option<f64>,
}

/// Where a travelling person is walking to, on the society plane.
///
/// Inserted and updated while travelling and removed on arrival or when the activity ends. Games
/// without the steering plugin move persons themselves toward this point.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Default)]
#[reflect(Component)]
pub struct Destination(pub Vec2);

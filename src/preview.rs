//! **Preview**: run a society forward and put it back, reporting where each person went and which
//! ties moved, so an editor can show "what a day does to this village" and then reset.
//!
//! [`preview_forward`] is [`capture`] + [`fast_forward`] + [`capture`] + [`restore`], with the
//! persons' `Transform`s put back too (a save holds the activity and the walk target, not the
//! host-owned position). It is as exact as a save: what [`SocietySave`] does not hold (the `Prices`
//! book, host components) is not reset.

use std::collections::BTreeMap;
use std::fmt;

use bevy::ecs::world::World;
use bevy::math::Vec3;
use bevy::prelude::Entity;
use bevy::transform::components::Transform;

use crate::core::{Person, PersonKey, SocietySettings};
use crate::defs::ActivityId;
use crate::fast_forward::{FastForward, FastForwardError, fast_forward};
use crate::save::{RestoreError, SocietySave, capture, restore};

/// One tie that is not what it was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TieChange {
    /// The [`PersonKey`] the tie points at.
    pub target: u64,
    /// Affinity before, or `None` when the tie did not exist.
    pub before: Option<f32>,
    /// Affinity after, or `None` when the tie no longer exists.
    pub after: Option<f32>,
}

/// What a preview did to one person.
#[derive(Debug, Clone, PartialEq)]
pub struct PersonPreview {
    /// The person's [`PersonKey`].
    pub key: u64,
    /// Society-plane position before.
    pub from: [f32; 2],
    /// Society-plane position at the end of the run.
    pub to: [f32; 2],
    /// The activity running before, if any.
    pub doing_before: Option<ActivityId>,
    /// The activity running at the end of the run, if any.
    pub doing_after: Option<ActivityId>,
    /// Every tie whose existence or affinity changed, in target-key order.
    pub ties: Vec<TieChange>,
}

/// What [`preview_forward`] saw.
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// The run itself.
    pub run: FastForward,
    /// Every person, in [`PersonKey`] order.
    pub persons: Vec<PersonPreview>,
    /// Activities the reset could not put back (their place or person key did not resolve); those
    /// people start the restored society idle.
    pub dropped_activities: usize,
}

/// Why a [`preview_forward`] did not run or could not reset.
#[derive(Debug, Clone, PartialEq)]
pub enum PreviewError {
    /// The run was refused; nothing changed.
    Run(FastForwardError),
    /// This many persons have no [`PersonKey`], so a reset could not bring them back. Nothing
    /// changed: key them first (`PersonKey` is the host's own id).
    Unkeyed(usize),
    /// The society ran but the reset failed, so the world is left advanced.
    Reset(RestoreError),
}

impl fmt::Display for PreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Run(e) => write!(f, "{e}"),
            Self::Unkeyed(n) => write!(
                f,
                "{n} person(s) have no PersonKey, so the society could not be put back after a \
                 preview; insert a PersonKey on every person first"
            ),
            Self::Reset(e) => write!(f, "the preview ran but the reset failed: {e}"),
        }
    }
}

impl std::error::Error for PreviewError {}

fn positions(world: &mut World) -> BTreeMap<u64, (Entity, Vec3)> {
    let mut state = world.query::<(Entity, &PersonKey, &Transform)>();
    state
        .iter(world)
        .map(|(e, k, t)| (k.0, (e, t.translation)))
        .collect()
}

/// Runs `days` game-days, reports what changed, and puts the society back as it was.
///
/// Every person needs a [`PersonKey`]. On `Ok` the world is the society it was before the call
/// (state through [`restore`], positions through the saved `Transform`s); on [`PreviewError::Run`]
/// and [`PreviewError::Unkeyed`] nothing was touched.
///
/// # Errors
/// [`PreviewError`].
pub fn preview_forward(world: &mut World, days: f32) -> Result<Preview, PreviewError> {
    let people = world
        .query_filtered::<(), bevy::prelude::With<Person>>()
        .iter(world)
        .count();
    let keyed = positions(world).len();
    if keyed < people {
        return Err(PreviewError::Unkeyed(people - keyed));
    }
    let plane = world
        .get_resource::<SocietySettings>()
        .map(|s| s.plane)
        .unwrap_or_default();

    let before = capture(world);
    let start = positions(world);
    let run = fast_forward(world, days).map_err(PreviewError::Run)?;
    let after = capture(world);
    let end = positions(world);

    let report = restore(world, before.clone()).map_err(PreviewError::Reset)?;
    for (entity, at) in start.values() {
        if let Some(mut transform) = world.get_mut::<Transform>(*entity) {
            transform.translation = *at;
        }
    }

    let persons = describe(&before, &after, &start, &end, |v| {
        let p = plane.project(v);
        [p.x, p.y]
    });
    Ok(Preview {
        run,
        persons,
        dropped_activities: report.dropped_activities,
    })
}

fn describe(
    before: &SocietySave,
    after: &SocietySave,
    start: &BTreeMap<u64, (Entity, Vec3)>,
    end: &BTreeMap<u64, (Entity, Vec3)>,
    project: impl Fn(Vec3) -> [f32; 2],
) -> Vec<PersonPreview> {
    let later: BTreeMap<u64, &crate::save::PersonSave> =
        after.persons.iter().map(|p| (p.key, p)).collect();
    let mut out = Vec::new();
    for was in &before.persons {
        let Some(now) = later.get(&was.key) else {
            continue;
        };
        let Some((_, from)) = start.get(&was.key) else {
            continue;
        };
        let to = end.get(&was.key).map_or(*from, |(_, v)| *v);
        let mut ties = Vec::new();
        let old: BTreeMap<u64, f32> = was.ties.iter().map(|t| (t.target, t.affinity)).collect();
        let new: BTreeMap<u64, f32> = now.ties.iter().map(|t| (t.target, t.affinity)).collect();
        let mut targets: Vec<u64> = old.keys().chain(new.keys()).copied().collect();
        targets.sort_unstable();
        targets.dedup();
        for target in targets {
            let (b, a) = (old.get(&target).copied(), new.get(&target).copied());
            if b != a {
                ties.push(TieChange {
                    target,
                    before: b,
                    after: a,
                });
            }
        }
        out.push(PersonPreview {
            key: was.key,
            from: project(*from),
            to: project(to),
            doing_before: was.doing.map(|d| d.activity),
            doing_after: now.doing.map(|d| d.activity),
            ties,
        });
    }
    out.sort_by_key(|p| p.key);
    out
}

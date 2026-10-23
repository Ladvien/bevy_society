//! The rumor book: every rumor in the society, addressed by `RumorId` (plan D5).

use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::reflect::Reflect;

use crate::defs::DeedId;

/// Index of a rumor in the [`RumorBook`] (plan D5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct RumorId(pub u32);

/// One rumor about one person's deed (plan D5). `valence` is the deed's `harm`: negative means damaging
/// to the subject.
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Rumor {
    /// The deed the rumor reports.
    pub deed: DeedId,
    /// The person the rumor is about.
    pub subject: Entity,
    /// The person the deed was done to, when there was one.
    pub patient: Option<Entity>,
    /// The person who started the rumor.
    pub origin: Entity,
    /// Game-day the rumor was created.
    pub day: f32,
    /// Whether the rumor reports what really happened.
    pub truth: bool,
    /// Emotional charge in `[-1, 1]`; negative is damaging to the subject.
    pub valence: f32,
    /// How many persons have heard it (each person counted once).
    pub known_by: u32,
    /// Whether the rumor still circulates; cleared when its subject leaves the world.
    pub alive: bool,
}

/// Every rumor in the society, in creation order (plan D5). `RumorId(i)` addresses `rumors[i]`.
#[derive(Resource, Debug, Clone, Default, PartialEq, Reflect)]
pub struct RumorBook {
    /// The rumors, indexed by `RumorId`.
    pub rumors: Vec<Rumor>,
}

impl RumorBook {
    /// Adds `rumor` and returns its id. The creator is counted as the first knower.
    pub fn add(&mut self, rumor: Rumor) -> RumorId {
        let id = RumorId(self.rumors.len() as u32);
        self.rumors.push(rumor);
        id
    }

    /// The rumor with `id`, if it exists.
    pub fn get(&self, id: RumorId) -> Option<&Rumor> {
        self.rumors.get(id.0 as usize)
    }

    /// Mutable access to the rumor with `id`, if it exists.
    pub fn get_mut(&mut self, id: RumorId) -> Option<&mut Rumor> {
        self.rumors.get_mut(id.0 as usize)
    }

    /// The share of `living_persons` who know rumor `id`: `known_by / living_persons` (plan D5).
    /// Returns 0 when there are no living persons or the rumor is unknown.
    pub fn reach(&self, id: RumorId, living_persons: usize) -> f32 {
        match (self.get(id), living_persons) {
            (Some(r), n) if n > 0 => r.known_by as f32 / n as f32,
            _ => 0.0,
        }
    }

    /// Marks every rumor about `subject` as no longer circulating, because the subject left the world.
    /// Returns the ids retired, in ascending order, so callers can drop them from `Knowledge`.
    pub fn retire_about(&mut self, subject: Entity) -> Vec<RumorId> {
        let mut retired = Vec::new();
        for (i, rumor) in self.rumors.iter_mut().enumerate() {
            if rumor.alive && rumor.subject == subject {
                rumor.alive = false;
                retired.push(RumorId(i as u32));
            }
        }
        retired
    }
}

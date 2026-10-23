//! Factions, status and contests (plan D6, engine-free rules in `model::curves`).

use bevy::ecs::component::Component;
use bevy::ecs::resource::Resource;
use bevy::reflect::Reflect;

use crate::defs::FactionId;
use crate::model::curves;

/// Steepness of the contest win probability (plan D6). Bonabeau et al. 1995, DOI 10.1016/0378-4371(95)00064-e;
/// Hemelrijk 1999, DOI 10.1098/rspb.1999.0646.
pub const CONTEST_STEEPNESS: f32 = 4.0;

/// Familiarity needed before a respect value counts toward standing (plan D6).
pub const STANDING_FAMILIARITY: f32 = 0.2;

/// The factions a person belongs to, sorted by `FactionId` (plan D6).
#[derive(Component, Debug, Clone, Default, PartialEq, Reflect)]
pub struct Memberships(pub Vec<FactionId>);

impl Memberships {
    fn search(&self, faction: FactionId) -> Result<usize, usize> {
        self.0.binary_search_by_key(&faction.0, |f| f.0)
    }

    /// Whether this person belongs to `faction`.
    pub fn contains(&self, faction: FactionId) -> bool {
        self.search(faction).is_ok()
    }

    /// Adds `faction` in sorted position; returns `false` when already a member.
    pub fn insert(&mut self, faction: FactionId) -> bool {
        match self.search(faction) {
            Ok(_) => false,
            Err(i) => {
                self.0.insert(i, faction);
                true
            }
        }
    }

    /// Removes `faction`; returns `false` when not a member.
    pub fn remove(&mut self, faction: FactionId) -> bool {
        match self.search(faction) {
            Ok(i) => {
                self.0.remove(i);
                true
            }
            Err(_) => false,
        }
    }

    /// Whether this person and `other` share at least one faction; the in-group test of plan D6.
    pub fn shares_with(&self, other: &Memberships) -> bool {
        self.0.iter().any(|f| other.contains(*f))
    }
}

/// A person's standing: the mean respect others hold for them (plan D6). Magee & Galinsky 2008,
/// DOI 10.5465/19416520802211628, on status as conferred respect; Berger et al. 1972, DOI 10.2307/2093465.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Reflect)]
pub struct Status {
    /// Mean respect received, over those who know this person well enough.
    pub standing: f32,
}

/// Standing from `(respect, familiarity)` pairs, one per person who holds a tie toward this person.
/// Only pairs with `familiarity >= STANDING_FAMILIARITY` count; 0 when none do (plan D6).
pub fn standing_from(pairs: impl IntoIterator<Item = (f32, f32)>) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0u32);
    for (respect, familiarity) in pairs {
        if familiarity >= STANDING_FAMILIARITY {
            sum += respect;
            n += 1;
        }
    }
    if n == 0 { 0.0 } else { sum / n as f32 }
}

/// Probability that the actor wins a contest (plan D6): logistic of the difference in
/// `(dominance + standing)` between actor and target, with steepness [`CONTEST_STEEPNESS`].
pub fn contest_win_chance(
    actor_dominance: f32,
    actor_standing: f32,
    target_dominance: f32,
    target_standing: f32,
) -> f32 {
    let diff = (actor_dominance + actor_standing) - (target_dominance + target_standing);
    curves::logistic(CONTEST_STEEPNESS, 0.0, diff)
}

/// Per-faction aggregates, indexed by `FactionId` (plan D6). Recomputed each tick by the groups system.
#[derive(Debug, Clone, Default, PartialEq, Reflect)]
pub struct FactionStat {
    /// Number of living members.
    pub members: u32,
    /// Mean standing of the members.
    pub mean_standing: f32,
    /// Mean affinity among in-faction pairs.
    pub cohesion: f32,
    /// Mean affinity of members toward each other faction's members, indexed by `FactionId`.
    pub regard: Vec<f32>,
}

/// All factions' aggregates (plan D6). Index `i` is `FactionId(i)`.
#[derive(Resource, Debug, Clone, Default, PartialEq, Reflect)]
pub struct FactionStats(pub Vec<FactionStat>);

impl FactionStats {
    /// The aggregate for `faction`, if present.
    pub fn get(&self, faction: FactionId) -> Option<&FactionStat> {
        self.0.get(faction.0 as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memberships_stay_sorted_and_unique() {
        let mut m = Memberships::default();
        assert!(m.insert(FactionId(2)));
        assert!(m.insert(FactionId(0)));
        assert!(!m.insert(FactionId(2)));
        assert_eq!(m.0.iter().map(|f| f.0).collect::<Vec<_>>(), vec![0, 2]);
        assert!(m.remove(FactionId(0)));
        assert!(!m.contains(FactionId(0)));
    }

    #[test]
    fn standing_ignores_strangers() {
        // Only the pair with familiarity 0.5 counts; the stranger's respect of 1.0 is ignored.
        let s = standing_from([(0.6, 0.5), (1.0, 0.05)]);
        assert!((s - 0.6).abs() < 1e-6);
        assert_eq!(standing_from([(1.0, 0.0)]), 0.0);
    }

    #[test]
    fn contest_is_even_when_matched_and_favours_the_stronger() {
        assert!((contest_win_chance(0.2, 0.1, 0.2, 0.1) - 0.5).abs() < 1e-6);
        assert!(contest_win_chance(0.9, 0.3, 0.1, 0.0) > 0.9);
    }
}

//! Factions, status and contests (plan D6, engine-free rules in `model::curves`).

use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::{App, FixedUpdate, Plugin, Query, Res, ResMut};
use bevy::reflect::Reflect;
use bevy_emerge_rng::DetRng;

use crate::core::require_core;
use crate::defs::{FactionId, SocietyDefs};
use crate::mind::{EmotionKind, Emotions};
use crate::model::curves;
use crate::relationships::{Tie, Ties};
use crate::{PersonIndex, SocietyClock, SocietySettings, SocietySystems, Tuning};

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

    /// Whether the two people share at least one faction.
    pub fn shares_with(&self, other: &Memberships) -> bool {
        self.0.iter().any(|f| other.contains(*f))
    }
}

/// The in-group bias a new tie between co-members starts with (plan D6): the `ingroup_bias` of the
/// lowest faction both people belong to, from `SocietyDefs`. `None` when they share no faction.
pub(crate) fn shared_faction_bias(
    mine: &Memberships,
    theirs: &Memberships,
    defs: &SocietyDefs,
) -> Option<f32> {
    mine.0
        .iter()
        .find(|f| theirs.contains(**f))
        .and_then(|f| defs.factions.get(f.0 as usize))
        .map(|spec| spec.ingroup_bias)
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
/// `(dominance + standing)` between actor and target, with `steepness` from
/// `Tuning::contest_steepness` (Bonabeau et al. 1995, DOI 10.1016/0378-4371(95)00064-e;
/// Hemelrijk 1999, DOI 10.1098/rspb.1999.0646).
pub fn contest_win_chance(
    steepness: f32,
    actor_dominance: f32,
    actor_standing: f32,
    target_dominance: f32,
    target_standing: f32,
) -> f32 {
    let diff = (actor_dominance + actor_standing) - (target_dominance + target_standing);
    curves::logistic(steepness, 0.0, diff)
}

/// One side of a contest: everything the win chance reads and the outcome writes (plan D6).
pub(crate) struct Contestant<'a> {
    /// The person's entity, used as the target of the ties and emotions written.
    pub entity: Entity,
    /// Mood dominance at the start of the contest.
    pub dominance: f32,
    /// Standing at the start of the contest.
    pub standing: f32,
    /// This person's ties.
    pub ties: &'a mut Ties,
    /// This person's active emotions.
    pub emotions: &'a mut Emotions,
}

/// Resolves a contest between `actor` and `target` (plan D6). One draw from `rng` decides it. The
/// loser defers to the winner (`respect` +0.1 and `Shame`); the winner gains `Pride` and loses 0.05
/// respect toward the loser; every witness raises its respect for the winner by 0.05. Returns the
/// winner's entity.
///
/// The three deltas (0.1, 0.05, 0.05) and the two emotion intensities are design defaults; the
/// mechanism — a winner's status rising and a loser's deference — is Bonabeau et al. 1995 (DOI
/// 10.1016/0378-4371(95)00064-e) and Hemelrijk 1999 (DOI 10.1098/rspb.1999.0646).
pub(crate) fn resolve_contest<R: DetRng>(
    rng: &mut R,
    steepness: f32,
    actor: Contestant<'_>,
    target: Contestant<'_>,
) -> Entity {
    let p = contest_win_chance(
        steepness,
        actor.dominance,
        actor.standing,
        target.dominance,
        target.standing,
    );
    let (winner, loser) = if rng.unit() < f64::from(p) {
        (actor, target)
    } else {
        (target, actor)
    };
    let (winner_entity, loser_entity) = (winner.entity, loser.entity);

    let deference = loser.ties.entry(winner_entity);
    deference.respect = (deference.respect + 0.1).clamp(-1.0, 1.0);
    loser
        .emotions
        .appraise(EmotionKind::Shame, 0.4, Some(winner_entity));

    let dominance = winner.ties.entry(loser_entity);
    dominance.respect = (dominance.respect - 0.05).clamp(-1.0, 1.0);
    winner
        .emotions
        .appraise(EmotionKind::Pride, 0.4, Some(loser_entity));

    winner_entity
}

/// One witness of a contest raises its respect for the winner by 0.05 (plan D6). The caller loops over
/// the actor's near persons other than the pair.
pub(crate) fn witness_contest(ties: &mut Ties, winner: Entity) {
    let regard = ties.entry(winner);
    regard.respect = (regard.respect + 0.05).clamp(-1.0, 1.0);
}

/// Intergroup contact (plan D6; Pettigrew & Tropp 2006, DOI 10.1037/0022-3514.90.5.751): an accepted
/// interaction between people who share no faction adds `contact_gain` affinity in both directions.
pub(crate) fn intergroup_contact(a_to_b: &mut Tie, b_to_a: &mut Tie, tuning: &Tuning) {
    a_to_b.add_affinity(tuning.contact_gain, tuning);
    b_to_a.add_affinity(tuning.contact_gain, tuning);
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

/// Recomputes every person's [`Status`] from the respect held toward them (plan D6), in sorted
/// person order. Each tie contributes `(respect, familiarity)` to its target's list.
fn update_standing(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    ties: Query<&Ties>,
    mut status: Query<&mut Status>,
) {
    let now = clock.now_days as f32;
    let tuning = &settings.tuning;
    // Slot of each person by entity bits, so a tie's target finds its list without a HashMap.
    let mut slots: Vec<(u64, usize)> = index
        .sorted
        .iter()
        .enumerate()
        .map(|(i, &(_, entity))| (entity.to_bits(), i))
        .collect();
    slots.sort_unstable();
    let mut received: Vec<Vec<(f32, f32)>> = vec![Vec::new(); index.sorted.len()];
    for &(_, entity) in &index.sorted {
        let Ok(owned) = ties.get(entity) else {
            continue;
        };
        for tie in &owned.0 {
            let Ok(pos) = slots.binary_search_by_key(&tie.target.to_bits(), |&(bits, _)| bits)
            else {
                continue;
            };
            let slot = slots[pos].1;
            received[slot].push((tie.respect, tie.familiarity(now, tuning)));
        }
    }
    for (i, &(_, entity)) in index.sorted.iter().enumerate() {
        if let Ok(mut owned) = status.get_mut(entity) {
            owned.standing = standing_from(received[i].iter().copied());
        }
    }
}

/// Recomputes [`FactionStats`] (plan D6), in sorted person order. `regard[g]` is the mean affinity
/// of members of a faction toward the members of faction `g`; `cohesion` is `regard` of the faction
/// itself.
fn update_faction_stats(
    index: Res<PersonIndex>,
    defs: Res<SocietyDefs>,
    memberships: Query<&Memberships>,
    status: Query<&Status>,
    ties: Query<&Ties>,
    mut stats: ResMut<FactionStats>,
) {
    let count = defs.factions.len();
    let mut out = Vec::with_capacity(count);
    for f in 0..count {
        let faction = FactionId(f as u16);
        let mut stat = FactionStat {
            regard: vec![0.0; count],
            ..FactionStat::default()
        };
        let mut sums = vec![0.0f32; count];
        let mut counts = vec![0u32; count];
        let mut standing_sum = 0.0f32;
        for &(_, entity) in &index.sorted {
            let Ok(mine) = memberships.get(entity) else {
                continue;
            };
            if !mine.contains(faction) {
                continue;
            }
            stat.members += 1;
            if let Ok(owned_status) = status.get(entity) {
                standing_sum += owned_status.standing;
            }
            let Ok(owned) = ties.get(entity) else {
                continue;
            };
            for tie in &owned.0 {
                let Ok(theirs) = memberships.get(tie.target) else {
                    continue;
                };
                for group in &theirs.0 {
                    let g = group.0 as usize;
                    if g < count {
                        sums[g] += tie.affinity;
                        counts[g] += 1;
                    }
                }
            }
        }
        if stat.members > 0 {
            stat.mean_standing = standing_sum / stat.members as f32;
        }
        for g in 0..count {
            if counts[g] > 0 {
                stat.regard[g] = sums[g] / counts[g] as f32;
            }
        }
        stat.cohesion = stat.regard[f];
        out.push(stat);
    }
    stats.0 = out;
}

/// Plugin for factions, standing and faction aggregates (plan D6). Runs in `SocietySystems::Relate`
/// after the tie labels are refreshed, so standing reads settled ties. Requires `SocietyCorePlugin`.
pub struct GroupsPlugin;

impl Plugin for GroupsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Memberships>()
            .register_type::<Status>()
            .register_type::<FactionStat>()
            .register_type::<FactionStats>()
            .init_resource::<FactionStats>()
            .add_systems(
                FixedUpdate,
                (update_standing, update_faction_stats)
                    .chain()
                    .after(crate::relationships::refresh_labels)
                    .in_set(SocietySystems::Relate),
            );
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "GroupsPlugin");
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
        assert!((contest_win_chance(4.0, 0.2, 0.1, 0.2, 0.1) - 0.5).abs() < 1e-6);
        assert!(contest_win_chance(4.0, 0.9, 0.3, 0.1, 0.0) > 0.9);
    }
}

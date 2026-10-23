//! Opinions on topics and what a person knows about rumors (plan D5, engine-free rules in `model`).
//!
//! The functions here take plain numbers and return deltas or decisions, so the systems that own
//! the world (`BeliefsPlugin`, `interact`) apply them in deterministic `PersonIndex` order.

use bevy::app::{App, FixedUpdate, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageReader;
use bevy::ecs::query::QueryData;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Query, Res, ResMut};
use bevy::log::error;
use bevy::reflect::Reflect;

use crate::core::{
    PersonIndex, SocietyClock, SocietySettings, SocietySystems, Tuning, require_core,
};
use crate::defs::{SocietyDefs, TopicId};
use crate::model::{opinion, rumor_dk};
use crate::relationships::{Tie, Ties};
use crate::reputation::NormViolated;
use crate::rumor::{PlantRumor, RumorBook, RumorId, RumorLearned, plant_rumor};

/// One opinion on one topic (plan D5). `value` is in `[-1, 1]`, `stubbornness` in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Opinion {
    /// Current opinion in `[-1, 1]`.
    pub value: f32,
    /// The opinion the person was born with; the Friedkin–Johnsen anchor.
    pub anchor: f32,
    /// How strongly the anchor holds, in `[0, 1]`.
    pub stubbornness: f32,
}

impl Default for Opinion {
    fn default() -> Self {
        Self {
            value: 0.0,
            anchor: 0.0,
            stubbornness: 0.2,
        }
    }
}

impl Opinion {
    /// An opinion at `anchor` with the default stubbornness; the anchor is also the starting value.
    pub fn at(anchor: f32) -> Self {
        Self {
            value: anchor,
            anchor,
            ..Self::default()
        }
    }

    /// Applies one accepted contact with a person whose opinion is `other`, over the relationship
    /// weight `weight = 0.5·trust + 0.5·max(0, affinity)` (plan D5). Returns the affinity delta the
    /// caller should add to `aff(self → other)`, computed from the pre-update difference.
    ///
    /// Bounded confidence (Deffuant et al. 2000, DOI 10.1142/S0219525900000078), then the anchor
    /// pull (Friedkin & Johnsen 1990, DOI 10.1080/0022250X.1990.9990069). The similarity term
    /// `similarity_gain · (1 − |d|)` is Byrne 1961 (DOI 10.1037/h0044721) as used by McPherson et al.
    /// 2001 (DOI 10.1146/annurev.soc.27.1.415).
    pub fn influence(&mut self, other: f32, openness: f32, weight: f32, tuning: &Tuning) -> f32 {
        let d = other - self.value;
        let epsilon =
            opinion::confidence_bound(tuning.confidence_base, tuning.confidence_openness, openness);
        self.value +=
            opinion::deffuant_step(self.value, other, epsilon, tuning.influence_mu, weight);
        self.value += opinion::anchor_step(
            tuning.anchor_pull,
            self.stubbornness,
            self.anchor,
            self.value,
        );
        self.value = self.value.clamp(-1.0, 1.0);
        opinion::similarity_affinity_step(tuning.similarity_gain, d)
    }

    /// Conformity toward a group mean (Asch 1956, DOI 10.1037/h0093718; Nowak, Szamrej & Latané 1990,
    /// DOI 10.1037/0033-295X.97.3.362). Applies only when `|group_mean| > 0.3`, as the model gates it.
    pub fn conform(
        &mut self,
        group_mean: f32,
        n_in_contact: usize,
        agreeableness: f32,
        dt_days: f32,
        tuning: &Tuning,
    ) {
        if !opinion::conformity_applies(group_mean) {
            return;
        }
        self.value += opinion::conformity_step(
            tuning.conformity_per_day,
            agreeableness,
            n_in_contact,
            dt_days,
            group_mean,
            self.value,
        );
        self.value = self.value.clamp(-1.0, 1.0);
    }
}

/// A person's opinions, indexed by `TopicId` (plan D5). Sized to the defs by `size_person_state`.
#[derive(Component, Debug, Clone, Default, PartialEq, Reflect)]
pub struct Opinions(pub Vec<Opinion>);

impl Opinions {
    /// The opinion on topic index `topic`, if sized.
    pub fn get(&self, topic: usize) -> Option<&Opinion> {
        self.0.get(topic)
    }

    /// Mutable access to the opinion on topic index `topic`, if sized.
    pub fn get_mut(&mut self, topic: usize) -> Option<&mut Opinion> {
        self.0.get_mut(topic)
    }

    /// The current value on `topic`, or 0 when unsized.
    pub fn value(&self, topic: usize) -> f32 {
        self.0.get(topic).map_or(0.0, |o| o.value)
    }
}

/// What one person knows about one rumor (plan D5).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Known {
    /// The rumor known.
    pub rumor: RumorId,
    /// Belief in `[0, 1]`.
    pub belief: f32,
    /// Whether the person still tells it (`false` once it stifles, Daley & Kendall 1964, DOI 10.1038/2041118a0).
    pub spreader: bool,
    /// Game-day the rumor was first heard.
    pub heard_day: f32,
}

/// Everything a person knows about rumors, sorted by `RumorId` (plan D5).
#[derive(Component, Debug, Clone, Default, PartialEq, Reflect)]
pub struct Knowledge(pub Vec<Known>);

impl Knowledge {
    fn search(&self, rumor: RumorId) -> Result<usize, usize> {
        self.0.binary_search_by_key(&rumor.0, |k| k.rumor.0)
    }

    /// What this person knows of `rumor`, if anything.
    pub fn get(&self, rumor: RumorId) -> Option<&Known> {
        self.search(rumor).ok().map(|i| &self.0[i])
    }

    /// Whether this person knows `rumor`.
    pub fn knows(&self, rumor: RumorId) -> bool {
        self.search(rumor).is_ok()
    }

    /// Inserts `known` in sorted position, replacing any entry for the same rumor.
    pub fn insert(&mut self, known: Known) {
        match self.search(known.rumor) {
            Ok(i) => self.0[i] = known,
            Err(i) => self.0.insert(i, known),
        }
    }

    /// Mutable access to the entry for `rumor`, if known.
    pub fn get_mut(&mut self, rumor: RumorId) -> Option<&mut Known> {
        match self.search(rumor) {
            Ok(i) => Some(&mut self.0[i]),
            Err(_) => None,
        }
    }

    /// The rumor this person would tell next (plan D5 gossip rule). `eligible(rumor)` returns
    /// `Some(valence)` when the rumor is alive and its subject is not the listener. Picks the maximum of
    /// `|valence| · belief · 0.995^(hours since heard)` (Park et al. 2023, DOI 10.48550/arXiv.2304.03442),
    /// via [`rumor_dk::telling_weight`]. Ties go to the lowest `RumorId`. Only spreaders qualify.
    pub fn best_telling(
        &self,
        now_days: f32,
        eligible: impl Fn(RumorId) -> Option<f32>,
    ) -> Option<RumorId> {
        let mut best: Option<(f32, RumorId)> = None;
        for known in self.0.iter().filter(|k| k.spreader) {
            let Some(valence) = eligible(known.rumor) else {
                continue;
            };
            let hours = (now_days - known.heard_day).max(0.0) * 24.0;
            let weight = rumor_dk::telling_weight(valence, known.belief, hours);
            match best {
                Some((w, _)) if weight <= w => {}
                _ => best = Some((weight, known.rumor)),
            }
        }
        best.map(|(_, r)| r)
    }

    /// Removes every entry for `rumor` (a rumor retired because its subject left the world).
    pub fn forget(&mut self, rumor: RumorId) {
        if let Ok(i) = self.search(rumor) {
            self.0.remove(i);
        }
    }
}

/// Belief a listener takes away from hearing a rumor: `teller_belief · (0.5 + 0.5 · trust)`
/// (Sommerfeld et al. 2007, DOI 10.1073/pnas.0704598104), re-exported for callers of this module.
pub fn belief_after_hearing(teller_belief: f32, trust: f32) -> f32 {
    rumor_dk::belief_after_hearing(teller_belief, trust)
}

/// The weight one side of an exchange gives the other: `0.5 · trust + 0.5 · max(0, affinity)`
/// (plan D5), read from `ties` toward `other`, defaulting to a fresh [`Tie`] when there is none.
pub(crate) fn influence_weight(ties: &Ties, other: Entity) -> f32 {
    let tie = ties.get(other).copied().unwrap_or_else(|| Tie::new(other));
    0.5 * tie.trust + 0.5 * tie.affinity.max(0.0)
}

/// One side of an accepted interaction's opinion exchange (plan D5), over `topics`.
///
/// `other_values` holds the other side's opinions **as they were before the exchange began**, so
/// calling this once per direction with both snapshots taken first makes the pair's result
/// independent of which side moves first. The confidence bound uses this side's `openness` and the
/// influence weight is [`influence_weight`] from this side's ties. The value [`Opinion::influence`]
/// returns is added to this side's affinity toward `other`, clamped to `[-1, 1]`.
///
/// Deffuant et al. 2000 (DOI 10.1142/S0219525900000078); Friedkin & Johnsen 1990
/// (DOI 10.1080/0022250X.1990.9990069); Byrne 1961 (DOI 10.1037/h0044721).
pub(crate) fn exchange_opinions(
    opinions: &mut Opinions,
    ties: &mut Ties,
    openness: f32,
    other: Entity,
    other_values: &[(TopicId, f32)],
    tuning: &Tuning,
) {
    let weight = influence_weight(ties, other);
    let mut delta = 0.0;
    for &(topic, other_value) in other_values {
        let t = usize::from(topic.0);
        if let Some(opinion) = opinions.get_mut(t) {
            delta += opinion.influence(other_value, openness, weight, tuning);
        }
    }
    let tie = ties.entry(other);
    tie.affinity = (tie.affinity + delta).clamp(-1.0, 1.0);
}

/// One person's values on `topics`, read before any side of the exchange moves.
pub(crate) fn snapshot_values(opinions: &Opinions, topics: &[TopicId]) -> Vec<(TopicId, f32)> {
    topics
        .iter()
        .map(|&topic| (topic, opinions.value(usize::from(topic.0))))
        .collect()
}

/// Runs opinions, rumors and reputation: applies [`PlantRumor`] on every fixed step, then, on
/// society ticks, pulls every person's opinions toward the group mean it is in contact with.
/// Requires [`crate::SocietyCorePlugin`].
pub struct BeliefsPlugin;

impl Plugin for BeliefsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Opinion>()
            .register_type::<Opinions>()
            .register_type::<Known>()
            .register_type::<Knowledge>()
            .register_type::<RumorId>()
            .register_type::<crate::rumor::Rumor>()
            .register_type::<RumorBook>()
            .init_resource::<RumorBook>()
            .add_message::<PlantRumor>()
            .add_message::<RumorLearned>()
            .add_message::<NormViolated>()
            // Ungated, like every game-written input message: a message written between ticks is
            // dropped after two fixed-step buffer swaps, and society ticks are 16 steps apart.
            .add_systems(
                FixedUpdate,
                apply_planted_rumors.in_set(SocietySystems::Clock),
            )
            .add_systems(
                FixedUpdate,
                conform_opinions.in_set(SocietySystems::Propagate),
            );
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "BeliefsPlugin");
    }
}

/// Turns every pending [`PlantRumor`] into a rumor through [`plant_rumor`], in message order.
///
/// An unknown deed name is an `error!` and the message is skipped; an origin that is not a person
/// (it has no [`Knowledge`]) is an `error!` and skipped too. Runs in the ungated clock set so a
/// message written between ticks is never dropped.
fn apply_planted_rumors(
    mut reader: MessageReader<PlantRumor>,
    defs: Res<SocietyDefs>,
    clock: Res<SocietyClock>,
    mut book: ResMut<RumorBook>,
    mut knowledge: Query<&mut Knowledge>,
) {
    for msg in reader.read() {
        let Some(deed) = defs.deed_id(&msg.deed) else {
            error!(
                "bevy_society: PlantRumor names unknown deed '{}'; ignored",
                msg.deed
            );
            continue;
        };
        let Ok(mut origin_knowledge) = knowledge.get_mut(msg.origin) else {
            error!(
                "bevy_society: PlantRumor origin {:?} is not a person; ignored",
                msg.origin
            );
            continue;
        };
        let Some(spec) = defs.deeds.get(usize::from(deed.0)) else {
            continue;
        };
        plant_rumor(
            &mut book,
            &mut origin_knowledge,
            msg.origin,
            msg.subject,
            msg.patient,
            deed,
            spec.harm,
            clock.now_days as f32,
            msg.truth,
        );
    }
}

/// The per-person state conformity reads: opinions and the agreeableness of their owner.
#[derive(QueryData)]
#[query_data(mutable)]
struct Conformer {
    opinions: &'static mut Opinions,
    personality: &'static crate::mind::Personality,
}

/// Each person's in-contact group mean and contact count, per topic, read from the society as it
/// stood before any conformity step (plan D5).
///
/// The mean is over the people in `Perception.contact`, weighted by `max(0.1, affinity + 1)` from
/// the person's ties (`model::opinion::weighted_mean`); the person is excluded from its own group
/// even when `Perception.contact` lists it. `n` counts the contacts, so a person alone in the world
/// has `n = 0` and does not conform.
fn conformity_means(
    index: &PersonIndex,
    perception: &crate::Perception,
    people: &Query<Conformer>,
    ties: &Query<&Ties>,
    topics: usize,
) -> Vec<(Vec<f32>, usize)> {
    let mut out = Vec::with_capacity(index.sorted.len());
    for (i, &(_, entity)) in index.sorted.iter().enumerate() {
        let mut sums = vec![(0.0f32, 0.0f32); topics];
        let mut n = 0usize;
        let Ok(owned) = ties.get(entity) else {
            out.push((vec![0.0; topics], 0));
            continue;
        };
        for &j in perception.contact.get(i).map_or(&[][..], Vec::as_slice) {
            if j as usize == i {
                continue;
            }
            let Some(&other) = perception.persons.get(j as usize) else {
                continue;
            };
            let Ok(view) = people.get(other) else {
                continue;
            };
            n += 1;
            let affinity = owned.get(other).map_or(0.0, |t| t.affinity);
            let weight = (affinity + 1.0).max(0.1);
            for (t, slot) in sums.iter_mut().enumerate() {
                slot.0 += view.opinions.value(t) * weight;
                slot.1 += weight;
            }
        }
        let means = sums
            .into_iter()
            .map(|(sum, weight)| if weight > 0.0 { sum / weight } else { 0.0 })
            .collect();
        out.push((means, n));
    }
    out
}

/// Pulls every person's opinions toward the group mean of the people it is in contact with (plan
/// D5, conformity).
///
/// Two passes: the group means are computed from pre-step values for everyone, then each person
/// conforms in [`PersonIndex::sorted`] order. A person conforms only with at least three contacts
/// and only on a topic whose group mean is strong enough, as `model::opinion` decides.
///
/// Asch 1956 (DOI 10.1037/h0093718); Nowak, Szamrej & Latané 1990 (DOI 10.1037/0033-295X.97.3.362).
fn conform_opinions(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    defs: Res<SocietyDefs>,
    index: Res<PersonIndex>,
    perception: Res<crate::Perception>,
    mut people: Query<Conformer>,
    ties: Query<&Ties>,
) {
    let topics = defs.topics.len();
    if topics == 0 {
        return;
    }
    // One query, read in the first pass and written in the second: computing every mean from
    // pre-step values makes the result independent of the order persons conform in.
    let means = conformity_means(&index, &perception, &people, &ties, topics);
    let dt = clock.dt_days;
    for (i, &(_, entity)) in index.sorted.iter().enumerate() {
        let Some(&(ref topic_means, n)) = means.get(i) else {
            continue;
        };
        if n < 3 {
            continue;
        }
        let Ok(mut view) = people.get_mut(entity) else {
            continue;
        };
        let personality = view.personality;
        for (t, &mean) in topic_means.iter().enumerate() {
            if let Some(opinion) = view.opinions.get_mut(t) {
                opinion.conform(mean, n, personality.agreeableness, dt, &settings.tuning);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opinions_converge_when_within_the_bound() {
        let tuning = Tuning::default();
        let mut a = Opinion::at(0.1);
        let mut b = Opinion::at(0.3);
        a.stubbornness = 0.0;
        b.stubbornness = 0.0;
        for _ in 0..50 {
            let (av, bv) = (a.value, b.value);
            a.influence(bv, 0.5, 1.0, &tuning);
            b.influence(av, 0.5, 1.0, &tuning);
        }
        assert!((a.value - b.value).abs() < 0.01);
    }

    #[test]
    fn distant_opinions_do_not_move() {
        let tuning = Tuning::default();
        let mut a = Opinion::at(-0.6);
        a.stubbornness = 0.0;
        a.influence(0.6, 0.5, 1.0, &tuning);
        assert!((a.value + 0.6).abs() < 1e-6);
    }

    #[test]
    fn stubborn_anchor_holds() {
        let tuning = Tuning::default();
        let mut a = Opinion::at(0.1);
        a.stubbornness = 1.0;
        for _ in 0..200 {
            a.influence(0.2, 1.0, 1.0, &tuning);
        }
        assert!((a.value - a.anchor).abs() < 0.15);
    }

    #[test]
    fn knowledge_stays_sorted_and_tells_the_heaviest_rumor() {
        let mut k = Knowledge::default();
        for (id, belief) in [(3u32, 0.4f32), (1, 0.9), (2, 0.9)] {
            k.insert(Known {
                rumor: RumorId(id),
                belief,
                spreader: true,
                heard_day: 0.0,
            });
        }
        let ids: Vec<u32> = k.0.iter().map(|x| x.rumor.0).collect();
        assert_eq!(ids, vec![1, 2, 3]);
        // Equal weight for 1 and 2: the lowest RumorId wins.
        let pick = k.best_telling(0.0, |_| Some(-1.0));
        assert_eq!(pick, Some(RumorId(1)));
        // Rumor 1 is about the listener and therefore ineligible.
        let pick = k.best_telling(0.0, |r| if r.0 == 1 { None } else { Some(-1.0) });
        assert_eq!(pick, Some(RumorId(2)));
    }
}

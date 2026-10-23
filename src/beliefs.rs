//! Opinions on topics and what a person knows about rumors (plan D5, engine-free rules in `model`).
//!
//! The functions here take plain numbers and return deltas or decisions, so the systems that own
//! the world (`BeliefsPlugin`, `interact`) apply them in deterministic `PersonIndex` order.

use bevy::ecs::component::Component;
use bevy::reflect::Reflect;

use crate::model::{opinion, rumor_dk};
use crate::rumor::RumorId;

/// Rates for opinion dynamics, one field per plan default (D5).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct BeliefRates {
    /// Base confidence bound ε (Deffuant et al. 2000, DOI 10.1142/S0219525900000078).
    pub confidence_base: f32,
    /// Openness contribution to the confidence bound.
    pub confidence_openness: f32,
    /// Convergence rate μ of the bounded-confidence update (Deffuant et al. 2000).
    pub influence_mu: f32,
    /// Pull toward the anchor, scaled by stubbornness (Friedkin & Johnsen 1990, DOI 10.1080/0022250X.1990.9990069).
    pub anchor_pull: f32,
    /// Affinity gain from similarity (Byrne 1961, DOI 10.1037/h0044721; McPherson et al. 2001, DOI 10.1146/annurev.soc.27.1.415).
    pub similarity_gain: f32,
    /// Conformity rate per game-day (Asch 1956, DOI 10.1037/h0093718; Nowak, Szamrej & Latané 1990, DOI 10.1037/0033-295X.97.3.362).
    pub conformity_per_day: f32,
}

impl Default for BeliefRates {
    fn default() -> Self {
        Self {
            confidence_base: 0.2,
            confidence_openness: 0.6,
            influence_mu: 0.3,
            anchor_pull: 0.1,
            similarity_gain: 0.05,
            conformity_per_day: 0.2,
        }
    }
}

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
    pub fn influence(
        &mut self,
        other: f32,
        openness: f32,
        weight: f32,
        rates: &BeliefRates,
    ) -> f32 {
        let d = other - self.value;
        let epsilon =
            opinion::confidence_bound(rates.confidence_base, rates.confidence_openness, openness);
        self.value +=
            opinion::deffuant_step(self.value, other, epsilon, rates.influence_mu, weight);
        self.value += opinion::anchor_step(
            rates.anchor_pull,
            self.stubbornness,
            self.anchor,
            self.value,
        );
        self.value = self.value.clamp(-1.0, 1.0);
        opinion::similarity_affinity_step(rates.similarity_gain, d)
    }

    /// Conformity toward a group mean (Asch 1956, DOI 10.1037/h0093718; Nowak, Szamrej & Latané 1990,
    /// DOI 10.1037/0033-295X.97.3.362). Applies only when `|group_mean| > 0.3`, as the model gates it.
    pub fn conform(
        &mut self,
        group_mean: f32,
        n_in_contact: usize,
        agreeableness: f32,
        dt_days: f32,
        rates: &BeliefRates,
    ) {
        if !opinion::conformity_applies(group_mean) {
            return;
        }
        self.value += opinion::conformity_step(
            rates.conformity_per_day,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opinions_converge_when_within_the_bound() {
        let rates = BeliefRates::default();
        let mut a = Opinion::at(0.1);
        let mut b = Opinion::at(0.3);
        a.stubbornness = 0.0;
        b.stubbornness = 0.0;
        for _ in 0..50 {
            let (av, bv) = (a.value, b.value);
            a.influence(bv, 0.5, 1.0, &rates);
            b.influence(av, 0.5, 1.0, &rates);
        }
        assert!((a.value - b.value).abs() < 0.01);
    }

    #[test]
    fn distant_opinions_do_not_move() {
        let rates = BeliefRates::default();
        let mut a = Opinion::at(-0.6);
        a.stubbornness = 0.0;
        a.influence(0.6, 0.5, 1.0, &rates);
        assert!((a.value + 0.6).abs() < 1e-6);
    }

    #[test]
    fn stubborn_anchor_holds() {
        let rates = BeliefRates::default();
        let mut a = Opinion::at(0.1);
        a.stubbornness = 1.0;
        for _ in 0..200 {
            a.influence(0.2, 1.0, 1.0, &rates);
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

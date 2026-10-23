//! Directed ties between persons: affinity, trust, respect, familiarity and labels (plan D4).
//!
//! `Ties` is a component rather than Bevy relationships because the edges are weighted, directed
//! and many-to-many. Entries stay sorted by the target's bits so lookups are binary searches and
//! every iteration is deterministic.

use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::reflect::Reflect;
use serde::Deserialize;

use crate::model::{balance, recall};

/// Rates and capacities for ties, one field per plan default (D4).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct TieRates {
    /// Familiarity power-law time constant in game-days (Anderson & Schooler 1991, DOI 10.1111/j.1467-9280.1991.tb00174.x).
    pub familiarity_tau_days: f32,
    /// Familiarity power-law decay exponent (Murre & Dros 2015, DOI 10.1371/journal.pone.0120644).
    pub familiarity_decay_exp: f32,
    /// Mere-exposure rate on the familiarity peak per game-day (Zajonc 1968, DOI 10.1037/h0025848).
    pub exposure_per_day: f32,
    /// Mere-exposure rate on affinity per game-day (Zajonc 1968, DOI 10.1037/h0025848).
    pub mere_exposure_per_day: f32,
    /// Affinity relaxation toward zero per game-day, never crossing zero.
    pub affinity_relax_per_day: f32,
    /// Structural-balance rate per game-day (Heider 1946, DOI 10.1080/00223980.1946.9917275;
    /// Cartwright & Harary 1956, DOI 10.1037/h0046049).
    pub balance_rate_per_day: f32,
    /// Maximum ties kept per person (Dunbar 1992, DOI 10.1016/0047-2484(92)90081-J).
    pub max_ties: usize,
    /// Number of close ties kept as the sympathy layer (Zhou et al. 2005, DOI 10.1098/rspb.2004.2970).
    pub close_ties: usize,
}

impl Default for TieRates {
    fn default() -> Self {
        Self {
            familiarity_tau_days: 1.0,
            familiarity_decay_exp: 0.5,
            exposure_per_day: 2.0,
            mere_exposure_per_day: 0.05,
            affinity_relax_per_day: 0.01,
            balance_rate_per_day: 0.5,
            max_ties: 150,
            close_ties: 15,
        }
    }
}

/// The label a tie carries, recomputed each tick from affinity and familiarity (plan D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, Deserialize)]
pub enum TieLabel {
    /// Barely known: familiarity below 0.1.
    #[default]
    Stranger,
    /// Known but neither friend nor rival.
    Acquaintance,
    /// Affinity at least 0.4 and familiarity at least 0.3.
    Friend,
    /// Affinity at most -0.4 and familiarity at least 0.2.
    Rival,
}

impl TieLabel {
    /// Classifies a tie. Friend is checked first, then Rival, then Stranger; anything else is an
    /// Acquaintance. Thresholds are the plan's D4 values, grounded in the familiarity and
    /// similarity-attraction literature (Zajonc 1968, DOI 10.1037/h0025848; Byrne 1961,
    /// DOI 10.1037/h0044721).
    pub fn classify(affinity: f32, familiarity: f32) -> Self {
        if affinity >= 0.4 && familiarity >= 0.3 {
            Self::Friend
        } else if affinity <= -0.4 && familiarity >= 0.2 {
            Self::Rival
        } else if familiarity < 0.1 {
            Self::Stranger
        } else {
            Self::Acquaintance
        }
    }

    /// Lowercase name used by scripts (`"stranger" | "acquaintance" | "friend" | "rival"`).
    pub fn script_name(self) -> &'static str {
        match self {
            Self::Stranger => "stranger",
            Self::Acquaintance => "acquaintance",
            Self::Friend => "friend",
            Self::Rival => "rival",
        }
    }
}

/// One directed edge from the owning person to `target` (plan D4).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Tie {
    /// The person this edge points at.
    pub target: Entity,
    /// Liking in `[-1, 1]`.
    pub affinity: f32,
    /// Reliance in `[0, 1]`; 0.5 for a new acquaintance.
    pub trust: f32,
    /// Conferred esteem in `[-1, 1]`.
    pub respect: f32,
    /// Familiarity peak in `[0, 1]`, decayed by power-law forgetting when read.
    pub familiarity_peak: f32,
    /// Game-day of the last contact.
    pub last_contact_day: f32,
    /// The label as of the last refresh.
    pub label: TieLabel,
}

impl Tie {
    /// A fresh tie with the plan's defaults: neutral affinity and respect, trust 0.5, no familiarity.
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            affinity: 0.0,
            trust: 0.5,
            respect: 0.0,
            familiarity_peak: 0.0,
            last_contact_day: 0.0,
            label: TieLabel::Stranger,
        }
    }

    /// Familiarity at `now_days`, decayed from the peak by power-law forgetting (D4).
    pub fn familiarity(&self, now_days: f32, rates: &TieRates) -> f32 {
        recall::familiarity(
            self.familiarity_peak,
            now_days - self.last_contact_day,
            rates.familiarity_tau_days,
            rates.familiarity_decay_exp,
        )
    }

    /// Records one contact over `dt_days`. Familiarity is first read at `now_days`, then raised by
    /// mere exposure; affinity is nudged toward the target (Zajonc 1968, DOI 10.1037/h0025848).
    pub fn expose(&mut self, now_days: f32, dt_days: f32, rates: &TieRates) {
        let f = self.familiarity(now_days, rates);
        self.familiarity_peak = recall::exposed_peak(f, rates.exposure_per_day, dt_days);
        self.last_contact_day = now_days;
        self.affinity = (self.affinity
            + rates.mere_exposure_per_day * dt_days * (1.0 - self.affinity.abs()))
        .clamp(-1.0, 1.0);
    }

    /// Relaxes affinity toward zero by `affinity_relax_per_day · dt_days`, never crossing zero.
    pub fn relax(&mut self, dt_days: f32, rates: &TieRates) {
        self.affinity =
            balance::relax_toward_zero(self.affinity, rates.affinity_relax_per_day * dt_days);
    }

    /// Recomputes the label from the current affinity and familiarity. Returns `Some((old, new))`
    /// only when it changed, so the caller can write one `TieLabelChanged` per transition.
    pub fn refresh_label(
        &mut self,
        now_days: f32,
        rates: &TieRates,
    ) -> Option<(TieLabel, TieLabel)> {
        let next = TieLabel::classify(self.affinity, self.familiarity(now_days, rates));
        if next == self.label {
            None
        } else {
            let old = self.label;
            self.label = next;
            Some((old, next))
        }
    }
}

/// A person's outgoing ties, sorted by `target.to_bits()` with at most one entry per target.
#[derive(Component, Debug, Clone, Default, PartialEq, Reflect)]
pub struct Ties(pub Vec<Tie>);

impl Ties {
    /// Binary-search result for `target`: `Ok(index)` when present, `Err(insert_at)` otherwise.
    fn search(&self, target: Entity) -> Result<usize, usize> {
        let bits = target.to_bits();
        self.0.binary_search_by_key(&bits, |t| t.target.to_bits())
    }

    /// The tie to `target`, if any.
    pub fn get(&self, target: Entity) -> Option<&Tie> {
        self.search(target).ok().map(|i| &self.0[i])
    }

    /// Mutable access to the tie to `target`, if any.
    pub fn get_mut(&mut self, target: Entity) -> Option<&mut Tie> {
        match self.search(target) {
            Ok(i) => Some(&mut self.0[i]),
            Err(_) => None,
        }
    }

    /// The tie to `target`, inserted with [`Tie::new`] in sorted position when absent.
    pub fn entry(&mut self, target: Entity) -> &mut Tie {
        let index = match self.search(target) {
            Ok(i) => i,
            Err(i) => {
                self.0.insert(i, Tie::new(target));
                i
            }
        };
        &mut self.0[index]
    }

    /// Removes and returns the tie to `target`, if any.
    pub fn remove(&mut self, target: Entity) -> Option<Tie> {
        match self.search(target) {
            Ok(i) => Some(self.0.remove(i)),
            Err(_) => None,
        }
    }

    /// Number of ties.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no ties.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Drops the least-familiar ties until at most `rates.max_ties` remain (Dunbar 1992, DOI
    /// 10.1016/0047-2484(92)90081-J). Ties of equal familiarity go first by lower target bits, so the
    /// result is deterministic.
    pub fn trim(&mut self, now_days: f32, rates: &TieRates) {
        if self.0.len() <= rates.max_ties {
            return;
        }
        let excess = self.0.len() - rates.max_ties;
        let mut ranked: Vec<(f32, u64)> = self
            .0
            .iter()
            .map(|t| (t.familiarity(now_days, rates), t.target.to_bits()))
            .collect();
        ranked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let doomed: Vec<u64> = ranked.iter().take(excess).map(|&(_, bits)| bits).collect();
        self.0.retain(|t| !doomed.contains(&t.target.to_bits()));
    }

    /// The `rates.close_ties` targets with the highest `max(0, affinity) · familiarity`, best first.
    /// Ties of equal score go by lower target bits (Zhou et al. 2005, DOI 10.1098/rspb.2004.2970).
    pub fn close_targets(&self, now_days: f32, rates: &TieRates) -> Vec<Entity> {
        let mut scored: Vec<(f32, u64, Entity)> = self
            .0
            .iter()
            .map(|t| {
                let score = t.affinity.max(0.0) * t.familiarity(now_days, rates);
                (score, t.target.to_bits(), t.target)
            })
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored
            .into_iter()
            .take(rates.close_ties)
            .map(|(_, _, e)| e)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(bits: u64) -> Entity {
        Entity::from_bits(bits)
    }

    #[test]
    fn labels_follow_the_d4_thresholds() {
        assert_eq!(TieLabel::classify(0.4, 0.3), TieLabel::Friend);
        assert_eq!(TieLabel::classify(0.39, 0.3), TieLabel::Acquaintance);
        assert_eq!(TieLabel::classify(0.5, 0.29), TieLabel::Acquaintance);
        assert_eq!(TieLabel::classify(-0.4, 0.2), TieLabel::Rival);
        assert_eq!(TieLabel::classify(-0.4, 0.19), TieLabel::Acquaintance);
        assert_eq!(TieLabel::classify(0.9, 0.05), TieLabel::Stranger);
        assert_eq!(TieLabel::classify(0.0, 0.5), TieLabel::Acquaintance);
    }

    #[test]
    fn entries_stay_sorted_by_target_bits() {
        let mut ties = Ties::default();
        for bits in [30u64, 10, 20, 10] {
            ties.entry(e(bits));
        }
        let bits: Vec<u64> = ties.0.iter().map(|t| t.target.to_bits()).collect();
        assert_eq!(bits, vec![10, 20, 30]);
        assert!(ties.get(e(20)).is_some());
        assert!(ties.remove(e(20)).is_some());
        assert!(ties.get(e(20)).is_none());
    }

    #[test]
    fn trim_drops_least_familiar_first() {
        let rates = TieRates {
            max_ties: 2,
            ..TieRates::default()
        };
        let mut ties = Ties::default();
        for (bits, peak) in [(1u64, 0.9f32), (2, 0.1), (3, 0.5)] {
            let t = ties.entry(e(bits));
            t.familiarity_peak = peak;
            t.last_contact_day = 0.0;
        }
        ties.trim(0.0, &rates);
        let kept: Vec<u64> = ties.0.iter().map(|t| t.target.to_bits()).collect();
        assert_eq!(kept, vec![1, 3]);
    }

    #[test]
    fn label_change_is_reported_once() {
        let rates = TieRates::default();
        let mut tie = Tie::new(e(7));
        tie.affinity = 0.5;
        tie.familiarity_peak = 0.9;
        assert_eq!(
            tie.refresh_label(0.0, &rates),
            Some((TieLabel::Stranger, TieLabel::Friend))
        );
        assert_eq!(tie.refresh_label(0.0, &rates), None);
    }
}

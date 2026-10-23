//! Rumor spreading: the Daley–Kendall stifling rule, the telling weight used to choose what to pass
//! on, and the belief a listener forms. Engine-free.

use crate::model::recall::recency;

/// What a telling does to the teller, decided by whether the listener already knows the rumor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactOutcome {
    /// The listener did not know it: the listener learns it and keeps spreading it.
    Learn,
    /// The listener already knew it: the teller stops spreading it.
    Stifle,
}

/// Daley–Kendall: a telling to someone who already knows the rumor stifles the teller (Daley &
/// Kendall 1964, DOI 10.1038/2041118a0).
pub fn on_contact(listener_knows: bool) -> ContactOutcome {
    if listener_knows {
        ContactOutcome::Stifle
    } else {
        ContactOutcome::Learn
    }
}

/// Telling weight `|valence| · belief · 0.995^(hours since heard)`. The teller passes on the rumor
/// with the highest weight. The weighting is this crate's design: strong, fresh rumors win. Recency
/// is Park et al. 2023 (DOI 10.48550/arXiv.2304.03442).
pub fn telling_weight(valence: f32, belief: f32, hours_since_heard: f32) -> f32 {
    valence.abs() * belief * recency(hours_since_heard)
}

/// Belief a listener forms from a teller: `teller_belief · (0.5 + 0.5 · trust(listener → teller))`.
/// Trust is clamped to `[0, 1]` first.
///
/// Design default (no source paper gives this formula); mechanisms: Sommerfeld et al. 2007 (DOI
/// 10.1073/pnas.0704598104) on gossip as a reputation tool, and trust as the weight a listener gives
/// a teller. `docs/research/rumor-and-reputation.md` records what the rule rests on.
pub fn belief_after_hearing(teller_belief: f32, trust_listener_to_teller: f32) -> f32 {
    teller_belief * (0.5 + 0.5 * trust_listener_to_teller.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telling_someone_who_knows_stifles() {
        assert_eq!(on_contact(true), ContactOutcome::Stifle);
        assert_eq!(on_contact(false), ContactOutcome::Learn);
    }

    #[test]
    fn fresher_and_stronger_rumors_are_told_first() {
        assert!(telling_weight(-0.7, 1.0, 0.0) > telling_weight(-0.3, 1.0, 0.0));
        assert!(telling_weight(-0.7, 1.0, 1.0) > telling_weight(-0.7, 1.0, 100.0));
    }

    #[test]
    fn belief_is_scaled_by_trust() {
        assert!((belief_after_hearing(1.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((belief_after_hearing(1.0, 0.0) - 0.5).abs() < 1e-6);
        assert!((belief_after_hearing(1.0, 5.0) - 1.0).abs() < 1e-6);
    }
}

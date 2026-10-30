//! Recall, forgetting and familiarity. Engine-free.

/// Recency of a memory `hours` old: `0.995^hours`, an exponential decay with a half-life of about
/// 138 hours. Park et al. 2023 (DOI 10.48550/arXiv.2304.03442) use this form for recency in their
/// generative agents' memory retrieval.
///
/// One departure: `hours` is measured **from the memory's creation**, not from its last retrieval.
/// Park et al. re-stamp a memory each time it is recalled; a society's memories are never replayed,
/// so creation time is the only stamp that exists.
pub fn recency(hours_since: f32) -> f32 {
    0.995_f32.powf(hours_since.max(0.0))
}

/// Familiarity after power-law forgetting: `peak · (1 + elapsed/τ)^(−d)`, with `elapsed` in game-days
/// since the last contact. Power-law forgetting: Anderson & Schooler 1991 (DOI 10.1111/j.1467-9280.1991.tb00174.x);
/// Murre & Dros 2015 (DOI 10.1371/journal.pone.0120644) fit this form to recall data. The exponent 0.5
/// is this crate's default, to be confirmed against `docs/research/memory.md`.
pub fn familiarity(peak: f32, elapsed_days: f32, tau_days: f32, decay_exp: f32) -> f32 {
    peak * (1.0 + elapsed_days.max(0.0) / tau_days).powf(-decay_exp)
}

/// Mere exposure: each contact raises the familiarity peak toward 1 by `rate · dt · (1 − f)`, where
/// `f` is the familiarity just before the contact (Zajonc 1968, DOI 10.1037/h0025848).
pub fn exposed_peak(familiarity_now: f32, rate_per_day: f32, dt_days: f32) -> f32 {
    (familiarity_now + rate_per_day * dt_days * (1.0 - familiarity_now)).clamp(0.0, 1.0)
}

/// Grudge: the **maximum** of `max(0, −valence) · recency` over the memories about one person, clamped
/// to `[0, 1]`. Each item is `(valence, recency)`. Park et al. 2023 (DOI 10.48550/arXiv.2304.03442)
/// weight retrieved memories by recency.
///
/// A design default. It was a **sum**, which made the value depend on how many entries the memory buffer
/// happened to keep: four trivial slights reached 1.0 exactly as one betrayal did. Measured on a cluster
/// of twelve, `snub` — whose only consideration is this — was 50.3% of activity starts when summing and
/// 26.0% when taking the maximum.
pub fn grudge(memories: impl IntoIterator<Item = (f32, f32)>) -> f32 {
    memories
        .into_iter()
        .map(|(valence, r)| (-valence).max(0.0) * r)
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0)
}

/// How many times faster a grievance fades toward someone the victim still trusts: `1 + forgiveness · trust`.
/// The grudge's recency reads the memory's age multiplied by this pace, so at pace 2 a grievance is as faded
/// after three days as it would have been after six.
///
/// **Forgiveness tracks relationship value.** McCullough, Root Luna, Berry, Tabak & Bono 2010 (DOI
/// 10.1037/a0019349) found that forgiveness grows with time since the offence, at a rate that differs between
/// people, and that it was "uniquely associated with participants' perceptions that their relationships with
/// their offenders retained value". Burnette, McCullough, Van Tongeren & Davis 2012 (DOI
/// 10.1177/0146167211424582): "the combination of low exploitation risk and high relationship value predicted
/// the greatest forgiveness". Strelan, Karremans & Krieg 2017 (DOI 10.1111/bjso.12173): the closeness →
/// forgiveness link is carried by post-transgression **trust**, which "always played the main mediating role".
/// So the input is the victim's current trust in the offender — one number that rises with good conduct and
/// falls with each new offence, which is value and exploitation risk read together.
///
/// The papers give the direction and the mediator, not a rate in game-days: `forgiveness` is a **design
/// default**. `0.0` (or less) is pace 1, the recency curve unchanged; trust is clamped to `[0, 1]`.
pub fn forgiving_pace(trust: f32, forgiveness: f32) -> f32 {
    1.0 + forgiveness.max(0.0) * trust.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recency_halves_in_about_138_hours() {
        assert!((recency(0.0) - 1.0).abs() < 1e-6);
        assert!((recency(138.0) - 0.5).abs() < 0.01);
        assert_eq!(recency(-5.0), 1.0);
    }

    #[test]
    fn familiarity_follows_the_power_law() {
        // peak · (1 + 3)^(−0.5) = peak / 2
        assert!((familiarity(0.8, 3.0, 1.0, 0.5) - 0.4).abs() < 1e-6);
        assert_eq!(familiarity(0.8, 0.0, 1.0, 0.5), 0.8);
    }

    #[test]
    fn exposure_approaches_one_and_never_exceeds_it() {
        let f = exposed_peak(0.0, 2.0, 1.0);
        assert!((f - 1.0).abs() < 1e-6);
        assert!(exposed_peak(0.9, 2.0, 1.0) <= 1.0);
    }

    #[test]
    fn grudge_counts_only_negative_memories_and_clamps() {
        assert_eq!(grudge([(0.5, 1.0), (0.3, 1.0)]), 0.0);
        // The maximum, not the sum: two 0.3 slights are one 0.3 slight, not a 0.6 grievance.
        assert!((grudge([(-0.3, 1.0), (-0.3, 1.0)]) - 0.3).abs() < 1e-6);
        assert!((grudge([(-0.5, 1.0), (-0.2, 1.0)]) - 0.5).abs() < 1e-6);
        assert!((grudge([(-0.3, 1.0)]) - 0.3).abs() < 1e-6);
        // Clamped at one, and repeated identical offences do not exceed the offence itself.
        assert_eq!(grudge([(-0.9, 1.0), (-0.9, 1.0)]), 0.9);
        assert_eq!(grudge([(-1.4, 1.0)]), 1.0);
    }

    #[test]
    fn a_trusted_offender_is_forgiven_faster_and_none_is_the_old_curve() {
        // Off: exactly the old recency, whatever the trust.
        assert_eq!(forgiving_pace(1.0, 0.0), 1.0);
        assert_eq!(forgiving_pace(0.3, -2.0), 1.0);
        // Pace grows with trust, and an offender nobody trusts gets none of it.
        assert_eq!(forgiving_pace(0.0, 2.0), 1.0);
        assert!((forgiving_pace(0.5, 2.0) - 2.0).abs() < 1e-6);
        assert!((forgiving_pace(1.0, 2.0) - 3.0).abs() < 1e-6);
        assert!(
            (forgiving_pace(7.0, 2.0) - 3.0).abs() < 1e-6,
            "trust is clamped"
        );
        // At pace 2 a grievance halves in about 69 hours instead of 138.
        assert!((recency(69.0 * forgiving_pace(0.5, 2.0)) - 0.5).abs() < 0.01);
    }
}

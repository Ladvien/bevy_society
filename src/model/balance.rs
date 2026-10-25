//! Structural balance and affinity relaxation. Engine-free.

/// Balance-driven change to the direct tie A → C from a triad A → B → C:
/// `rate · dt · aff(A→B) · aff(B→C)`. Heider 1946 (DOI 10.1080/00223980.1946.9917275) and
/// Cartwright & Harary 1956 (DOI 10.1037/h0046049): a friend of a friend is pulled toward
/// friendship, a friend of an enemy toward dislike.
pub fn triad_delta(rate_per_day: f32, dt_days: f32, affinity_ab: f32, affinity_bc: f32) -> f32 {
    rate_per_day * dt_days * affinity_ab * affinity_bc
}

/// The **dyadic** half of the same theory: `aff(A→B)` and `aff(B→A)` are pulled toward their mean by
/// `rate · dt · (mean − own)`.
///
/// Heider's balance theory is usually stated for triads and it is also, and more simply, a statement
/// about a dyad: "in a dyadic relation, balance basically means reciprocity: feeling positive about a
/// person who doesn't reciprocate this feeling is unbalanced" (Heider 1946, as summarised in Schweighofer
/// & Schweitzer, DOI `10.18564/jasss.5323`, in the corpus). The crate implemented the triadic case in
/// [`triad_delta`] and not this one — measured on `village-evening`, **50%** of directed friend ties were
/// unreciprocated and the two directions of a pair disagreed by **0.145** on average, against a friend
/// band 0.05 wide.
///
/// Both directions move, which is what a *balance* step is: the pair converges on their mean rather than
/// one being dragged to the other. Returns the change to add to `affinity_ab`; the change to
/// `affinity_ba` is its negative, so the mean is conserved.
///
/// **Design default for the rate** — the source is qualitative and supplies no number.
pub fn dyad_delta(rate_per_day: f32, dt_days: f32, affinity_ab: f32, affinity_ba: f32) -> f32 {
    let mean = 0.5 * (affinity_ab + affinity_ba);
    // `min(1.0)` so a step can never overshoot the mean and oscillate around it.
    let step = (rate_per_day * dt_days).clamp(0.0, 1.0);
    (mean - affinity_ab) * step
}

/// Moves `value` toward zero by `amount` without crossing zero. Affinity fades toward neutral when
/// no interaction keeps it alive; the rate is a modelling choice with no source paper (see
/// `docs/research/relationships.md`).
pub fn relax_toward_zero(value: f32, amount: f32) -> f32 {
    if value > 0.0 {
        (value - amount).max(0.0)
    } else {
        (value + amount).min(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friend_of_a_friend_is_pulled_toward_friendship() {
        assert!(triad_delta(0.5, 1.0, 0.8, 0.8) > 0.0);
        assert!(triad_delta(0.5, 1.0, 0.8, -0.8) < 0.0);
    }

    #[test]
    fn a_dyad_converges_on_its_mean_without_overshooting() {
        // Two directions far apart: both move, and they meet.
        let d = dyad_delta(1.0, 0.5, 0.30, 0.10);
        assert!(
            (d - (0.20 - 0.30) * 0.5).abs() < 1e-6,
            "each moves half way to the mean"
        );
        // Half way each is `0.30 - 0.05 = 0.25` against `0.10 + 0.05 = 0.15`: closer, not yet equal.
        assert!(
            ((0.30 + d) - (0.10 - d) - 0.10).abs() < 1e-6,
            "the gap halves"
        );
        // With a full step they meet exactly.
        let d = dyad_delta(1.0, 1.0, 0.30, 0.10);
        assert!(
            ((0.30 + d) - (0.10 - d)).abs() < 1e-6,
            "a full step ends up equal"
        );
        // At the mean there is nothing to do.
        assert_eq!(dyad_delta(2.0, 0.1, 0.25, 0.25), 0.0);
        // A rate that would overshoot is clamped: the pair converges, it never oscillates.
        let d = dyad_delta(1000.0, 1.0, 0.30, 0.10);
        assert!(
            (0.30 + d - 0.20).abs() < 1e-6,
            "clamped to the mean, not past it"
        );
        // The mean is conserved.
        let (a, b) = (0.30_f32, -0.10_f32);
        let d = dyad_delta(0.5, 0.25, a, b);
        assert!(((a + d) + (b - d) - (a + b)).abs() < 1e-6);
    }

    #[test]
    fn relaxation_never_crosses_zero() {
        assert_eq!(relax_toward_zero(0.01, 0.05), 0.0);
        assert_eq!(relax_toward_zero(-0.01, 0.05), 0.0);
        assert!((relax_toward_zero(0.5, 0.1) - 0.4).abs() < 1e-6);
    }
}

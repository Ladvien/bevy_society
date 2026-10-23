//! Structural balance and affinity relaxation. Engine-free.

/// Balance-driven change to the direct tie A → C from a triad A → B → C:
/// `rate · dt · aff(A→B) · aff(B→C)`. Heider 1946 (DOI 10.1080/00223980.1946.9917275) and
/// Cartwright & Harary 1956 (DOI 10.1037/h0046049): a friend of a friend is pulled toward
/// friendship, a friend of an enemy toward dislike.
pub fn triad_delta(rate_per_day: f32, dt_days: f32, affinity_ab: f32, affinity_bc: f32) -> f32 {
    rate_per_day * dt_days * affinity_ab * affinity_bc
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
    fn relaxation_never_crosses_zero() {
        assert_eq!(relax_toward_zero(0.01, 0.05), 0.0);
        assert_eq!(relax_toward_zero(-0.01, 0.05), 0.0);
        assert!((relax_toward_zero(0.5, 0.1) - 0.4).abs() < 1e-6);
    }
}

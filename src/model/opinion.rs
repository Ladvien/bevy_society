//! Opinion dynamics: bounded-confidence influence, Friedkin–Johnsen anchoring, similarity-driven
//! affinity and conformity. Engine-free.

/// Confidence bound `ε_i = base + gain · O_i`, with `O_i` the listener's openness in `[0, 1]`
/// (Deffuant et al. 2000, DOI 10.1142/S0219525900000078).
pub fn confidence_bound(base: f32, gain: f32, openness: f32) -> f32 {
    base + gain * openness
}

/// Bounded-confidence pull `μ · w · (x_j − x_i)`, applied only when `|x_j − x_i| < ε`; otherwise `0`
/// (Deffuant et al. 2000, DOI 10.1142/S0219525900000078). `weight` is the trust-and-affinity weight
/// `w_ij` of the speaker to the listener.
pub fn deffuant_step(x_i: f32, x_j: f32, epsilon: f32, mu: f32, weight: f32) -> f32 {
    let d = x_j - x_i;
    if d.abs() < epsilon {
        mu * weight * d
    } else {
        0.0
    }
}

/// Friedkin–Johnsen pull toward the anchor: `rate · stubbornness · (anchor − x)` (Friedkin & Johnsen
/// 1990, DOI 10.1080/0022250X.1990.9990069).
pub fn anchor_step(rate: f32, stubbornness: f32, anchor: f32, x: f32) -> f32 {
    rate * stubbornness * (anchor - x)
}

/// Similarity attraction: affinity changes by `gain · (1 − |d|)`, where `d` is the opinion gap
/// before the update, so strong disagreement lowers affinity (Byrne 1961, DOI 10.1037/h0044721;
/// McPherson, Smith-Lovin & Cook 2001, DOI 10.1146/annurev.soc.27.1.415).
pub fn similarity_affinity_step(gain: f32, d: f32) -> f32 {
    gain * (1.0 - d.abs())
}

/// Conformity pull `rate · A_i · (n / (n + 3)) · dt · (m − x)` toward the in-contact group mean `m`,
/// with `n` persons in contact and `A_i` the person's agreeableness in `[0, 1]` (Asch 1956, DOI
/// 10.1037/h0093718; Nowak, Szamrej & Latané 1990, DOI 10.1037/0033-295X.97.3.362).
pub fn conformity_step(
    rate_per_day: f32,
    agreeableness: f32,
    n_in_contact: usize,
    dt_days: f32,
    group_mean: f32,
    x: f32,
) -> f32 {
    let n = n_in_contact as f32;
    rate_per_day * agreeableness * (n / (n + 3.0)) * dt_days * (group_mean - x)
}

/// Whether the group mean is strong enough to conform to: `|m| > 0.3`.
pub fn conformity_applies(group_mean: f32) -> bool {
    group_mean.abs() > 0.3
}

/// Affinity-weighted mean of `(value, affinity)` pairs, each weighted `max(0.1, affinity + 1)`, so a
/// stranger still counts a little. `None` when there are no samples. The weighting is this crate's
/// reading of Nowak, Szamrej & Latané 1990 (DOI 10.1037/0033-295X.97.3.362).
pub fn weighted_mean(samples: impl IntoIterator<Item = (f32, f32)>) -> Option<f32> {
    let (sum, weight) =
        samples
            .into_iter()
            .fold((0.0_f32, 0.0_f32), |(s, w), (value, affinity)| {
                let weight = (affinity + 1.0).max(0.1);
                (s + weight * value, w + weight)
            });
    if weight > 0.0 {
        Some(sum / weight)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn influence_only_inside_the_confidence_bound() {
        assert!(deffuant_step(0.1, 0.3, 0.5, 0.3, 1.0) > 0.0);
        assert_eq!(deffuant_step(-0.6, 0.6, 0.5, 0.3, 1.0), 0.0);
    }

    #[test]
    fn confidence_grows_with_openness() {
        assert!((confidence_bound(0.2, 0.6, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn anchor_pulls_stubborn_opinions_back() {
        assert!(anchor_step(0.1, 1.0, 0.1, 0.9) < 0.0);
        assert_eq!(anchor_step(0.1, 0.0, 0.1, 0.9), 0.0);
    }

    #[test]
    fn disagreement_lowers_affinity() {
        assert!(similarity_affinity_step(0.05, 0.1) > 0.0);
        assert!(similarity_affinity_step(0.05, 1.5) < 0.0);
    }

    #[test]
    fn weighted_mean_respects_affinity_and_empty_input() {
        assert_eq!(weighted_mean(std::iter::empty()), None);
        let m = weighted_mean([(1.0, 1.0), (0.0, -1.0)]).unwrap_or(f32::NAN);
        // weights 2.0 and 0.1: (2.0 + 0) / 2.1
        assert!((m - 2.0 / 2.1).abs() < 1e-6);
    }

    #[test]
    fn conformity_needs_a_strong_group_mean() {
        assert!(conformity_applies(0.5));
        assert!(!conformity_applies(0.2));
        assert!(conformity_step(0.2, 1.0, 3, 1.0, 0.8, 0.0) > 0.0);
    }
}

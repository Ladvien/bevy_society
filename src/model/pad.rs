//! Personality to mood baseline, after Mehrabian (1996) as used by ALMA. Engine-free.

/// Mood baseline `[pleasure, arousal, dominance]` from the Big Five, `traits = [O, C, E, A, N]`,
/// each in `[0, 1]`.
///
/// Mehrabian 1996 (DOI 10.1080/00049539608259510) as used by ALMA (Gebhard 2005, DOI
/// 10.1145/1082473.1082478). With t′ = 2t − 1 and emotional stability S = −N′:
/// P = 0.21·E′ + 0.59·A′ + 0.19·S, A = 0.15·O′ + 0.30·A′ − 0.57·S,
/// D = 0.25·O′ + 0.17·C′ + 0.60·E′ − 0.32·A′, each clamped to `[−1, 1]`.
pub fn baseline(traits: [f32; 5]) -> [f32; 3] {
    let [o, c, e, a, n] = traits.map(|t| 2.0 * t - 1.0);
    let s = -n;
    let pleasure = (0.21 * e + 0.59 * a + 0.19 * s).clamp(-1.0, 1.0);
    let arousal = (0.15 * o + 0.30 * a - 0.57 * s).clamp(-1.0, 1.0);
    let dominance = (0.25 * o + 0.17 * c + 0.60 * e - 0.32 * a).clamp(-1.0, 1.0);
    [pleasure, arousal, dominance]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_personality_has_a_neutral_baseline() {
        let [p, a, d] = baseline([0.5; 5]);
        assert!(p.abs() < 1e-6 && a.abs() < 1e-6 && d.abs() < 1e-6);
    }

    #[test]
    fn extraverted_agreeable_people_start_pleased_and_dominant() {
        // O′=0, C′=0, E′=1, A′=1, S=1 ⇒ P = 0.21 + 0.59 + 0.19 = 0.99, D = 0.60 − 0.32 = 0.28.
        let [p, _, d] = baseline([0.5, 0.5, 1.0, 1.0, 0.0]);
        assert!((p - 0.99).abs() < 1e-5, "pleasure {p}");
        assert!((d - 0.28).abs() < 1e-5, "dominance {d}");
    }

    #[test]
    fn neuroticism_lowers_stability_and_raises_arousal() {
        let calm = baseline([0.5, 0.5, 0.5, 0.5, 0.0]);
        let anxious = baseline([0.5, 0.5, 0.5, 0.5, 1.0]);
        assert!(anxious[1] > calm[1]);
    }
}

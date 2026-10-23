//! Personality, mood, emotions, needs and memories (plan D3). Pure data plus the update rules that
//! run on them; the systems that call these live in `MindPlugin`.

use bevy::ecs::component::Component;
use bevy::reflect::Reflect;
use bevy_emerge_rng::DetRng;
use serde::Deserialize;

use crate::defs::{DeedId, InteractionId};
use crate::model::pad;
use crate::model::recall;
use crate::rumor::RumorId;
use bevy::ecs::entity::Entity;

/// One of the five personality traits (McCrae & John 1992, DOI 10.1111/j.1467-6494.1992.tb00970.x).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Reflect)]
pub enum Trait {
    /// Openness to experience.
    Openness,
    /// Conscientiousness.
    Conscientiousness,
    /// Extraversion.
    Extraversion,
    /// Agreeableness.
    Agreeableness,
    /// Neuroticism (emotional instability is its negation).
    Neuroticism,
}

impl Trait {
    /// Parses a trait from its lowercase script name (`"openness"` … `"neuroticism"`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "openness" => Some(Self::Openness),
            "conscientiousness" => Some(Self::Conscientiousness),
            "extraversion" => Some(Self::Extraversion),
            "agreeableness" => Some(Self::Agreeableness),
            "neuroticism" => Some(Self::Neuroticism),
            _ => None,
        }
    }
}

/// The five traits, each in `[0, 1]`; default 0.5 on every trait.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Personality {
    /// Openness to experience, in `[0, 1]`.
    pub openness: f32,
    /// Conscientiousness, in `[0, 1]`.
    pub conscientiousness: f32,
    /// Extraversion, in `[0, 1]`.
    pub extraversion: f32,
    /// Agreeableness, in `[0, 1]`.
    pub agreeableness: f32,
    /// Neuroticism, in `[0, 1]`.
    pub neuroticism: f32,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        }
    }
}

impl Personality {
    /// The traits in the order `[O, C, E, A, N]` used by `model::pad`.
    pub fn as_array(&self) -> [f32; 5] {
        [
            self.openness,
            self.conscientiousness,
            self.extraversion,
            self.agreeableness,
            self.neuroticism,
        ]
    }

    /// The value of one trait.
    pub fn value(&self, which: Trait) -> f32 {
        match which {
            Trait::Openness => self.openness,
            Trait::Conscientiousness => self.conscientiousness,
            Trait::Extraversion => self.extraversion,
            Trait::Agreeableness => self.agreeableness,
            Trait::Neuroticism => self.neuroticism,
        }
    }

    /// Samples each trait as the mean of three uniform draws, which centres the population on 0.5
    /// and makes extremes rare (McCrae & John 1992, DOI 10.1111/j.1467-6494.1992.tb00970.x;
    /// Goldberg 1990, DOI 10.1037/0022-3514.59.6.1216).
    pub fn sample<R: DetRng>(rng: &mut R) -> Self {
        let mut mean3 = || (rng.unit() + rng.unit() + rng.unit()) as f32 / 3.0;
        Self {
            openness: mean3(),
            conscientiousness: mean3(),
            extraversion: mean3(),
            agreeableness: mean3(),
            neuroticism: mean3(),
        }
    }

    /// The baseline mood this personality settles back to (Mehrabian 1996, DOI
    /// 10.1080/00049539608259510; Gebhard 2005, DOI 10.1145/1082473.1082478).
    pub fn baseline_mood(&self) -> Mood {
        Mood::from_pad(pad::baseline(self.as_array()))
    }
}

/// Pleasure, arousal and dominance, each in `[-1, 1]` (Russell 1980, DOI 10.1037/h0077714;
/// Mehrabian 1996, DOI 10.1007/BF02686918).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct Mood {
    /// Pleasure (valence).
    pub pleasure: f32,
    /// Arousal.
    pub arousal: f32,
    /// Dominance.
    pub dominance: f32,
}

/// Rates of the mood update, per game-day (plan D3 defaults).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct MoodRates {
    /// How fast mood is pulled to the emotion centre (`mood_pull_per_day`, 4.0).
    pub pull_per_day: f32,
    /// How fast mood returns to baseline (`mood_return_per_day`, 1.0).
    pub return_per_day: f32,
    /// Emotional contagion rate (`contagion_per_day`, 0.3; Hatfield et al. 1993, DOI
    /// 10.1111/1467-8721.ep10770953).
    pub contagion_per_day: f32,
}

impl Default for MoodRates {
    fn default() -> Self {
        Self {
            pull_per_day: 4.0,
            return_per_day: 1.0,
            contagion_per_day: 0.3,
        }
    }
}

impl Mood {
    /// Builds a mood from a PAD triple, clamping each axis to `[-1, 1]`.
    pub fn from_pad(pad: [f32; 3]) -> Self {
        Self {
            pleasure: pad[0].clamp(-1.0, 1.0),
            arousal: pad[1].clamp(-1.0, 1.0),
            dominance: pad[2].clamp(-1.0, 1.0),
        }
    }

    /// The PAD triple `[P, A, D]`.
    pub fn as_array(&self) -> [f32; 3] {
        [self.pleasure, self.arousal, self.dominance]
    }

    /// One mood step over `dt_days` (plan D3). `centre` is the intensity-weighted PAD of the active
    /// emotions and `intensity` their capped total `I`. `contact_pleasure` is the affinity-weighted
    /// mean pleasure of the person's contacts (`None` when no contact has positive weight).
    /// `contagion_weight` scales the contagion term (the plan's `E`; the caller chooses it).
    ///
    /// Pull: Gebhard 2005 (DOI 10.1145/1082473.1082478); return to baseline: Mehrabian 1996 (DOI
    /// 10.1080/00049539608259510); contagion: Hatfield et al. 1993 (DOI 10.1111/1467-8721.ep10770953).
    pub fn step(
        &mut self,
        baseline: Mood,
        centre: [f32; 3],
        intensity: f32,
        contact_pleasure: Option<f32>,
        contagion_weight: f32,
        dt_days: f32,
        rates: &MoodRates,
    ) {
        let pull = (rates.pull_per_day * dt_days).min(1.0) * intensity;
        self.pleasure += (centre[0] - self.pleasure) * pull;
        self.arousal += (centre[1] - self.arousal) * pull;
        self.dominance += (centre[2] - self.dominance) * pull;

        let back = (rates.return_per_day * dt_days).min(1.0);
        self.pleasure += (baseline.pleasure - self.pleasure) * back;
        self.arousal += (baseline.arousal - self.arousal) * back;
        self.dominance += (baseline.dominance - self.dominance) * back;

        if let Some(contacts) = contact_pleasure {
            self.pleasure +=
                rates.contagion_per_day * contagion_weight * dt_days * (contacts - self.pleasure);
        }
        *self = Self::from_pad(self.as_array());
    }
}

/// The OCC emotion subset (Ortony, Clore & Collins; via Gratch & Marsella 2004, DOI
/// 10.1016/j.cogsys.2004.02.002, and Dias & Paiva 2005, DOI 10.1007/11595014_13).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Reflect)]
pub enum EmotionKind {
    /// Joy.
    Joy,
    /// Distress.
    Distress,
    /// Hope.
    Hope,
    /// Fear.
    Fear,
    /// Pride.
    Pride,
    /// Shame.
    Shame,
    /// Admiration.
    Admiration,
    /// Reproach.
    Reproach,
    /// Gratitude.
    Gratitude,
    /// Anger.
    Anger,
    /// Liking.
    Liking,
    /// Disliking.
    Disliking,
}

impl EmotionKind {
    /// The PAD vector of this emotion (ALMA's table, Gebhard 2005, DOI 10.1145/1082473.1082478).
    pub fn pad(self) -> [f32; 3] {
        match self {
            Self::Joy => [0.40, 0.20, 0.10],
            Self::Distress => [-0.40, -0.20, -0.50],
            Self::Hope => [0.20, 0.20, -0.10],
            Self::Fear => [-0.64, 0.60, -0.43],
            Self::Pride => [0.40, 0.30, 0.30],
            Self::Shame => [-0.30, 0.10, -0.60],
            Self::Admiration => [0.50, 0.30, -0.20],
            Self::Reproach => [-0.30, -0.10, 0.40],
            Self::Gratitude => [0.40, 0.20, -0.30],
            Self::Anger => [-0.51, 0.59, 0.25],
            Self::Liking => [0.40, 0.16, -0.24],
            Self::Disliking => [-0.40, 0.20, 0.10],
        }
    }
}

/// One active emotion. `toward` names the person it is about, when there is one.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ActiveEmotion {
    /// Which emotion.
    pub kind: EmotionKind,
    /// Intensity in `[0, 1]`.
    pub intensity: f32,
    /// The person the emotion is about.
    pub toward: Option<Entity>,
}

/// The emotions a person currently feels.
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
pub struct Emotions(pub Vec<ActiveEmotion>);

impl Emotions {
    /// Adds `intensity` to an active emotion with the same kind and target (capped at 1.0), or pushes
    /// a new one.
    pub fn appraise(&mut self, kind: EmotionKind, intensity: f32, toward: Option<Entity>) {
        if let Some(e) = self
            .0
            .iter_mut()
            .find(|e| e.kind == kind && e.toward == toward)
        {
            e.intensity = (e.intensity + intensity).min(1.0);
        } else {
            self.0.push(ActiveEmotion {
                kind,
                intensity: intensity.clamp(0.0, 1.0),
                toward,
            });
        }
    }

    /// Halves every intensity per `half_life_days` and drops those below 0.05 (plan D3, half-life 0.1
    /// game-day).
    pub fn decay(&mut self, dt_days: f32, half_life_days: f32) {
        let factor = 0.5f32.powf(dt_days / half_life_days);
        for e in &mut self.0 {
            e.intensity *= factor;
        }
        self.0.retain(|e| e.intensity >= 0.05);
    }

    /// The intensity-weighted PAD centre `V` and the capped total intensity `I = min(1, Σ intensity)`.
    pub fn virtual_centre(&self) -> ([f32; 3], f32) {
        let total: f32 = self.0.iter().map(|e| e.intensity).sum();
        if total <= 0.0 {
            return ([0.0; 3], 0.0);
        }
        let mut centre = [0.0f32; 3];
        for e in &self.0 {
            let pad = e.kind.pad();
            for axis in 0..3 {
                centre[axis] += pad[axis] * e.intensity;
            }
        }
        for c in &mut centre {
            *c /= total;
        }
        (centre, total.min(1.0))
    }

    /// The strongest active emotion, the first one on a tie.
    pub fn strongest(&self) -> Option<ActiveEmotion> {
        let mut best: Option<ActiveEmotion> = None;
        for e in &self.0 {
            if best.is_none_or(|b| e.intensity > b.intensity) {
                best = Some(*e);
            }
        }
        best
    }

    /// The summed intensity of one emotion kind, over all targets.
    pub fn intensity_of(&self, kind: EmotionKind) -> f32 {
        self.0
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| e.intensity)
            .sum::<f32>()
            .min(1.0)
    }
}

/// Need levels, indexed by `NeedId`. Level 1 is satisfied, 0 is desperate.
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
pub struct Needs(pub Vec<f32>);

impl Needs {
    /// The level of need `index`, or 0 when out of range.
    pub fn level(&self, index: usize) -> f32 {
        self.0.get(index).copied().unwrap_or(0.0)
    }

    /// Adds `delta` to need `index`, clamped to `[0, 1]`. Out-of-range indices are ignored.
    pub fn add(&mut self, index: usize, delta: f32) {
        if let Some(level) = self.0.get_mut(index) {
            *level = (*level + delta).clamp(0.0, 1.0);
        }
    }

    /// Decays every need by its `decay_per_day · dt_days`, never below 0 (Maslow 1943, DOI
    /// 10.1037/h0054346; Baumeister & Leary 1995, DOI 10.1037/0033-2909.117.3.497).
    pub fn decay(&mut self, decay_per_day: &[f32], dt_days: f32) {
        for (level, rate) in self.0.iter_mut().zip(decay_per_day) {
            *level = (*level - rate * dt_days).max(0.0);
        }
    }
}

/// What a memory is about.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum MemoryKind {
    /// A social exchange.
    Interaction {
        /// The interaction definition.
        id: InteractionId,
        /// Whether it was accepted.
        accepted: bool,
        /// Whether this person started it.
        initiated: bool,
    },
    /// A deed seen by the person.
    Witnessed {
        /// The deed definition.
        deed: DeedId,
        /// Who did it.
        actor: Entity,
    },
    /// A rumor heard from someone.
    HeardRumor(RumorId),
    /// A conversation with the player.
    Talked,
    /// A free-form memory from a script.
    Custom(String),
}

/// One remembered event.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct Memory {
    /// The game-day it happened.
    pub day: f32,
    /// What happened.
    pub kind: MemoryKind,
    /// The other person involved, if any.
    pub other: Option<Entity>,
    /// Good (+) or bad (−) for the rememberer, in `[-1, 1]`.
    pub valence: f32,
    /// How much the memory matters, in `[0, 1]`.
    pub importance: f32,
}

/// A bounded list of memories (`memory_capacity`, default 64; Park et al. 2023, DOI
/// 10.48550/arXiv.2304.03442).
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
pub struct Memories {
    /// The memories, in the order recorded.
    pub entries: Vec<Memory>,
}

impl Memories {
    /// The retention score `r + importance (+ 1 when about the given person)`, where `r` is the
    /// recency of the memory (Park et al. 2023, DOI 10.48550/arXiv.2304.03442).
    fn score(memory: &Memory, now_days: f64, about: Option<Entity>) -> f32 {
        let hours = ((now_days - memory.day as f64) * 24.0).max(0.0) as f32;
        let about_bonus = match (about, memory.other) {
            (Some(a), Some(o)) if a == o => 1.0,
            _ => 0.0,
        };
        recall::recency(hours) + memory.importance + about_bonus
    }

    /// Records a memory. When full, the lowest-scoring memory is evicted first (score without any
    /// "about" bonus).
    pub fn record(&mut self, memory: Memory, now_days: f64, capacity: usize) {
        if capacity == 0 {
            return;
        }
        while self.entries.len() >= capacity {
            let worst = self
                .entries
                .iter()
                .enumerate()
                .map(|(i, m)| (i, Self::score(m, now_days, None)))
                .fold(None::<(usize, f32)>, |acc, (i, s)| match acc {
                    Some((_, best)) if best <= s => acc,
                    _ => Some((i, s)),
                });
            match worst {
                Some((i, _)) => {
                    self.entries.remove(i);
                }
                None => break,
            }
        }
        self.entries.push(memory);
    }

    /// The `n` best memories by score, highest first. Ties keep recording order.
    pub fn recall(&self, now_days: f64, about: Option<Entity>, n: usize) -> Vec<&Memory> {
        let mut scored: Vec<(f32, &Memory)> = self
            .entries
            .iter()
            .map(|m| (Self::score(m, now_days, about), m))
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        scored.into_iter().take(n).map(|(_, m)| m).collect()
    }

    /// Grudge against `other`: the sum of `max(0, −valence) · r` over memories about them, clamped to
    /// `[0, 1]` (power-law forgetting, Anderson & Schooler 1991, DOI 10.1111/j.1467-9280.1991.tb00174.x).
    pub fn grudge(&self, other: Entity, now_days: f64) -> f32 {
        recall::grudge(self.entries.iter().filter_map(|m| {
            (m.other == Some(other)).then(|| {
                let hours = ((now_days - m.day as f64) * 24.0).max(0.0) as f32;
                (m.valence, recall::recency(hours))
            })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_emerge_rng::seeded;

    fn ent(bits: u64) -> Entity {
        Entity::from_bits(bits)
    }

    #[test]
    fn personality_sample_is_deterministic_and_in_range() {
        let mut a = seeded(3);
        let mut b = seeded(3);
        let pa = Personality::sample(&mut a);
        let pb = Personality::sample(&mut b);
        assert_eq!(pa, pb);
        for v in pa.as_array() {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn appraise_merges_same_kind_and_target_and_caps() {
        let mut e = Emotions::default();
        let who = Some(ent(1));
        e.appraise(EmotionKind::Anger, 0.7, who);
        e.appraise(EmotionKind::Anger, 0.7, who);
        e.appraise(EmotionKind::Anger, 0.2, None);
        assert_eq!(e.0.len(), 2);
        assert_eq!(e.0[0].intensity, 1.0);
    }

    #[test]
    fn emotions_decay_below_threshold_is_dropped() {
        let mut e = Emotions::default();
        e.appraise(EmotionKind::Joy, 0.5, None);
        e.decay(0.1, 0.1);
        assert!((e.0[0].intensity - 0.25).abs() < 1e-6);
        e.decay(0.5, 0.1);
        assert!(e.0.is_empty());
    }

    #[test]
    fn centre_is_weighted_mean_and_intensity_capped() {
        let mut e = Emotions::default();
        e.appraise(EmotionKind::Joy, 0.5, None);
        e.appraise(EmotionKind::Distress, 0.5, None);
        let (centre, total) = e.virtual_centre();
        assert!((total - 1.0).abs() < 1e-6);
        assert!((centre[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn needs_clamp_and_decay_floor_at_zero() {
        let mut n = Needs(vec![0.5, 0.2]);
        n.add(0, 1.0);
        assert_eq!(n.level(0), 1.0);
        n.decay(&[2.0, 0.1], 1.0);
        assert_eq!(n.level(0), 0.0);
        assert!((n.level(1) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn mood_relaxes_to_baseline_without_emotions() {
        let base = Mood::from_pad([0.3, -0.2, 0.1]);
        let mut m = Mood::from_pad([-0.9, 0.9, -0.9]);
        for _ in 0..200 {
            m.step(base, [0.0; 3], 0.0, None, 0.0, 0.05, &MoodRates::default());
        }
        assert!((m.pleasure - base.pleasure).abs() < 1e-3);
        assert!((m.arousal - base.arousal).abs() < 1e-3);
    }

    fn mem(day: f32, other: Option<Entity>, valence: f32, importance: f32) -> Memory {
        Memory {
            day,
            kind: MemoryKind::Talked,
            other,
            valence,
            importance,
        }
    }

    #[test]
    fn recall_prefers_about_then_recency() {
        let mut m = Memories::default();
        let a = ent(10);
        let b = ent(11);
        m.record(mem(0.0, Some(b), 0.0, 0.1), 0.0, 8);
        m.record(mem(0.0, Some(a), 0.0, 0.1), 0.0, 8);
        let top = m.recall(0.0, Some(a), 1);
        assert_eq!(top[0].other, Some(a));
    }

    #[test]
    fn record_evicts_lowest_score_when_full() {
        let mut m = Memories::default();
        m.record(mem(0.0, None, 0.0, 0.0), 0.0, 2);
        m.record(mem(0.0, None, 0.0, 0.9), 0.0, 2);
        m.record(mem(0.0, None, 0.0, 0.5), 0.0, 2);
        assert_eq!(m.entries.len(), 2);
        let importances: Vec<f32> = m.entries.iter().map(|e| e.importance).collect();
        assert!(!importances.contains(&0.0));
    }

    #[test]
    fn grudge_sums_negative_recent_memories_and_clamps() {
        let mut m = Memories::default();
        let a = ent(20);
        m.record(mem(0.0, Some(a), -0.6, 0.5), 0.0, 8);
        m.record(mem(0.0, Some(a), -0.6, 0.5), 0.0, 8);
        assert_eq!(m.grudge(a, 0.0), 1.0);
        assert_eq!(m.grudge(ent(21), 0.0), 0.0);
        let mut positive = Memories::default();
        positive.record(mem(0.0, Some(a), 0.8, 0.5), 0.0, 8);
        assert_eq!(positive.grudge(a, 0.0), 0.0);
    }
}

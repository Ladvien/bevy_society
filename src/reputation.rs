//! What witnesses, patients and actors make of a deed (plan D5, indirect reciprocity).
//!
//! Each helper here mutates only the components of the person it is about, which the caller passes
//! in as `&mut`, and reads every other person's state as copied scalars. Helpers never write
//! messages and never touch the `World`: they return the few values the caller has to write, so the
//! caller stays in charge of ordering and of the deterministic `PersonIndex.sorted` walk.

use bevy::ecs::entity::Entity;
use bevy::ecs::message::Message;

use crate::beliefs::{Knowledge, Opinions};
use crate::core::{AssessmentNorm, SocietySettings, Tuning};
use crate::defs::{DeedId, TopicId};
use crate::mind::{EmotionKind, Emotions, Memories, Memory, MemoryKind, Personality};
use crate::relationships::Ties;
use crate::rumor::{RumorBook, RumorId, plant_rumor};

/// Trust at or below which a patient counts as distrusted, so punishing them is justified
/// (Ohtsuki & Iwasa 2004, DOI 10.1016/j.jtbi.2004.06.005).
pub const DISTRUSTED: f32 = 0.3;

/// Written when a witness judges an actor's deed harshly enough to count as a norm violation
/// (plan D5): the witness's approval is below −0.5.
///
/// This is the society's own event. `TriggerOn::NormViolated` and the compiled
/// `TriggerKind::NormViolated` are the authored rules that match on it; all three are separate types
/// on purpose.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct NormViolated {
    /// The person who saw the deed.
    pub witness: Entity,
    /// The person who did it.
    pub actor: Entity,
    /// Name of the deed, as defined in the defs file.
    pub deed: String,
}

/// One witness's judgement of one deed (plan D5). The caller writes `NormViolated` and the rumor
/// from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WitnessReaction {
    /// The witness's approval, in `[-1, 1]`: its opinion on the deed's topic.
    pub(crate) approval: f32,
    /// Whether the approval is harsh enough for a [`NormViolated`] message.
    pub(crate) violated: bool,
}

/// The trust a witness, patient or actor moves by: `witness_gain · (0.5 + conscientiousness)`
/// (Nowak & Sigmund 1998, DOI 10.1038/31225).
pub(crate) fn trust_gain(tuning: &Tuning, conscientiousness: f32) -> f32 {
    tuning.witness_gain * (0.5 + conscientiousness)
}

/// One witness's reaction to a deed: trust, affinity, respect, an emotion, a `Witnessed` memory and
/// the approval the caller needs.
///
/// Trust moves by `trust_gain · harm` unless the witness uses [`AssessmentNorm::Standing`], the
/// deed is damaging and the patient is distrusted: then a justified punishment leaves the witness's
/// trust alone. Affinity moves by `0.15 · approval` and respect by `0.1 · approval`; the witness
/// feels `Admiration` (approval above zero) or `Reproach` (below) toward the actor.
///
/// Image scoring is Nowak & Sigmund 1998 (DOI 10.1038/31225); standing is Ohtsuki & Iwasa 2004
/// (DOI 10.1016/j.jtbi.2004.06.005). The affinity and respect weights are design defaults.
#[expect(
    clippy::too_many_arguments,
    reason = "one call per witnessed deed; the read/write split stays visible at the call site"
)]
pub(crate) fn witness_reaction(
    ties: &mut Ties,
    emotions: &mut Emotions,
    memories: &mut Memories,
    opinions: &Opinions,
    personality: &Personality,
    deed: DeedId,
    deed_topic: TopicId,
    harm: f32,
    actor: Entity,
    patient: Option<Entity>,
    trust_in_patient: Option<f32>,
    settings: &SocietySettings,
    // How many witnesses saw this deed, including this one; see `Tuning::witness_dilution`.
    witnesses: usize,
    now_days: f64,
) -> WitnessReaction {
    // EXPERIMENT: approval of the *act*, not of the norm. A witness who values kindness (opinion
    // > 0) disapproves of an insult (harm < 0).
    let norm = opinions.value(usize::from(deed_topic.0)).clamp(-1.0, 1.0);
    let approval = norm * harm.signum();
    // **A deed seen by many costs each of them less.** Without this the total cost grew linearly with the
    // crowd while a trust gain could only ever reach two people, so a village's trust could only fall.
    // `witness_dilution` is the exponent: 0.0 is the old behaviour, 1.0 divides by the count, 0.5 is the
    // square root. The dilution applies to the deed's *magnitude*, never to its direction or to whether it
    // is right to disapprove — a witness that objects still objects.
    let seen = (witnesses.max(1) as f32).powf(settings.tuning.witness_dilution);
    let gain = trust_gain(&settings.tuning, personality.conscientiousness) / seen;

    let justified = patient.is_some()
        && settings.assessment == AssessmentNorm::Standing
        && harm < 0.0
        && trust_in_patient.is_some_and(|trust| trust < DISTRUSTED);
    if !justified {
        let tie = ties.entry(actor);
        tie.trust = (tie.trust + gain * harm).clamp(0.0, 1.0);
    }

    let tie = ties.entry(actor);
    tie.add_affinity(
        settings.tuning.witness_affinity * approval,
        &settings.tuning,
    );
    tie.respect = (tie.respect + 0.1 * approval).clamp(-1.0, 1.0);

    if approval > 0.0 {
        emotions.appraise(EmotionKind::Admiration, approval, Some(actor));
    } else if approval < 0.0 {
        emotions.appraise(EmotionKind::Reproach, -approval, Some(actor));
    }

    memories.record(
        Memory {
            day: now_days as f32,
            kind: MemoryKind::Witnessed { deed, actor },
            other: Some(actor),
            valence: (approval * harm.abs()).clamp(-1.0, 1.0),
            importance: harm.abs().clamp(0.0, 1.0),
        },
        now_days,
        settings.memory_capacity,
    );

    WitnessReaction {
        approval,
        violated: approval < -0.5,
    }
}

/// The patient's reaction to a deed done to them (plan D5).
///
/// `Gratitude` when the deed helped, `Anger` when it hurt, both toward the actor and with intensity
/// `|harm|`. Trust moves by `trust_gain · harm`, where the caller computes `trust_gain` from the
/// patient's conscientiousness.
///
/// **Affinity moves by `witness_affinity · severity`**, and `severity` is the deed's own harm scaled by
/// [`Tuning::negativity_ratio`] when it is negative:
///
/// ```text
/// severity = harm                         if harm >= 0.0
///          = harm · negativity_ratio      if harm <  0.0
/// ```
///
/// Before 2026-10-27-03 this function moved no affinity at all, and the consequence was measurable: every
/// accepted `insult` in `village-evening` left the victim's affinity for its insulter at **+0.034** — positive,
/// set by mere exposure, beside the insulter's +0.131. A person who has just been insulted went on liking the
/// person who insulted them, and the more often they met, the more they liked them.
///
/// The asymmetry is **Baumeister, Bratslavsky, Finkenauer & Vohs 2001** (DOI `10.1037/1089-2680.5.4.323`),
/// reporting Gottman 1994: bad events are "on average five times as powerful as good ones", and "negative,
/// displeasurable behaviors were reciprocated to a significant degree, whereas the reciprocation of positive,
/// pleasurable behaviors was weaker and not significant". The paper gives the ratio; the weight is
/// [`Tuning::witness_affinity`].
///
/// **The patient reciprocates; it does not judge.** `witness_reaction` writes `witness_affinity · approval`,
/// where `approval = norm · sign(harm)` — a witness weighs the deed against its own opinion on the topic. The
/// patient's write reads the deed's `harm` directly, because reciprocation is a response to what happened
/// rather than an assessment of the kind of act. It is also the only reading that cannot be silently zero: an
/// opinion is 0.0 for a person whose `Opinions` were never sized, which would leave that victim exactly as
/// untouched as before.
pub(crate) fn patient_reaction(
    ties: &mut Ties,
    emotions: &mut Emotions,
    harm: f32,
    gain: f32,
    actor: Entity,
    tuning: &Tuning,
) {
    if harm > 0.0 {
        emotions.appraise(EmotionKind::Gratitude, harm, Some(actor));
    } else if harm < 0.0 {
        emotions.appraise(EmotionKind::Anger, -harm, Some(actor));
    }
    let severity = if harm < 0.0 {
        harm * tuning.negativity_ratio
    } else {
        harm
    };
    let tie = ties.entry(actor);
    tie.trust = (tie.trust + gain * harm).clamp(0.0, 1.0);
    tie.add_affinity(tuning.witness_affinity * severity, tuning);
}

/// The actor's own reaction to a deed it did (plan D5): `Pride` when its opinion on the deed's topic
/// is above 0.2, `Shame` when below −0.2, in both cases toward the patient and with intensity
/// `|opinion|`. The 0.2 thresholds are the plan's; the intensity is a design default.
pub(crate) fn actor_reaction(
    emotions: &mut Emotions,
    opinions: &Opinions,
    deed_topic: TopicId,
    patient: Entity,
) {
    let own = opinions.value(usize::from(deed_topic.0));
    if own > 0.2 {
        emotions.appraise(EmotionKind::Pride, own, Some(patient));
    } else if own < -0.2 {
        emotions.appraise(EmotionKind::Shame, -own, Some(patient));
    }
}

/// Starts one rumor about a witnessed deed (plan D5): the first witness in index order is the
/// origin, at full belief and spreading; every other witness knows it at full belief and spreads it
/// too. `known_by` counts every witness once. Returns the new rumor, or `None` when there were no
/// witnesses.
///
/// The caller owns each witness's `Knowledge` and writes every value back, because
/// `Query::get_mut` cannot hand out a slice of `&mut Knowledge` for a variable-length set.
#[expect(
    clippy::too_many_arguments,
    reason = "the caller owns this state anyway; bundling it into a struct would hide the read/write split"
)]
pub(crate) fn spread_witnessed_rumor(
    book: &mut RumorBook,
    witnesses: &mut [(Entity, Knowledge)],
    subject: Entity,
    patient: Option<Entity>,
    deed: DeedId,
    harm: f32,
    now_days: f32,
    truth: bool,
    knowledge_capacity: usize,
) -> Option<RumorId> {
    let (origin, rest) = witnesses.split_first_mut()?;
    let id = plant_rumor(
        book,
        &mut origin.1,
        origin.0,
        subject,
        patient,
        deed,
        harm,
        now_days,
        truth,
        knowledge_capacity,
    );
    for (_, knowledge) in rest.iter_mut() {
        knowledge.insert(
            crate::beliefs::Known {
                rumor: id,
                belief: 1.0,
                spreader: true,
                heard_day: now_days,
            },
            knowledge_capacity,
            now_days,
        );
    }
    if let Some(rumor) = book.get_mut(id) {
        rumor.known_by += rest.len() as u32;
    }
    Some(id)
}

#[cfg(test)]
mod patient_tests {
    use super::*;

    fn victim_after(harm: f32, tuning: &Tuning) -> f32 {
        let actor = Entity::from_raw_u32(7).expect("a valid entity index");
        let mut ties = Ties::default();
        let mut emotions = Emotions::default();
        patient_reaction(&mut ties, &mut emotions, harm, 0.0, actor, tuning);
        ties.get(actor).map_or(0.0, |t| t.affinity)
    }

    #[test]
    fn an_injury_lowers_and_a_benefit_raises_the_victims_affinity() {
        let t = Tuning::default();
        assert!(victim_after(-0.2, &t) < 0.0);
        assert!(victim_after(0.2, &t) > 0.0);
    }

    #[test]
    fn a_benefit_is_unscaled_and_an_injury_is_scaled_by_the_ratio() {
        let t = Tuning::default();
        let benefit = victim_after(0.04, &t);
        assert!((benefit - t.witness_affinity * 0.04).abs() < 1e-6);
        let injury = victim_after(-0.04, &t);
        assert!((injury / benefit + t.negativity_ratio).abs() < 1e-4);
    }

    #[test]
    fn ratio_zero_switches_off_harm_and_one_makes_it_symmetric() {
        let off = Tuning {
            negativity_ratio: 0.0,
            ..Tuning::default()
        };
        assert_eq!(victim_after(-0.3, &off), 0.0);
        let sym = Tuning {
            negativity_ratio: 1.0,
            ..Tuning::default()
        };
        assert!((victim_after(-0.1, &sym) + victim_after(0.1, &sym)).abs() < 1e-6);
    }

    #[test]
    fn the_write_respects_the_affinity_ceiling() {
        let t = Tuning::default();
        let actor = Entity::from_raw_u32(7).expect("a valid entity index");
        let mut ties = Ties::default();
        let mut emotions = Emotions::default();
        for _ in 0..10 {
            patient_reaction(&mut ties, &mut emotions, -0.9, 0.0, actor, &t);
        }
        let aff = ties.get(actor).map_or(0.0, |x| x.affinity);
        assert!((aff + t.affinity_ceiling).abs() < 1e-6);
    }
}

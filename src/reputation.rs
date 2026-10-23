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
    now_days: f64,
) -> WitnessReaction {
    let approval = opinions.value(usize::from(deed_topic.0)).clamp(-1.0, 1.0);
    let gain = trust_gain(&settings.tuning, personality.conscientiousness);

    let justified = patient.is_some()
        && settings.assessment == AssessmentNorm::Standing
        && harm < 0.0
        && trust_in_patient.is_some_and(|trust| trust < DISTRUSTED);
    if !justified {
        let tie = ties.entry(actor);
        tie.trust = (tie.trust + gain * harm).clamp(0.0, 1.0);
    }

    let tie = ties.entry(actor);
    tie.affinity = (tie.affinity + 0.15 * approval).clamp(-1.0, 1.0);
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

/// The patient's reaction to a deed done to them (plan D5): `Gratitude` when the deed helped,
/// `Anger` when it hurt, both toward the actor and with intensity `|harm|`; and trust moved by
/// `trust_gain · harm`, where the caller computes `trust_gain` from the patient's
/// conscientiousness.
pub(crate) fn patient_reaction(
    ties: &mut Ties,
    emotions: &mut Emotions,
    harm: f32,
    gain: f32,
    actor: Entity,
) {
    if harm > 0.0 {
        emotions.appraise(EmotionKind::Gratitude, harm, Some(actor));
    } else if harm < 0.0 {
        emotions.appraise(EmotionKind::Anger, -harm, Some(actor));
    }
    let tie = ties.entry(actor);
    tie.trust = (tie.trust + gain * harm).clamp(0.0, 1.0);
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
pub(crate) fn spread_witnessed_rumor(
    book: &mut RumorBook,
    witnesses: &mut [(Entity, &mut Knowledge)],
    subject: Entity,
    patient: Option<Entity>,
    deed: DeedId,
    harm: f32,
    now_days: f32,
    truth: bool,
) -> Option<RumorId> {
    let (origin, rest) = witnesses.split_first_mut()?;
    let id = plant_rumor(
        book, origin.1, origin.0, subject, patient, deed, harm, now_days, truth,
    );
    for (_, knowledge) in rest.iter_mut() {
        knowledge.insert(crate::beliefs::Known {
            rumor: id,
            belief: 1.0,
            spreader: true,
            heard_day: now_days,
        });
    }
    if let Some(rumor) = book.get_mut(id) {
        rumor.known_by += rest.len() as u32;
    }
    Some(id)
}

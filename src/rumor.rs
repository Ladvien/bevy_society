//! The rumor book: every rumor in the society, addressed by `RumorId` (plan D5).

use bevy::ecs::entity::Entity;
use bevy::ecs::message::Message;
use bevy::ecs::resource::Resource;
use bevy::reflect::Reflect;

use crate::beliefs::{Knowledge, Known, belief_after_hearing};
use crate::defs::DeedId;
use crate::relationships::Ties;

/// Index of a rumor in the [`RumorBook`] (plan D5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct RumorId(pub u32);

/// One rumor about one person's deed (plan D5). `valence` is the deed's `harm`: negative means damaging
/// to the subject.
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Rumor {
    /// The deed the rumor reports.
    pub deed: DeedId,
    /// The person the rumor is about.
    pub subject: Entity,
    /// The person the deed was done to, when there was one.
    pub patient: Option<Entity>,
    /// The person who started the rumor.
    pub origin: Entity,
    /// Game-day the rumor was created.
    pub day: f32,
    /// Whether the rumor reports what really happened.
    pub truth: bool,
    /// Emotional charge in `[-1, 1]`; negative is damaging to the subject.
    pub valence: f32,
    /// How many persons have heard it (each person counted once).
    pub known_by: u32,
    /// Whether the rumor still circulates; cleared when its subject leaves the world.
    pub alive: bool,
}

/// Game-facing request to start a rumor about `subject`'s deed, told by `origin` (plan D5). An unknown
/// `deed` name is logged and ignored.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PlantRumor {
    /// The person who starts the rumor; knows it from the start.
    pub origin: Entity,
    /// The person the rumor is about.
    pub subject: Entity,
    /// The person the deed was done to, when there was one.
    pub patient: Option<Entity>,
    /// The deed's name, as defined in the defs file.
    pub deed: String,
    /// Whether the rumor reports what really happened.
    pub truth: bool,
}

/// Written when a person first hears a rumor by gossip (plan D5).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RumorLearned {
    /// The listener.
    pub person: Entity,
    /// The rumor they learned.
    pub rumor: RumorId,
}

/// Every rumor in the society, in creation order (plan D5). `RumorId(i)` addresses `rumors[i]`.
#[derive(Resource, Debug, Clone, Default, PartialEq, Reflect)]
pub struct RumorBook {
    /// The rumors, indexed by `RumorId`.
    pub rumors: Vec<Rumor>,
}

impl RumorBook {
    /// Adds `rumor` and returns its id. `known_by` is stored exactly as given: [`plant_rumor`] is
    /// what sets it to 1 and gives the origin a spreader entry, so a rumor added by hand must count
    /// its own knowers.
    pub fn add(&mut self, rumor: Rumor) -> RumorId {
        let id = RumorId(self.rumors.len() as u32);
        self.rumors.push(rumor);
        id
    }

    /// The rumor with `id`, if it exists.
    pub fn get(&self, id: RumorId) -> Option<&Rumor> {
        self.rumors.get(id.0 as usize)
    }

    /// Mutable access to the rumor with `id`, if it exists.
    pub fn get_mut(&mut self, id: RumorId) -> Option<&mut Rumor> {
        self.rumors.get_mut(id.0 as usize)
    }

    /// The share of `living_persons` who know rumor `id`: `known_by / living_persons` (plan D5).
    /// Returns 0 when there are no living persons or the rumor is unknown.
    pub fn reach(&self, id: RumorId, living_persons: usize) -> f32 {
        match (self.get(id), living_persons) {
            (Some(r), n) if n > 0 => r.known_by as f32 / n as f32,
            _ => 0.0,
        }
    }

    /// Marks every rumor about `subject` as no longer circulating, because the subject left the world.
    /// Returns the ids retired, in ascending order, so callers can drop them from `Knowledge`.
    pub fn retire_about(&mut self, subject: Entity) -> Vec<RumorId> {
        let mut retired = Vec::new();
        for (i, rumor) in self.rumors.iter_mut().enumerate() {
            if rumor.alive && rumor.subject == subject {
                rumor.alive = false;
                retired.push(RumorId(i as u32));
            }
        }
        retired
    }
}

/// Starts a rumor about `subject`'s deed and makes `origin` its first spreader at full belief (plan D5).
/// `harm` is the deed's harm and becomes the rumor's valence. Returns the new rumor's id. Callers resolve
/// the deed name and its harm through `SocietyDefs`.
#[expect(
    clippy::too_many_arguments,
    reason = "the rumor's fields are passed one by one so that callers cannot mix up the entities"
)]
pub(crate) fn plant_rumor(
    book: &mut RumorBook,
    origin_knowledge: &mut Knowledge,
    origin: Entity,
    subject: Entity,
    patient: Option<Entity>,
    deed: DeedId,
    harm: f32,
    now_days: f32,
    truth: bool,
) -> RumorId {
    let id = book.add(Rumor {
        deed,
        subject,
        patient,
        origin,
        day: now_days,
        truth,
        valence: harm,
        known_by: 1,
        alive: true,
    });
    origin_knowledge.insert(Known {
        rumor: id,
        belief: 1.0,
        spreader: true,
        heard_day: now_days,
    });
    id
}

/// What one telling did (plan D5, gossip).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gossip {
    /// The teller had no rumor to pass on.
    Nothing,
    /// The listener already knew it, so the teller stops spreading it (Daley & Kendall 1964).
    Stifled(RumorId),
    /// The listener learned it; the rumor's subject is carried for the `{subject}` bark placeholder.
    Learned {
        /// The rumor learned.
        rumor: RumorId,
        /// The person the rumor is about.
        subject: Entity,
    },
}

/// One gossip exchange over an accepted interaction (plan D5).
///
/// The teller picks, among its spreader rumors that are alive and not about the listener, the one
/// with the highest telling weight ([`Knowledge::best_telling`]). With none it tells nothing. When
/// the listener already knows the pick, the teller stifles for it. Otherwise the listener learns it
/// with belief `teller_belief · (0.5 + 0.5 · trust(listener → teller))`, spreading, the book's
/// `known_by` grows, and the listener's tie to the subject moves by `gossip_trust_gain · valence ·
/// belief` and `gossip_affinity_gain · valence · belief`. The listener records a
/// `HeardRumor(id)` memory and, when the subject is one of its close ties and the rumor is
/// damaging, feels `Distress` with intensity `|valence| · belief`.
///
/// Daley & Kendall 1964 (DOI 10.1038/2041118a0); Sommerfeld et al. 2007 (DOI 10.1073/pnas.0704598104);
/// Dunbar 2004 (DOI 10.1037/1089-2680.8.2.100) for gossip as reputation.
#[expect(
    clippy::too_many_arguments,
    reason = "one call per accepted gossip; the teller's and listener's state stay separate arguments"
)]
pub(crate) fn gossip(
    book: &mut RumorBook,
    teller_book: &Knowledge,
    listener_knowledge: &mut Knowledge,
    listener_ties: &mut Ties,
    listener_emotions: &mut crate::mind::Emotions,
    listener_memories: &mut crate::mind::Memories,
    listener_entity: Entity,
    listener_trust_in_teller: f32,
    listener_close_targets: &[Entity],
    settings: &crate::core::SocietySettings,
    now_days: f32,
) -> Gossip {
    let pick = teller_book.best_telling(now_days, |rumor| {
        let entry = book.get(rumor)?;
        if !entry.alive || entry.subject == listener_entity {
            return None;
        }
        Some(entry.valence)
    });
    let Some(rumor) = pick else {
        return Gossip::Nothing;
    };
    let Some(entry) = book.get(rumor) else {
        return Gossip::Nothing;
    };
    let (valence, subject, teller_belief) = match teller_book.get(rumor) {
        Some(known) => (entry.valence, entry.subject, known.belief),
        None => return Gossip::Nothing,
    };

    if listener_knowledge.knows(rumor) {
        // The teller stifles; it owns that `Knowledge`, so the caller writes it.
        return Gossip::Stifled(rumor);
    }

    let belief = belief_after_hearing(teller_belief, listener_trust_in_teller);
    listener_knowledge.insert(Known {
        rumor,
        belief,
        spreader: true,
        heard_day: now_days,
    });
    if let Some(entry) = book.get_mut(rumor) {
        entry.known_by += 1;
    }

    let tie = listener_ties.entry(subject);
    tie.trust = (tie.trust + settings.tuning.gossip_trust_gain * valence * belief).clamp(0.0, 1.0);
    tie.affinity =
        (tie.affinity + settings.tuning.gossip_affinity_gain * valence * belief).clamp(-1.0, 1.0);

    let importance = (valence.abs() * belief).clamp(0.0, 1.0);
    listener_memories.record(
        crate::mind::Memory {
            day: now_days,
            kind: crate::mind::MemoryKind::HeardRumor(rumor),
            other: Some(subject),
            valence: (valence * belief).clamp(-1.0, 1.0),
            importance,
        },
        f64::from(now_days),
        settings.memory_capacity,
    );

    if valence < 0.0 && listener_close_targets.contains(&subject) {
        listener_emotions.appraise(
            crate::mind::EmotionKind::Distress,
            importance,
            Some(subject),
        );
    }

    Gossip::Learned { rumor, subject }
}

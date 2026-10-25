//! Applying one queued [`ScriptAction`] to the world (plan D8).
//!
//! Split out of `core.rs` so the exclusive system there stays a loop over the queue and every rule
//! about what an `fx` call may write sits in one file. Called only by
//! [`crate::core::apply_script_actions`], once per tick, with the whole world.

use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::log::{error, warn};

use crate::beliefs::{Knowledge, Opinions};
use crate::core::{PersonId, PersonIndex, SocietySettings};
use crate::defs::SocietyDefs as Defs;
use crate::groups::Memberships;
use crate::mind::{EmotionKind, Emotions, Memories, Memory, MemoryKind, Needs};
use crate::relationships::Ties;
use crate::rumor::{RumorBook, plant_rumor};
use crate::script::{ScriptAction, ScriptEvent};

/// Applies one action. Every id is resolved here; an unresolvable one is a `warn!` naming the id and
/// the action is skipped, so a script that invents an id cannot silently write to the wrong person.
pub(crate) fn apply(world: &mut World, action: ScriptAction, now_days: f64) {
    match action {
        ScriptAction::Adjust {
            person,
            other,
            field,
            delta,
        } => {
            let (Some(person), Some(other)) = (entity(world, person), entity(world, other)) else {
                return;
            };
            let tuning = world
                .get_resource::<SocietySettings>()
                .map_or_else(Default::default, |settings| settings.tuning.clone());
            let Some(mut ties) = world.get_mut::<Ties>(person) else {
                return;
            };
            let tie = ties.entry(other);
            match field.as_str() {
                "affinity" => tie.add_affinity(delta, &tuning),
                "trust" => tie.trust = (tie.trust + delta).clamp(0.0, 1.0),
                "respect" => tie.respect = (tie.respect + delta).clamp(-1.0, 1.0),
                "familiarity" => {
                    // Familiarity is stored as a peak read through power-law forgetting, so a delta
                    // moves the peak and restamps the contact: `familiarity(now)` then reads at least
                    // the clamped peak.
                    let now = now_days as f32;
                    let current = tie.familiarity(now, &tuning);
                    tie.familiarity_peak = (current + delta).clamp(0.0, 1.0);
                    tie.last_contact_day = now;
                }
                other_field => warn!(
                    "bevy_society: fx.adjust names unknown tie field '{other_field}'; ignored"
                ),
            }
        }
        ScriptAction::Emote {
            person,
            kind,
            intensity,
        } => {
            let Some(person) = entity(world, person) else {
                return;
            };
            let Some(kind) = EmotionKind::from_name(&kind) else {
                warn!("bevy_society: fx.emote names unknown emotion '{kind}'; ignored");
                return;
            };
            let Some(mut emotions) = world.get_mut::<Emotions>(person) else {
                return;
            };
            emotions.appraise(kind, intensity.clamp(0.0, 1.0), None);
        }
        ScriptAction::Need {
            person,
            need,
            delta,
        } => {
            let (Some(person), Some(defs)) = (entity(world, person), world.get_resource::<Defs>())
            else {
                return;
            };
            let Some(id) = defs.need_id(&need) else {
                warn!("bevy_society: fx.need names unknown need '{need}'; ignored");
                return;
            };
            let Some(mut needs) = world.get_mut::<Needs>(person) else {
                return;
            };
            if let Some(slot) = needs.0.get_mut(usize::from(id.0)) {
                *slot = (*slot + delta).clamp(0.0, 1.0);
            }
        }
        ScriptAction::Opinion {
            person,
            topic,
            delta,
        } => {
            let (Some(person), Some(defs)) = (entity(world, person), world.get_resource::<Defs>())
            else {
                return;
            };
            let Some(id) = defs.topic_id(&topic) else {
                warn!("bevy_society: fx.opinion names unknown topic '{topic}'; ignored");
                return;
            };
            let Some(mut opinions) = world.get_mut::<Opinions>(person) else {
                return;
            };
            if let Some(opinion) = opinions.get_mut(usize::from(id.0)) {
                opinion.value = (opinion.value + delta).clamp(-1.0, 1.0);
            }
        }
        ScriptAction::Join { person, faction } => join_or_leave(world, person, faction, true),
        ScriptAction::Leave { person, faction } => join_or_leave(world, person, faction, false),
        ScriptAction::Rumor {
            origin,
            subject,
            deed,
            truth,
        } => plant(world, origin, subject, &deed, truth, now_days),
        ScriptAction::Interact {
            actor,
            target,
            interaction,
        } => {
            let (Some(actor), Some(target)) = (entity(world, actor), entity(world, target)) else {
                return;
            };
            world.write_message(crate::interact::Interact {
                actor,
                target,
                interaction,
            });
        }
        ScriptAction::Bark { speaker, text } => bark(world, speaker, text, false),
        ScriptAction::Think { speaker, text } => bark(world, speaker, text, true),
        ScriptAction::Remember {
            person,
            other,
            text,
            valence,
            importance,
        } => {
            let Some(person) = entity(world, person) else {
                return;
            };
            let other = if other < 0 {
                None
            } else {
                entity(world, other)
            };
            let capacity = world
                .get_resource::<SocietySettings>()
                .map_or(64, |settings| settings.memory_capacity);
            let Some(mut memories) = world.get_mut::<Memories>(person) else {
                return;
            };
            memories.record(
                Memory {
                    day: now_days as f32,
                    kind: MemoryKind::Custom(text),
                    other,
                    valence: valence.clamp(-1.0, 1.0),
                    importance: importance.clamp(0.0, 1.0),
                },
                now_days,
                capacity,
            );
        }
        // `fx.converse` queues this action; `dialogue` turns it into a `TalkTo` for next frame.
        ScriptAction::Converse { npc, player } => converse(world, npc, player),
        ScriptAction::Emit {
            name,
            actor,
            target,
            data,
        } => {
            let actor = if actor < 0 {
                None
            } else {
                entity(world, actor)
            };
            let target = if target < 0 {
                None
            } else {
                entity(world, target)
            };
            world.write_message(ScriptEvent {
                name,
                actor,
                target,
                data,
            });
        }
    }
}

/// Adds or removes one faction membership, naming an unknown faction in a `warn!`.
fn join_or_leave(world: &mut World, person: i64, faction: String, join: bool) {
    let (Some(person), Some(defs)) = (entity(world, person), world.get_resource::<Defs>()) else {
        return;
    };
    let Some(id) = defs.faction_id(&faction) else {
        warn!("bevy_society: an fx membership names unknown faction '{faction}'; ignored");
        return;
    };
    let Some(mut memberships) = world.get_mut::<Memberships>(person) else {
        return;
    };
    if join {
        memberships.insert(id);
    } else {
        memberships.remove(id);
    }
}

/// The entity of a live person, or `None` with a `warn!` when the id names nobody.
fn entity(world: &World, id: i64) -> Option<Entity> {
    let index = world.get_resource::<PersonIndex>()?;
    let Ok(id) = u32::try_from(id) else {
        warn!("bevy_society: a script named person id {id}, which is not a person; ignored");
        return None;
    };
    match index.by_id.get(&PersonId(id)) {
        Some(&entity) => Some(entity),
        None => {
            warn!("bevy_society: a script named person id {id}, which is not a person; ignored");
            None
        }
    }
}

/// A script's `fx.converse`: the conversation is started by `DialoguePlugin` on the next frame.
#[cfg(feature = "dialogue")]
fn converse(world: &mut World, npc: i64, player: i64) {
    match (entity(world, npc), entity(world, player)) {
        (Some(npc), Some(player)) => {
            world.write_message(crate::dialogue::TalkTo { npc, player });
        }
        _ => {
            warn!(
                "bevy_society: a script asked for a conversation with an unknown person ({npc}, {player}); ignored"
            );
        }
    }
}

/// Without the `dialogue` feature there is no conversation to start, so the action is dropped.
#[cfg(not(feature = "dialogue"))]
fn converse(_world: &mut World, npc: i64, player: i64) {
    warn!(
        "bevy_society: a script asked for a conversation between {npc} and {player}, but the dialogue feature is not built in"
    );
}

/// Starts a rumor through the same function [`crate::PlantRumor`] uses, with the deed's own harm.
fn plant(world: &mut World, origin: i64, subject: i64, deed: &str, truth: bool, now_days: f64) {
    let (Some(origin), Some(subject)) = (entity(world, origin), entity(world, subject)) else {
        return;
    };
    let Some(defs) = world.get_resource::<Defs>() else {
        return;
    };
    let Some(id) = defs.deed_id(deed) else {
        error!("bevy_society: fx.rumor names unknown deed '{deed}'; ignored");
        return;
    };
    let Some(harm) = defs.deeds.get(usize::from(id.0)).map(|spec| spec.harm) else {
        return;
    };
    let mut book = world.remove_resource::<RumorBook>().unwrap_or_default();
    // Read the cap before taking the mutable borrow of `knowledge`.
    let knowledge_capacity = world
        .get_resource::<crate::core::SocietySettings>()
        .map_or(64, |s| s.knowledge_capacity);
    let Some(mut knowledge) = world.get_mut::<Knowledge>(origin) else {
        world.insert_resource(book);
        return;
    };
    plant_rumor(
        &mut book,
        &mut knowledge,
        origin,
        subject,
        None,
        id,
        harm,
        now_days as f32,
        truth,
        knowledge_capacity,
    );
    world.insert_resource(book);
}

/// Writes a [`crate::SocietyBark`] with the speaker's feeling.
fn bark(world: &mut World, speaker: i64, text: String, thought: bool) {
    let Some(speaker) = entity(world, speaker) else {
        return;
    };
    let feeling = world.get::<crate::mind::Emotions>(speaker).map_or(
        crate::interact::Feeling::Neutral,
        crate::interact::Feeling::from_emotions,
    );
    world.write_message(crate::interact::SocietyBark {
        speaker,
        text,
        thought,
        feeling,
    });
}

//! # bevy_society
//!
//! Living societies for Bevy: personalities, moods, needs, memories, relationships, opinions,
//! rumors, reputation and factions; utility-AI daily lives; Rhai-scripted triggers; and dialogue
//! and barks through `bevy_speech_bubbles`.
//!
//! Every model is distilled from social-psychology and game-AI literature (see `docs/research/`)
//! and every behaviour is deterministic for a given seed.

pub mod model;

mod behavior;
#[cfg(feature = "steering")]
mod steering;

mod beliefs;
mod core;
mod defs;
mod groups;
mod mind;
mod plugins;
mod relationships;
mod rumor;
mod script;

pub use behavior::{Destination, Doing, DoingTarget};
pub use beliefs::{BeliefRates, Knowledge, Known, Opinion, Opinions, belief_after_hearing};
pub use core::{
    AssessmentNorm, Autonomous, Home, NextPersonId, Perception, Person, PersonId, PersonIndex,
    Place, PlaceTags, SocietyClock, SocietyCorePlugin, SocietyPlane, SocietyRng, SocietySettings,
    SocietySystems, SocietyTick, Tuning, society_tick_due,
};
pub use defs::{
    AcceptDef, ActivityDef, ActivityId, ActivityTarget, Consideration, ConversationRuleDef,
    DeedDef, DeedId, DefsError, DefsFile, EffectsDef, FactionDef, FactionId, FieldInput,
    IDLE_ACTIVITY, Input, InteractionBarks, InteractionDef, InteractionId, NeedDef, NeedId,
    NodeEffectDef, SocietyDefs, TieDelta, TopicDef, TopicId, TriggerDef, TriggerOn,
    size_person_state,
};
pub use groups::{
    CONTEST_STEEPNESS, FactionStat, FactionStats, Memberships, STANDING_FAMILIARITY, Status,
    contest_win_chance, standing_from,
};
pub use mind::{
    ActiveEmotion, EmotionKind, Emotions, Memories, Memory, MemoryKind, Mood, MoodRates, Needs,
    Personality, Trait,
};
pub use plugins::SocietyPlugins;
pub use relationships::{Tie, TieLabel, TieRates, Ties};
pub use rumor::{Rumor, RumorBook, RumorId};
pub use script::ScriptId;
#[cfg(feature = "steering")]
pub use steering::{SocialMover, SocialVelocity, SteeringPlugin};

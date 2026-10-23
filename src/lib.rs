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
#[cfg(feature = "fields")]
mod fields;
mod groups;
mod interact;
mod mind;
mod plugins;
mod relationships;
mod reputation;
mod rules;
mod rumor;
mod script;
mod script_apply;
mod snapshot;

pub use behavior::{Destination, Doing, DoingTarget};
pub use beliefs::{BeliefsPlugin, Knowledge, Known, Opinion, Opinions, belief_after_hearing};
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
#[cfg(feature = "fields")]
pub use fields::{FieldChannel, SocialFieldPlugin, SocialFieldSettings, SocialFields};
pub use groups::{
    FactionStat, FactionStats, GroupsPlugin, Memberships, STANDING_FAMILIARITY, Status,
    contest_win_chance, standing_from,
};
pub use interact::{Feeling, Interact, SocietyBark};
pub use mind::{
    ActiveEmotion, Appraise, EmotionKind, Emotions, Memories, Memory, MemoryKind, Mood, MoodRates,
    Needs, Personality, Trait,
};
pub use plugins::SocietyPlugins;
pub use relationships::{RelationshipsPlugin, Tie, TieLabel, TieLabelChanged, Ties};
pub use reputation::NormViolated;
pub use rules::{FireTrigger, RulesPlugin};
pub use rumor::{PlantRumor, Rumor, RumorBook, RumorId, RumorLearned};
pub use script::{ScriptEngine, ScriptError, ScriptEvent, ScriptId, ScriptValue};
#[cfg(feature = "steering")]
pub use steering::{SocialMover, SocialVelocity, SteeringPlugin};

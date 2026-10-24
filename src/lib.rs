//! # bevy_society
//!
//! Living societies for Bevy: personalities, moods, needs, memories, relationships, opinions,
//! rumors, reputation and factions; utility-AI daily lives; Rhai-scripted triggers; and dialogue
//! and barks through `bevy_speech_bubbles`.
//!
//! Every model is distilled from social-psychology and game-AI literature (see `docs/research/`)
//! and every behaviour is deterministic for a given seed.
//!
//! ## Quick start
//!
//! Spawn people with [`Person`], give them somewhere to go with [`Place`], hand in the definitions
//! and read what happens. The example needs no renderer; with the default `dialogue` feature it
//! also needs `SpeechBubblesPlugin` (see the note at the end of the example):
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_society::{
//!     Autonomous, Home, Person, Place, PlaceTags, SocietyDefs, SocietyPlugins, SocietySettings,
//! };
//!
//! fn main() {
//!     let defs = SocietyDefs::starter().expect("the starter pack is valid");
//!     let mut app = App::new();
//!     app.add_plugins(MinimalPlugins)
//!         .insert_resource(SocietySettings::default())
//!         .insert_resource(defs)
//!         .add_plugins(SocietyPlugins);
//!
//!     let well = app
//!         .world_mut()
//!         .spawn((Name::new("well"), Place { capacity: 4 }, PlaceTags(vec!["gathering".into()])))
//!         .id();
//!     for name in ["Bram", "Alys"] {
//!         app.world_mut().spawn((
//!             Person,
//!             Autonomous,
//!             Name::new(name),
//!             Home(well),
//!             Transform::default(),
//!         ));
//!     }
//!
//!     for _ in 0..600 {
//!         app.update();
//!     }
//!     // With the default `dialogue` feature the host must also add
//!     // `bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera>`, or `finish()` panics.
//!     // `no_run`: a headless `App::update` never calls `finish()`, so this only compiles.
//! }
//! ```
//!
//! ## Plugins
//!
//! [`SocietyPlugins`] installs the whole society. Each member also works alone after
//! [`SocietyCorePlugin`]: the mind plugin for emotions, mood, needs and memories, the relationships
//! plugin for ties, [`BeliefsPlugin`] for opinions and rumors, [`GroupsPlugin`] for factions and
//! status, the behaviour plugin for daily life and social exchanges, [`RulesPlugin`] for Rhai
//! triggers, then the feature-gated social-field, steering and dialogue plugins.
//!
//! ## Ticks, snapshots and scripts
//!
//! The society advances on society ticks: one every [`SocietySettings::tick_secs`] of fixed-step
//! time, with every rate expressed per game-day. The systems run in the order of [`SocietySystems`]
//! — `Clock`, `Perceive`, `Mind`, `Decide`, `Interact`, `Propagate`, `Relate`, `Rules` — and only
//! `Clock` runs on every fixed step.
//!
//! Scripts see the society as it stood at the start of the tick, through a snapshot built in
//! `Perceive`; a script's own writes go to a queue. Those writes are applied once per tick, after
//! `Rules`, by one exclusive system, so no script can change what another script in the same tick
//! reads. Game-written input messages ([`PlantRumor`], [`Interact`], [`FireTrigger`]) are drained
//! on every fixed step, not only on society ticks, so a message written between ticks is never
//! dropped.
//!
//! ## Guides
//!
//! `docs/guide/getting-started.md`, `concepts.md`, `authoring.md`, `scripting.md` and `dialogue.md`;
//! `docs/research/` for the papers behind every formula; `docs/DEMOS.md` for the capture and GIF.

pub mod model;

mod behavior;
#[cfg(feature = "dialogue")]
mod dialogue;
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

pub use behavior::{
    ActivityEnded, ActivityStarted, BehaviorPlugin, Destination, Doing, DoingTarget,
    InteractionResolved,
};
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
    NodeEffectDef, Signal, SocietyDefs, TieDelta, TopicDef, TopicId, TriggerDef, TriggerOn,
    size_person_state,
};
#[cfg(feature = "dialogue")]
pub use dialogue::{DialoguePlugin, TalkTo};
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

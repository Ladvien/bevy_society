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
//! ```
//! use bevy::prelude::*;
//! use bevy_society::{
//!     Autonomous, Home, Person, Place, PlaceTags, SocietyClock, SocietyDefs, SocietyPlugins,
//!     SocietySettings,
//! };
//!
//! let defs = SocietyDefs::starter().expect("the starter pack is valid");
//! // A headless app has no frame clock, so `Time<Fixed>` never advances on its own. Without this the
//! // society takes zero ticks however many updates you run.
//! let step = std::time::Duration::from_secs_f64(1.0 / 64.0);
//!
//! let mut app = App::new();
//! app.add_plugins((MinimalPlugins, TransformPlugin))
//!     .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(step))
//!     .insert_resource(SocietySettings::default())
//!     .insert_resource(defs);
//! // `DialoguePlugin` needs a speech-bubbles plugin over a camera, which this snippet has not.
//! let plugins = SocietyPlugins.build();
//! #[cfg(feature = "dialogue")]
//! let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
//! app.add_plugins(plugins);
//!
//! let well = app
//!     .world_mut()
//!     .spawn((Name::new("well"), Place { capacity: 6 }, PlaceTags(vec!["gathering".into()]), Transform::default()))
//!     .id();
//! for name in ["Bram", "Alys", "Corin"] {
//!     app.world_mut()
//!         .spawn((Person, Autonomous, Name::new(name), Home(well), Transform::default()));
//! }
//!
//! for _ in 0..600 {
//!     app.update();
//! }
//! // The society advanced: this line is what proves the fixed step was pinned.
//! assert!(app.world().resource::<SocietyClock>().tick > 0);
//! ```
//!
//! The same code is `examples/quickstart.rs`, and `README.md` carries it too — `tests/readme.rs`
//! checks the README's text so the three cannot drift.
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

mod absence;
mod behavior;
#[cfg(feature = "dialogue")]
mod dialogue;
#[cfg(feature = "steering")]
mod steering;

mod beliefs;
mod core;
mod defs;
mod economy;
mod fast_forward;
#[cfg(feature = "fields")]
mod fields;
mod groups;
mod interact;
mod mind;
mod placement;
mod plugins;
#[cfg(feature = "serde")]
mod preview;
mod relationships;
mod reputation;
mod rules;
mod rumor;
#[cfg(feature = "serde")]
mod save;
mod script;
mod script_apply;
mod snapshot;
mod trace;

pub use absence::PersonReturned;
pub use behavior::{
    ActivityEnded, ActivityStarted, BehaviorPlugin, Destination, Doing, DoingTarget, EndActivity,
    InteractionResolved, PersonDied, ReportDeed, TravelFailed,
};
pub use beliefs::{BeliefsPlugin, Knowledge, Known, Opinion, Opinions, belief_after_hearing};
pub use core::{
    AssessmentNorm, Autonomous, Away, AwaySince, BoundPlaces, ClockSource, Deceased, Dormant, Home,
    HostClock, Locomotion, NextPersonId, Perception, Person, PersonId, PersonIndex, PersonKey,
    Place, PlaceKey, PlaceTags, SocietyClock, SocietyCorePlugin, SocietyPlane, SocietyRng,
    SocietySettings, SocietySystems, SocietyTick, Tuning, skill_level, society_tick_due,
};
pub use defs::{
    AcceptDef, ActivityDef, ActivityId, ActivityTarget, Consideration, ConversationRuleDef,
    DeedDef, DeedId, DefsError, DefsFile, EffectsDef, FactionDef, FactionId, FieldInput, GoodDef,
    GoodId, IDLE_ACTIVITY, Input, InteractionBarks, InteractionDef, InteractionId, NeedDef, NeedId,
    NodeEffectDef, Signal, SignalDef, SignalId, SkillDef, SkillGroup, SkillId, SocietyDefs,
    TieDelta, TopicDef, TopicId, TriggerArea, TriggerDef, TriggerOn, UrgentDef, size_person_state,
};
#[cfg(feature = "dialogue")]
pub use dialogue::{
    DialoguePlugin, DialogueProblem, DialogueReferenceKind, TalkTo, dialogue_problems,
};
pub use economy::{EconomyPlugin, Prices, Stock, value_of};
#[cfg(feature = "fields")]
pub use fields::{Commotion, SocialFieldPlugin, SocialFieldSettings, SocialFields};
// The two value types the fields are configured with, re-exported so a host names one crate. Both are
// `bevy_stigmergy` types now: `ChannelDef` because the crate's own mirror type had no reason to exist
// once `bevy_stigmergy` 0.2 added `Reflect`, and `CellFrame` because it is the shared world↔cell
// mapping this crate and Sanctuary both used to carry their own copy of.
#[cfg(feature = "fields")]
pub use bevy_stigmergy::{CellFrame, ChannelDef};
pub use fast_forward::{FastForward, FastForwardError, fast_forward};
pub use groups::{
    FactionStat, FactionStats, GroupsPlugin, Memberships, STANDING_FAMILIARITY, Status,
    contest_win_chance, standing_from,
};
pub use interact::{Feeling, Interact, SocietyBark};
pub use mind::{
    ActiveEmotion, Appraise, Chronotype, EmotionKind, Emotions, Memories, Memory, MemoryKind, Mood,
    MoodRates, Needs, Personality, Signals, Trait,
};
pub use placement::{
    PersonDef, PersonalityDef, PlaceDef, PlaceDependent, Placed, PlacementFile, TieSpec,
    persons_in_rect, place_dependents,
};
pub use plugins::SocietyPlugins;
#[cfg(feature = "serde")]
pub use preview::{PersonPreview, Preview, PreviewError, TieChange, preview_forward};
pub use relationships::{RelationshipsPlugin, Tie, TieLabel, TieLabelChanged, Ties};
pub use reputation::NormViolated;
pub use rules::{
    FireTrigger, RulesPlugin, TriggerFired, TriggerTest, TriggerTestError, reload_defs,
    test_trigger,
};
pub use rumor::{PlantRumor, Rumor, RumorBook, RumorId, RumorLearned};
#[cfg(feature = "serde")]
pub use save::{
    ClockSave, DoingSave, DoingTargetSave, EmotionSave, FieldsSave, MemoryKindSave, MemorySave,
    NamesSave, PersonSave, RestoreError, RestoreReport, RngSave, RumorSave, SocietySave, TieSave,
    TriggersSave, VERSION, capture, restore,
};
pub use script::{
    ScriptEngine, ScriptError, ScriptEvent, ScriptId, ScriptKind, ScriptSyntaxError, ScriptValue,
    check_script,
};
#[cfg(feature = "steering")]
pub use steering::{Obstacle, SocialMover, SocialVelocity, SteeringPlugin};
pub use trace::{Decision, DecisionTrace, ScoredOption};

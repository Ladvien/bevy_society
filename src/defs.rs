//! Authored definitions: the data a host hands in (needs, signals, topics, deeds, factions,
//! activities, interactions, triggers, conversation rules and node effects). `DefsFile` is the
//! RON-shaped input; `SocietyDefs` is the compiled resource the simulation reads, with ids and Rhai
//! scripts resolved. The crate never reads a file: the host passes in text it has already loaded.

use std::collections::{HashMap, HashSet};
use std::fmt;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::log::error;
use bevy::reflect::Reflect;
use serde::Deserialize;

use crate::beliefs::{Opinion, Opinions};
use crate::core::Person;
use crate::groups::Memberships;
use crate::mind::{EmotionKind, Needs, Signals, Trait};
use crate::model::curves::Curve;
use crate::relationships::TieLabel;
use crate::script::{CompiledScript, ScriptCompiler, ScriptId};

/// Whether the `fields` feature is on. Field inputs and gradient targets need it.
const FIELDS_ENABLED: bool = cfg!(feature = "fields");

/// Name of the reserved activity every compiled `SocietyDefs` contains. Persons with no valid
/// option pick it for one game-hour.
pub const IDLE_ACTIVITY: ActivityId = ActivityId(0);

const IDLE: &str = "idle";

/// Index of a need, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NeedId(pub u16);

/// Index of a host signal, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
pub struct SignalId(pub u16);

/// Index of a good, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GoodId(pub u16);

/// Index of a skill, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkillId(pub u16);

/// Index of a topic, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TopicId(pub u16);

/// Index of a deed, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeedId(pub u16);

/// Index of a faction, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FactionId(pub u16);

/// Index of an activity, in definition order. `IDLE_ACTIVITY` is always `ActivityId(0)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityId(pub u16);

/// Index of an interaction, in definition order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InteractionId(pub u16);

/// Everything a host hands in, as one RON document. Every list is optional.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct DefsFile {
    /// Needs persons carry; they decay per game-day.
    pub needs: Vec<NeedDef>,
    /// Signals the host writes and the crate only reads.
    pub signals: Vec<SignalDef>,
    /// Named capabilities, each read from one personality trait.
    pub skills: Vec<SkillDef>,
    /// Tradeable quantities persons hold.
    pub goods: Vec<GoodDef>,
    /// Opinion topics.
    pub topics: Vec<TopicDef>,
    /// Deeds that witnesses judge and that can become rumors.
    pub deeds: Vec<DeedDef>,
    /// Factions (groups persons belong to).
    pub factions: Vec<FactionDef>,
    /// Things persons do.
    pub activities: Vec<ActivityDef>,
    /// Social exchanges between two persons.
    pub interactions: Vec<InteractionDef>,
    /// Rhai rules that fire on simulation events.
    pub triggers: Vec<TriggerDef>,
    /// Conversation selection rules for `TalkTo`.
    pub conversations: Vec<ConversationRuleDef>,
    /// Scripts run when a dialogue node is entered.
    pub node_effects: Vec<NodeEffectDef>,
}

/// A need. Level 1 is satisfied, 0 is desperate.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeedDef {
    /// Unique name.
    pub id: String,
    /// Loss per game-day, 0 or more.
    #[serde(default)]
    pub decay_per_day: f32,
    /// Starting level.
    #[serde(default = "one")]
    pub initial: f32,
}

/// A host signal. The crate never changes its value; the host sets it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalDef {
    /// Unique name.
    pub id: String,
}

/// Which personality axis a skill draws on.
///
/// **A design default, measured.** Acemoglu & Autor (2010, DOI `10.3386/w16082`) separate a **task** ("a
/// unit of work activity that produces output") from a **skill** ("a worker's endowment of capabilities for
/// performing various tasks"), and that is the distinction this file needs: an activity is the task, and
/// this is the endowment. The three-way grouping of Big Five traits onto kinds of work is the crate's own
/// choice — no paper in the corpus assigns traits to trades — and each group reads exactly one trait so
/// that the mapping is inspectable in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SkillGroup {
    /// Diligence: scales with conscientiousness.
    Craft,
    /// Invention: scales with openness.
    Creative,
    /// Dealing with people: scales with extraversion.
    Social,
}

impl SkillGroup {
    /// The trait this group reads, as an index into [`Personality::values`](crate::Personality).
    pub fn trait_of(self) -> crate::defs::Trait {
        match self {
            Self::Craft => Trait::Conscientiousness,
            Self::Creative => Trait::Openness,
            Self::Social => Trait::Extraversion,
        }
    }
}

/// A tradeable quantity. Not a currency: there is no exchange rate and no `Good` is worth another.
///
/// "Meat and honey are meticulously shared among the Ache, but goods purchased by money are not" —
/// Henrich et al. 2005 (DOI `10.1017/S0140525X05000142`). A good is a level a person holds, nothing more.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodDef {
    /// Unique name.
    pub id: String,
}

/// A named capability. Level comes from personality, not from stored state.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillDef {
    /// Unique name.
    pub id: String,
    /// Which trait the level is read from.
    pub group: SkillGroup,
}

/// An opinion topic.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicDef {
    /// Unique name.
    pub id: String,
}

/// A deed a person can do to another. Witnesses judge it by their opinion of `topic`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeedDef {
    /// Unique name.
    pub id: String,
    /// The topic a witness's opinion judges the deed by.
    pub topic: String,
    /// Harm in [-1, 1]; positive helps the patient.
    pub harm: f32,
}

/// A faction.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionDef {
    /// Unique name.
    pub id: String,
    /// Extra affinity weight for members toward co-members (Tajfel et al. 1971, DOI 10.1002/ejsp.2420010202).
    #[serde(default = "default_ingroup_bias")]
    pub ingroup_bias: f32,
}

/// Something a person can do, chosen by utility scoring.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivityDef {
    /// Unique name.
    pub id: String,
    /// Where the activity happens.
    pub target: ActivityTarget,
    /// How long the activity lasts once arrived, in game-hours.
    pub duration_hours: f32,
    /// Dual-utility rank; the highest rank with any option above zero wins (Dill 2015).
    #[serde(default)]
    pub rank: u8,
    /// Multiplies the option's score.
    #[serde(default = "one")]
    pub weight: f32,
    /// Considerations whose product is the option's score.
    #[serde(default)]
    pub considerations: Vec<Consideration>,
    /// Needs this activity raises per game-hour while arrived.
    #[serde(default)]
    pub satisfies: Vec<(String, f32)>,
    /// Interaction performed on arrival; requires a `Person` target.
    pub interaction: Option<String>,
    /// Goods this activity makes, **per completed use**, with the amount.
    ///
    /// Per use, not per game-hour: a stock is a quantity, while `satisfies` is the rate-shaped field. A
    /// task that is interrupted produces nothing, which is the same rule `satisfies` follows.
    #[serde(default)]
    pub produces: Vec<(String, f32)>,
    /// Goods this activity uses up, per completed use. A person without them still does the activity —
    /// a good changes what someone prefers, never what they may do.
    #[serde(default)]
    pub consumes: Vec<(String, f32)>,
    /// The skill this activity is a **task** for, when it is one.
    ///
    /// A task is "a unit of work activity that produces output" and a skill is "a worker's endowment of
    /// capabilities for performing various tasks" — Acemoglu & Autor 2010 (DOI `10.3386/w16082`). Naming a
    /// skill here does **not** gate the activity: an unskilled person can still choose it, at a lower score
    /// if a consideration reads `Skill(..)`. That is comparative advantage rather than assignment, which is
    /// the paper's own point ("the assignment of skills to tasks is endogenous").
    #[serde(default)]
    pub profession: Option<String>,
    /// Thought barks while doing the activity.
    #[serde(default)]
    pub barks: Vec<String>,
}

/// Where an activity happens.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum ActivityTarget {
    /// In place, for no location.
    Itself,
    /// A random point within the radius of the person.
    Wander(f32),
    /// Any place carrying this tag.
    Place(String),
    /// The person's home.
    Home,
    /// The place this person has bound under this name, from
    /// [`BoundPlaces`](crate::BoundPlaces). A person with no such binding has no candidate: they do not
    /// do the activity.
    Bound(String),
    /// Another person nearby.
    Person,
    /// Toward (or away from, when false) a field's gradient. Needs the `fields` feature.
    Gradient(FieldInput, bool),
}

/// One scoring term: an input mapped through a curve.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consideration {
    /// The signal read for the option.
    pub input: Input,
    /// Maps the input to [0, 1].
    pub curve: Curve,
}

/// Signals a consideration can read.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Input {
    /// The actor's level of a need.
    Need(String),
    /// A value the host wrote for the actor.
    Signal(String),
    /// The clock hour divided by 24.
    Hour,
    /// The actor's mood pleasure, rescaled to [0, 1].
    Pleasure,
    /// The actor's mood arousal, rescaled to [0, 1].
    Arousal,
    /// The actor's mood dominance, rescaled to [0, 1].
    Dominance,
    /// A personality trait of the actor, rescaled to [0, 1].
    Trait(Trait),
    /// The actor's strongest current intensity of an emotion.
    Emotion(EmotionKind),
    /// Affinity toward the target, rescaled to [0, 1]. Person targets only.
    Affinity,
    /// Trust toward the target. Person targets only.
    Trust,
    /// Respect toward the target, rescaled to [0, 1]. Person targets only.
    Respect,
    /// Familiarity with the target. Person targets only.
    Familiarity,
    /// Distance to the target divided by the perception radius. Place, Home and Person targets.
    Distance,
    /// Fraction of the actor's close ties doing this activity. Person targets only.
    SocialProof,
    /// A named skill's level, in `[0, 1]`. See [`SkillDef`].
    Skill(String),
    /// The **joint surplus** a trade in this good would produce, in `[0, 1]`.
    ///
    /// `value(target's store) − value(actor's store)` for the good named, where `value(held) = 1/(1+held)`.
    /// This is Hall & Milgrom's precondition as a consideration: "once ... they have met and found that they
    /// would enjoy a **joint surplus**, the threats to disclaim the match are hollow" (Hall 2005, DOI
    /// `10.3386/w11678`). Near zero when both stores are the same, and largest when one is empty.
    ///
    /// Zero unless the actor holds **more than** [`Tuning::trade_reserve`](crate::Tuning) of the good (it has
    /// something to spare) **and** the gap is at least [`Tuning::trade_margin`](crate::Tuning) (it is worth
    /// the walk). The input itself is a measurement of two stores, not a rate. It needs a `Person` target,
    /// and a target holding no goods at all is not a counterparty.
    Surplus(String),
    /// A social field value at the target cell. Needs the `fields` feature.
    Field(FieldInput),
    /// A Rhai float expression, clamped to [0, 1].
    Script(String),
}

/// Social fields a consideration or gradient target can read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum FieldInput {
    /// Presence of everyone at the cell, relative to its peak.
    Crowd,
    /// Recent deeds at the cell, relative to its peak.
    Commotion,
    /// The actor's faction's presence at the cell, relative to its peak.
    OwnFaction,
    /// Presence of factions the actor is not in, relative to its peak.
    OtherFactions,
    /// The actor's faction's share of presence at the cell (Schelling 1971, DOI 10.1080/0022250X.1971.9989794).
    OwnShare,
}

/// A social exchange between two persons.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InteractionDef {
    /// Unique name.
    pub id: String,
    /// Rhai boolean; the exchange is refused by rule when false.
    pub requires: Option<String>,
    /// Acceptance probability terms.
    #[serde(default)]
    pub accept: AcceptDef,
    /// Effects when the target accepts.
    #[serde(default)]
    pub on_accept: EffectsDef,
    /// Effects when the target refuses.
    #[serde(default)]
    pub on_reject: EffectsDef,
    /// Topics that influence both persons' opinions when the exchange is accepted.
    #[serde(default)]
    pub influence: Vec<String>,
    /// Whether an accepted exchange passes on a rumor.
    #[serde(default)]
    pub gossip: bool,
    /// Deed the exchange performs on the target when accepted.
    pub deed: Option<String>,
    /// Whether the exchange is a contest that sets a winner.
    #[serde(default)]
    pub contest: bool,
    /// Bark lines.
    #[serde(default)]
    pub barks: InteractionBarks,
}

/// Terms of the acceptance probability (Deffuant et al. 2000 and Byrne 1961 inform the defaults;
/// the plan records the numbers).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AcceptDef {
    /// Base probability.
    pub base: f32,
    /// Weight of affinity toward the actor.
    pub affinity: f32,
    /// Weight of the target's trust in the actor.
    pub trust: f32,
    /// Weight of the target's agreeableness.
    pub agreeableness: f32,
    /// Weight of the target's mood pleasure.
    pub pleasure: f32,
    /// Bonus when both share a faction.
    pub ingroup: f32,
}

impl Default for AcceptDef {
    fn default() -> Self {
        Self {
            base: 0.5,
            affinity: 0.3,
            trust: 0.2,
            agreeableness: 0.2,
            pleasure: 0.1,
            ingroup: 0.1,
        }
    }
}

/// Tie changes applied by an interaction outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TieDelta {
    /// Change in affinity.
    pub affinity: f32,
    /// Change in trust.
    pub trust: f32,
    /// Change in respect.
    pub respect: f32,
    /// Change in familiarity.
    pub familiarity: f32,
}

/// The effects of one interaction outcome.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct EffectsDef {
    /// Tie change from actor toward target.
    pub actor_to_target: TieDelta,
    /// Tie change from target toward actor.
    pub target_to_actor: TieDelta,
    /// Emotion the actor feels, with intensity.
    pub actor_emotion: Option<(EmotionKind, f32)>,
    /// Emotion the target feels, with intensity.
    pub target_emotion: Option<(EmotionKind, f32)>,
    /// Need changes for the actor.
    pub actor_needs: Vec<(String, f32)>,
    /// Need changes for the target.
    pub target_needs: Vec<(String, f32)>,
    /// **A good the actor hands over on accept**, with the amount: a trade.
    ///
    /// Pairwise barter, not a market: "a less-studied alternative to auction systems is the pattern of
    /// Edgeworth barter, in which buyers and sellers interact directly with each other in repeated pairwise
    /// transactions, without the benefit of an auctioneer" (corpus doc `10.1007_11678809_10`). The transfer
    /// is *declarative* — the activity says what changes hands — so no new system decides who trades with
    /// whom; `Perceive` already does, and the acceptance draw decides whether it happens.
    #[serde(default)]
    pub actor_gives: Vec<(String, f32)>,
    /// A good the target hands over on accept, with the amount.
    #[serde(default)]
    pub target_gives: Vec<(String, f32)>,
    /// How memorable the outcome is, in [0, 1].
    pub importance: f32,
    /// Rhai statements run with actor and target bound.
    pub script: Option<String>,
}

impl Default for EffectsDef {
    fn default() -> Self {
        Self {
            actor_gives: Vec::new(),
            target_gives: Vec::new(),
            actor_to_target: TieDelta::default(),
            target_to_actor: TieDelta::default(),
            actor_emotion: None,
            target_emotion: None,
            actor_needs: Vec::new(),
            target_needs: Vec::new(),
            importance: 0.3,
            script: None,
        }
    }
}

/// Bark lines for an interaction. Templates may use `{self}`, `{target}`, `{subject}` and `{place}`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct InteractionBarks {
    /// What the actor says when it starts the exchange.
    pub actor: Vec<String>,
    /// What the target says when it accepts.
    pub accepted: Vec<String>,
    /// What the target says when it refuses.
    pub rejected: Vec<String>,
}

/// A rule that fires on a simulation event and runs a script.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerDef {
    /// Unique name, used in error messages.
    pub id: String,
    /// The event that starts evaluation.
    pub on: TriggerOn,
    /// Rhai boolean that must hold for `then` to run.
    #[serde(rename = "if")]
    pub condition: Option<String>,
    /// Rhai statements to run.
    pub then: String,
    /// Fire at most once per (trigger, actor).
    #[serde(default)]
    pub once: bool,
    /// Minimum game-hours between firings per (trigger, actor).
    #[serde(default)]
    pub cooldown_hours: f32,
}

/// The events a trigger can listen to.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum TriggerOn {
    /// Every simulation tick.
    Tick,
    /// When the clock crosses this hour, in [0, 24).
    Hour(f32),
    /// When a person dies, with the deceased bound as the actor.
    PersonDied,
    /// When this interaction resolves.
    Interaction(String),
    /// When a tie's label changes to this label.
    TieLabel(TieLabel),
    /// When a rumor of this deed first reaches this fraction of living persons.
    RumorReach {
        /// The deed.
        deed: String,
        /// Fraction in (0, 1].
        fraction: f32,
    },
    /// When a witness judges this deed a norm violation.
    NormViolated(String),
    /// When a person's need first falls below this level.
    NeedBelow {
        /// The need.
        need: String,
        /// The threshold level.
        below: f32,
    },
    /// When this activity starts.
    Activity(String),
    /// When a dialogue node is entered in this conversation.
    DialogueNode {
        /// The conversation name in the host's `DialogueScript`.
        conversation: String,
        /// The node name.
        node: String,
    },
    /// A named event the host fires with `FireTrigger`.
    Custom(String),
}

/// Chooses the conversation for a `TalkTo`. The highest priority matching rule wins.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationRuleDef {
    /// The conversation name in the host's `DialogueScript`.
    pub conversation: String,
    /// Higher wins; ties go to the earlier definition.
    #[serde(default)]
    pub priority: i32,
    /// Rhai boolean; actor is the NPC, target is the player.
    #[serde(rename = "if")]
    pub condition: Option<String>,
}

/// Runs a script when a dialogue node is entered.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeEffectDef {
    /// The conversation name.
    pub conversation: String,
    /// The node name.
    pub node: String,
    /// Rhai statements to run, with actor as the NPC and target as the player.
    pub then: String,
}

/// Why a definitions file was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefsError {
    /// The RON text did not parse.
    Ron(String),
    /// An id was defined twice.
    Duplicate {
        /// What kind of definition.
        kind: &'static str,
        /// The repeated id.
        id: String,
    },
    /// A definition names an id that is not defined.
    UnknownId {
        /// What kind of definition was expected.
        kind: &'static str,
        /// The missing id.
        id: String,
        /// The definition that names it.
        owner: String,
    },
    /// A value is out of range or a combination is not allowed.
    Invalid {
        /// The definition at fault.
        owner: String,
        /// What is wrong.
        message: String,
    },
    /// A Rhai script does not compile.
    Script {
        /// The definition that holds the script.
        owner: String,
        /// The Rhai parse error.
        message: String,
    },
}

impl fmt::Display for DefsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ron(message) => write!(f, "bevy_society: definitions do not parse: {message}"),
            Self::Duplicate { kind, id } => {
                write!(f, "bevy_society: {kind} '{id}' is defined more than once")
            }
            Self::UnknownId { kind, id, owner } => {
                write!(
                    f,
                    "bevy_society: {owner} names {kind} '{id}', which is not defined"
                )
            }
            Self::Invalid { owner, message } => write!(f, "bevy_society: {owner}: {message}"),
            Self::Script { owner, message } => {
                write!(
                    f,
                    "bevy_society: {owner}: script does not compile: {message}"
                )
            }
        }
    }
}

impl std::error::Error for DefsError {}

impl DefsFile {
    /// Parses a definitions file from RON text.
    pub fn from_ron(text: &str) -> Result<Self, DefsError> {
        ron::from_str(text).map_err(|err| DefsError::Ron(err.to_string()))
    }

    /// Adds `other` to `self`. An id defined in both files, or twice in `other`, is an error and
    /// `self` is left unchanged.
    pub fn merge(&mut self, other: DefsFile) -> Result<(), DefsError> {
        check_disjoint(
            "need",
            self.needs.iter().map(|d| d.id.as_str()),
            other.needs.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "signal",
            self.signals.iter().map(|d| d.id.as_str()),
            other.signals.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "topic",
            self.topics.iter().map(|d| d.id.as_str()),
            other.topics.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "deed",
            self.deeds.iter().map(|d| d.id.as_str()),
            other.deeds.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "faction",
            self.factions.iter().map(|d| d.id.as_str()),
            other.factions.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "activity",
            self.activities.iter().map(|d| d.id.as_str()),
            other.activities.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "interaction",
            self.interactions.iter().map(|d| d.id.as_str()),
            other.interactions.iter().map(|d| d.id.as_str()),
        )?;
        // **`goods` and `skills` were omitted here until 2026-10-26-13.** Eleven lists were carried and two
        // were not, so a pack merged in that declares a good contributed nothing: `compile` then resolved
        // every `consumes` and `produces` that named it to "not defined", and the merge either failed with a
        // confusing unknown-id error or — for a good only the *other* pack used — dropped the good silently.
        // Found when `bake` moved into the starter pack, which is exactly the merge the village example does.
        check_disjoint(
            "good",
            self.goods.iter().map(|d| d.id.as_str()),
            other.goods.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "skill",
            self.skills.iter().map(|d| d.id.as_str()),
            other.skills.iter().map(|d| d.id.as_str()),
        )?;
        check_disjoint(
            "trigger",
            self.triggers.iter().map(|d| d.id.as_str()),
            other.triggers.iter().map(|d| d.id.as_str()),
        )?;
        self.needs.extend(other.needs);
        self.signals.extend(other.signals);
        self.skills.extend(other.skills);
        self.goods.extend(other.goods);
        self.topics.extend(other.topics);
        self.deeds.extend(other.deeds);
        self.factions.extend(other.factions);
        self.activities.extend(other.activities);
        self.interactions.extend(other.interactions);
        self.triggers.extend(other.triggers);
        self.conversations.extend(other.conversations);
        self.node_effects.extend(other.node_effects);
        Ok(())
    }

    /// The starter pack (`assets/starter.society.ron`).
    pub fn starter() -> Result<Self, DefsError> {
        Self::from_ron(include_str!("../assets/starter.society.ron"))
    }
}

fn check_disjoint<'a>(
    kind: &'static str,
    mine: impl Iterator<Item = &'a str>,
    theirs: impl Iterator<Item = &'a str>,
) -> Result<(), DefsError> {
    let mut seen: HashSet<&str> = mine.collect();
    for name in theirs {
        if !seen.insert(name) {
            return Err(DefsError::Duplicate {
                kind,
                id: name.to_owned(),
            });
        }
    }
    Ok(())
}

/// Compiled definitions: the resource the simulation reads. `Default` is the empty set plus the
/// reserved `idle` activity.
///
/// `Clone` because an exclusive system takes a copy before writing the world: a `&SocietyDefs`
/// borrowed from the world cannot coexist with `&mut World`, and every decision needs to read it.
#[derive(Resource, Clone)]
pub struct SocietyDefs {
    pub(crate) needs: Vec<NeedSpec>,
    pub(crate) signals: Vec<String>,
    /// Named capabilities, in definition order.
    pub(crate) skills: Vec<SkillGroup>,
    /// Goods, in definition order. Public so a host can count what its pack declares.
    pub goods: Vec<String>,
    pub(crate) topics: Vec<TopicSpec>,
    pub(crate) deeds: Vec<DeedSpec>,
    pub(crate) factions: Vec<FactionSpec>,
    pub(crate) activities: Vec<ActivitySpec>,
    pub(crate) interactions: Vec<InteractionSpec>,
    pub(crate) triggers: Vec<TriggerSpec>,
    pub(crate) conversations: Vec<ConversationSpec>,
    pub(crate) node_effects: Vec<NodeEffectSpec>,
    pub(crate) scripts: Vec<CompiledScript>,
    need_ids: HashMap<String, u16>,
    signal_ids: HashMap<String, u16>,
    skill_ids: HashMap<String, u16>,
    good_ids: HashMap<String, u16>,
    topic_ids: HashMap<String, u16>,
    deed_ids: HashMap<String, u16>,
    faction_ids: HashMap<String, u16>,
    activity_ids: HashMap<String, u16>,
    interaction_ids: HashMap<String, u16>,
}

impl Default for SocietyDefs {
    fn default() -> Self {
        Self::empty()
    }
}

impl SocietyDefs {
    fn empty() -> Self {
        let mut activity_ids = HashMap::new();
        activity_ids.insert(IDLE.to_owned(), IDLE_ACTIVITY.0);
        Self {
            needs: Vec::new(),
            signals: Vec::new(),
            skills: Vec::new(),
            goods: Vec::new(),
            good_ids: HashMap::new(),
            topics: Vec::new(),
            deeds: Vec::new(),
            factions: Vec::new(),
            activities: vec![ActivitySpec {
                name: IDLE.to_owned(),
                target: ActivityTarget::Itself,
                duration_hours: 1.0,
                rank: 0,
                weight: 1.0,
                considerations: Vec::new(),
                satisfies: Vec::new(),
                interaction: None,
                produces: Vec::new(),
                consumes: Vec::new(),
                barks: Vec::new(),
            }],
            interactions: Vec::new(),
            triggers: Vec::new(),
            conversations: Vec::new(),
            node_effects: Vec::new(),
            scripts: Vec::new(),
            need_ids: HashMap::new(),
            signal_ids: HashMap::new(),
            skill_ids: HashMap::new(),
            topic_ids: HashMap::new(),
            deed_ids: HashMap::new(),
            faction_ids: HashMap::new(),
            activity_ids,
            interaction_ids: HashMap::new(),
        }
    }

    /// Compiles authored definitions into ids and scripts. Errors name the offending definition.
    pub fn compile(file: DefsFile) -> Result<Self, DefsError> {
        let mut defs = Self::empty();
        let mut scripts = ScriptCompiler::new();
        for def in file.needs {
            defs.add_need(def)?;
        }
        for def in file.signals {
            defs.add_signal(def)?;
        }
        for def in file.skills {
            defs.add_skill(def)?;
        }
        for def in file.goods {
            defs.add_good(def)?;
        }
        for def in file.topics {
            defs.add_topic(def)?;
        }
        for def in file.deeds {
            defs.add_deed(def)?;
        }
        for def in file.factions {
            defs.add_faction(def)?;
        }
        for def in file.interactions {
            defs.add_interaction(def, &mut scripts)?;
        }
        for def in file.activities {
            defs.add_activity(def, &mut scripts)?;
        }
        let mut trigger_names = HashSet::new();
        for def in file.triggers {
            defs.add_trigger(def, &mut scripts, &mut trigger_names)?;
        }
        for def in file.conversations {
            defs.add_conversation(def, &mut scripts)?;
        }
        for def in file.node_effects {
            defs.add_node_effect(def, &mut scripts)?;
        }
        defs.scripts = scripts.into_scripts();
        Ok(defs)
    }

    /// Parses and compiles a definitions file from RON text.
    pub fn from_ron(text: &str) -> Result<Self, DefsError> {
        Self::compile(DefsFile::from_ron(text)?)
    }

    /// The compiled starter pack.
    pub fn starter() -> Result<Self, DefsError> {
        Self::compile(DefsFile::starter()?)
    }

    /// The id of a need, if defined.
    pub fn need_id(&self, name: &str) -> Option<NeedId> {
        self.need_ids.get(name).copied().map(NeedId)
    }

    /// The id of a good, if defined.
    pub fn good_id(&self, name: &str) -> Option<GoodId> {
        self.good_ids.get(name).copied().map(GoodId)
    }

    /// An activity's compiled specification, by its authored name.
    pub(crate) fn activity_by_name(&self, name: &str) -> Option<&ActivitySpec> {
        self.activity_id(name)
            .and_then(|id| self.activities.get(usize::from(id.0)))
    }

    /// The id of a skill, if defined.
    pub fn skill_id(&self, name: &str) -> Option<SkillId> {
        self.skill_ids.get(name).copied().map(SkillId)
    }

    /// The group a skill reads, by id.
    pub fn skill_group(&self, id: SkillId) -> Option<SkillGroup> {
        self.skills.get(usize::from(id.0)).copied()
    }

    /// The id of a signal, if defined.
    pub fn signal_id(&self, name: &str) -> Option<SignalId> {
        self.signal_ids.get(name).copied().map(SignalId)
    }

    /// The id of a topic, if defined.
    pub fn topic_id(&self, name: &str) -> Option<TopicId> {
        self.topic_ids.get(name).copied().map(TopicId)
    }

    /// The id of a deed, if defined.
    pub fn deed_id(&self, name: &str) -> Option<DeedId> {
        self.deed_ids.get(name).copied().map(DeedId)
    }

    /// The id of a faction, if defined.
    pub fn faction_id(&self, name: &str) -> Option<FactionId> {
        self.faction_ids.get(name).copied().map(FactionId)
    }

    /// The id of an activity, if defined. `idle` is always defined.
    pub fn activity_id(&self, name: &str) -> Option<ActivityId> {
        self.activity_ids.get(name).copied().map(ActivityId)
    }

    /// The id of an interaction, if defined.
    pub fn interaction_id(&self, name: &str) -> Option<InteractionId> {
        self.interaction_ids.get(name).copied().map(InteractionId)
    }

    /// The name of a need, or `""` for an id this definition set does not contain.
    pub fn need_name(&self, id: NeedId) -> &str {
        self.needs
            .get(usize::from(id.0))
            .map_or("", |spec| spec.name.as_str())
    }

    /// The name of a signal, or `""` for an id this definition set does not contain.
    pub fn signal_name(&self, id: SignalId) -> &str {
        self.signals
            .get(usize::from(id.0))
            .map_or("", String::as_str)
    }

    /// The name of a topic, or `""` for an unknown id.
    pub fn topic_name(&self, id: TopicId) -> &str {
        self.topics
            .get(usize::from(id.0))
            .map_or("", |spec| spec.name.as_str())
    }

    /// The name of a deed, or `""` for an unknown id.
    pub fn deed_name(&self, id: DeedId) -> &str {
        self.deeds
            .get(usize::from(id.0))
            .map_or("", |spec| spec.name.as_str())
    }

    /// The name of a faction, or `""` for an unknown id.
    pub fn faction_name(&self, id: FactionId) -> &str {
        self.factions
            .get(usize::from(id.0))
            .map_or("", |spec| spec.name.as_str())
    }

    /// The name of an activity, or `""` for an unknown id.
    pub fn activity_name(&self, id: ActivityId) -> &str {
        self.activities
            .get(usize::from(id.0))
            .map_or("", |spec| spec.name.as_str())
    }

    /// Every activity's name with the signals its considerations read, in definition order.
    ///
    /// A host needs this to check whether a definition relies on a signal the behaviour systems do
    /// not yet score: `BehaviorPlugin` warns at startup about `Field` and about `Gradient` targets,
    /// which the definitions file accepts but which currently choose nothing.
    pub fn activity_signals(&self) -> Vec<(&str, Vec<crate::defs::Signal>)> {
        self.activities
            .iter()
            .map(|spec| {
                (
                    spec.name.as_str(),
                    spec.considerations.iter().map(|term| term.signal).collect(),
                )
            })
            .collect()
    }

    /// How fast `need` decays per game-day, or `None` when the name is not defined.
    ///
    /// With [`Self::best_source`] this is the whole of a need's budget: a need whose best single use
    /// supplies less than its decay must be met more than once a day, and a definition where that
    /// number is large describes people who starve.
    pub fn need_decay(&self, need: &str) -> Option<f32> {
        let id = self.need_id(need)?;
        self.needs
            .get(usize::from(id.0))
            .map(|spec| spec.decay_per_day)
    }

    /// Every activity that supplies `need`, with the amount each pays per use, in definition order.
    ///
    /// A need with one source is a need with no choice: whatever else competes for the same hour takes
    /// from it. `fun` in the starter pack had exactly one.
    pub fn sources_for(&self, need: &str) -> Vec<(&str, f32)> {
        let Some(id) = self.need_id(need) else {
            return Vec::new();
        };
        self.activities
            .iter()
            .filter_map(|activity| {
                let rate = activity
                    .satisfies
                    .iter()
                    .find(|(slot, _)| *slot == id)
                    .map(|(_, rate)| *rate)?;
                Some((activity.name.as_str(), rate * activity.duration_hours))
            })
            .collect()
    }

    /// The activity that supplies `need` most per use, with the amount it supplies, or `None` when no
    /// activity satisfies it.
    ///
    /// A use supplies `satisfies · duration_hours`, because `satisfies` is a per-game-hour rate paid
    /// for the activity's whole duration. Ties go to the earlier activity in definition order.
    pub fn best_source(&self, need: &str) -> Option<(&str, f32)> {
        let id = self.need_id(need)?;
        self.activities
            .iter()
            .filter_map(|activity| {
                let rate = activity
                    .satisfies
                    .iter()
                    .find(|(slot, _)| *slot == id)
                    .map(|(_, rate)| *rate)?;
                Some((activity.name.as_str(), rate * activity.duration_hours))
            })
            .fold(None::<(&str, f32)>, |best, candidate| match best {
                Some((_, amount)) if amount >= candidate.1 => best,
                _ => Some(candidate),
            })
    }

    /// Whether the activity called `name` targets a field gradient.
    pub fn activity_targets_gradient(&self, name: &str) -> bool {
        self.activities
            .iter()
            .any(|spec| spec.name == name && matches!(spec.target, ActivityTarget::Gradient(..)))
    }

    /// Whether any compiled activity targets a field gradient.
    pub fn has_gradient_activity(&self) -> bool {
        self.activities
            .iter()
            .any(|spec| matches!(spec.target, ActivityTarget::Gradient(..)))
    }

    /// Every activity's name with its duration in game-hours, in definition order.
    ///
    /// A host needs this to check its own `seconds_per_day`: a duration shorter than a society tick
    /// makes the activity degenerate, and `BehaviorPlugin` warns about it at startup. Exposed rather
    /// than inferred so a host (or a test) can make the same check without the compiled internals.
    pub fn activity_durations(&self) -> Vec<(&str, f32)> {
        self.activities
            .iter()
            .map(|spec| (spec.name.as_str(), spec.duration_hours))
            .collect()
    }

    /// The name of an interaction, or `""` for an unknown id.
    pub fn interaction_name(&self, id: InteractionId) -> &str {
        self.interactions
            .get(usize::from(id.0))
            .map_or("", |spec| spec.name.as_str())
    }

    /// Needs for a new person: each need starts at its `initial` level, then the named values
    /// replace theirs.
    pub fn needs<'a>(
        &self,
        values: impl IntoIterator<Item = (&'a str, f32)>,
    ) -> Result<Needs, DefsError> {
        let mut levels: Vec<f32> = self.needs.iter().map(|spec| spec.initial).collect();
        for (name, level) in values {
            let id = require(self.need_id(name), "need", name, "person needs")?;
            if let Some(slot) = levels.get_mut(usize::from(id.0)) {
                *slot = level;
            }
        }
        Ok(Needs(levels))
    }

    /// Signals for a new person: one slot per defined signal, every value `0.0`, then the named
    /// values replace theirs. The crate never changes them afterwards.
    pub fn signals<'a>(
        &self,
        values: impl IntoIterator<Item = (&'a str, f32)>,
    ) -> Result<Signals, DefsError> {
        let mut levels = vec![0.0; self.signals.len()];
        for (name, value) in values {
            let id = require(self.signal_id(name), "signal", name, "person signals")?;
            if let Some(slot) = levels.get_mut(usize::from(id.0)) {
                *slot = if value.is_finite() { value } else { 0.0 };
            }
        }
        Ok(Signals(levels))
    }

    /// Opinions for a new person. Each named topic starts at its value, which is also its
    /// Friedkin–Johnsen anchor (DOI 10.1080/0022250X.1990.9990069); the others start neutral.
    pub fn opinions<'a>(
        &self,
        values: impl IntoIterator<Item = (&'a str, f32)>,
    ) -> Result<Opinions, DefsError> {
        let mut topics = vec![Opinion::default(); self.topics.len()];
        for (name, value) in values {
            let id = require(self.topic_id(name), "topic", name, "person opinions")?;
            if let Some(slot) = topics.get_mut(usize::from(id.0)) {
                *slot = Opinion::at(value);
            }
        }
        Ok(Opinions(topics))
    }

    /// Faction memberships for a new person, sorted and without duplicates.
    pub fn memberships<'a>(
        &self,
        factions: impl IntoIterator<Item = &'a str>,
    ) -> Result<Memberships, DefsError> {
        let mut ids = Vec::new();
        for name in factions {
            ids.push(require(
                self.faction_id(name),
                "faction",
                name,
                "person memberships",
            )?);
        }
        ids.sort();
        ids.dedup();
        Ok(Memberships(ids))
    }

    fn add_need(&mut self, def: NeedDef) -> Result<(), DefsError> {
        let owner = format!("need '{}'", def.id);
        if def.decay_per_day.is_nan() || def.decay_per_day < 0.0 {
            return Err(invalid(&owner, "decay_per_day must be 0 or more"));
        }
        register(&mut self.need_ids, "need", &def.id, self.needs.len())?;
        self.needs.push(NeedSpec {
            name: def.id,
            decay_per_day: def.decay_per_day,
            initial: def.initial,
        });
        Ok(())
    }

    fn add_signal(&mut self, def: SignalDef) -> Result<(), DefsError> {
        register(&mut self.signal_ids, "signal", &def.id, self.signals.len())?;
        self.signals.push(def.id);
        Ok(())
    }

    fn add_skill(&mut self, def: SkillDef) -> Result<(), DefsError> {
        register(&mut self.skill_ids, "skill", &def.id, self.skills.len())?;
        self.skills.push(def.group);
        Ok(())
    }

    fn add_good(&mut self, def: GoodDef) -> Result<(), DefsError> {
        register(&mut self.good_ids, "good", &def.id, self.goods.len())?;
        self.goods.push(def.id);
        Ok(())
    }

    fn add_topic(&mut self, def: TopicDef) -> Result<(), DefsError> {
        register(&mut self.topic_ids, "topic", &def.id, self.topics.len())?;
        self.topics.push(TopicSpec { name: def.id });
        Ok(())
    }

    fn add_deed(&mut self, def: DeedDef) -> Result<(), DefsError> {
        let owner = format!("deed '{}'", def.id);
        let topic = require(self.topic_id(&def.topic), "topic", &def.topic, &owner)?;
        if !(-1.0..=1.0).contains(&def.harm) {
            return Err(invalid(&owner, "harm must be in [-1, 1]"));
        }
        register(&mut self.deed_ids, "deed", &def.id, self.deeds.len())?;
        self.deeds.push(DeedSpec {
            name: def.id,
            topic,
            harm: def.harm,
        });
        Ok(())
    }

    fn add_faction(&mut self, def: FactionDef) -> Result<(), DefsError> {
        register(
            &mut self.faction_ids,
            "faction",
            &def.id,
            self.factions.len(),
        )?;
        self.factions.push(FactionSpec {
            name: def.id,
            ingroup_bias: def.ingroup_bias,
        });
        Ok(())
    }

    fn add_interaction(
        &mut self,
        def: InteractionDef,
        scripts: &mut ScriptCompiler,
    ) -> Result<(), DefsError> {
        let owner = format!("interaction '{}'", def.id);
        register(
            &mut self.interaction_ids,
            "interaction",
            &def.id,
            self.interactions.len(),
        )?;
        let requires = match def.requires.as_deref() {
            Some(source) => Some(scripts.expression(&format!("{owner} requires"), source)?),
            None => None,
        };
        let on_accept =
            self.compile_effects(&format!("{owner} on_accept"), &def.on_accept, scripts)?;
        let on_reject =
            self.compile_effects(&format!("{owner} on_reject"), &def.on_reject, scripts)?;
        let mut influence = Vec::with_capacity(def.influence.len());
        for topic in &def.influence {
            influence.push(require(self.topic_id(topic), "topic", topic, &owner)?);
        }
        let deed = match def.deed.as_deref() {
            Some(name) => Some(require(self.deed_id(name), "deed", name, &owner)?),
            None => None,
        };
        let barks_owner = format!("{owner} barks");
        for line in def
            .barks
            .actor
            .iter()
            .chain(&def.barks.accepted)
            .chain(&def.barks.rejected)
        {
            check_template(&barks_owner, line)?;
        }
        self.interactions.push(InteractionSpec {
            name: def.id,
            requires,
            accept: def.accept,
            on_accept,
            on_reject,
            influence,
            gossip: def.gossip,
            deed,
            contest: def.contest,
            barks: def.barks,
        });
        Ok(())
    }

    fn compile_effects(
        &self,
        owner: &str,
        def: &EffectsDef,
        scripts: &mut ScriptCompiler,
    ) -> Result<EffectsSpec, DefsError> {
        let actor_needs = self.need_deltas(owner, &def.actor_needs)?;
        let target_needs = self.need_deltas(owner, &def.target_needs)?;
        let actor_gives = self.good_list(owner, &def.actor_gives)?;
        let target_gives = self.good_list(owner, &def.target_gives)?;
        let script = match def.script.as_deref() {
            Some(source) => Some(scripts.statements(owner, source)?),
            None => None,
        };
        Ok(EffectsSpec {
            actor_to_target: def.actor_to_target,
            target_to_actor: def.target_to_actor,
            actor_emotion: def.actor_emotion,
            target_emotion: def.target_emotion,
            actor_needs,
            target_needs,
            actor_gives,
            target_gives,
            importance: def.importance,
            script,
        })
    }

    fn need_deltas(
        &self,
        owner: &str,
        pairs: &[(String, f32)],
    ) -> Result<Vec<(NeedId, f32)>, DefsError> {
        let mut out = Vec::with_capacity(pairs.len());
        for (name, delta) in pairs {
            out.push((require(self.need_id(name), "need", name, owner)?, *delta));
        }
        Ok(out)
    }

    fn add_activity(
        &mut self,
        def: ActivityDef,
        scripts: &mut ScriptCompiler,
    ) -> Result<(), DefsError> {
        let owner = format!("activity '{}'", def.id);
        if def.duration_hours.is_nan() || def.duration_hours <= 0.0 {
            return Err(invalid(&owner, "duration_hours must be above 0"));
        }
        if def.weight.is_nan() || def.weight < 0.0 {
            return Err(invalid(&owner, "weight must be 0 or more"));
        }
        check_target(&owner, &def.target)?;
        register(
            &mut self.activity_ids,
            "activity",
            &def.id,
            self.activities.len(),
        )?;
        let mut considerations = Vec::with_capacity(def.considerations.len());
        for (index, term) in def.considerations.iter().enumerate() {
            let term_owner = format!("{owner} consideration {index}");
            considerations.push(self.compile_term(&term_owner, term, &def.target, scripts)?);
        }
        let satisfies_owner = format!("{owner} satisfies");
        let satisfies = self.need_deltas(&satisfies_owner, &def.satisfies)?;
        let interaction = match def.interaction.as_deref() {
            Some(name) => {
                if !matches!(def.target, ActivityTarget::Person) {
                    return Err(invalid(&owner, "an interaction needs a Person target"));
                }
                Some(require(
                    self.interaction_id(name),
                    "interaction",
                    name,
                    &owner,
                )?)
            }
            None => None,
        };
        let barks_owner = format!("{owner} barks");
        for line in &def.barks {
            check_template(&barks_owner, line)?;
        }
        // Goods, resolved against the definitions file. An unknown name is an error rather than a
        // silent no-op: a task that produces nothing because of a typo is a bug a host cannot see.
        let produces = self.good_list(&owner, &def.produces)?;
        let consumes = self.good_list(&owner, &def.consumes)?;
        self.activities.push(ActivitySpec {
            name: def.id,
            target: def.target,
            duration_hours: def.duration_hours,
            rank: def.rank,
            weight: def.weight,
            considerations,
            satisfies,
            interaction,
            produces,
            consumes,
            barks: def.barks,
        });
        Ok(())
    }

    /// Resolves `(name, amount)` pairs to goods, refusing an unknown name or a negative amount.
    fn good_list(
        &self,
        owner: &str,
        list: &[(String, f32)],
    ) -> Result<Vec<(GoodId, f32)>, DefsError> {
        let mut out = Vec::with_capacity(list.len());
        for (name, amount) in list {
            if *amount < 0.0 {
                return Err(invalid(
                    owner,
                    format!("good '{name}' has a negative amount"),
                ));
            }
            out.push((require(self.good_id(name), "good", name, owner)?, *amount));
        }
        Ok(out)
    }

    fn compile_term(
        &self,
        owner: &str,
        term: &Consideration,
        target: &ActivityTarget,
        scripts: &mut ScriptCompiler,
    ) -> Result<Term, DefsError> {
        check_input(owner, &term.input, target)?;
        if matches!(term.curve, Curve::Hours(..)) && !matches!(term.input, Input::Hour) {
            return Err(invalid(owner, "an Hours curve needs the Hour input"));
        }
        let signal = match &term.input {
            Input::Need(name) => Signal::Need(require(self.need_id(name), "need", name, owner)?),
            Input::Signal(name) => {
                Signal::Host(require(self.signal_id(name), "signal", name, owner)?)
            }
            Input::Hour => Signal::Hour,
            Input::Pleasure => Signal::Pleasure,
            Input::Arousal => Signal::Arousal,
            Input::Dominance => Signal::Dominance,
            Input::Trait(personality) => Signal::Trait(*personality),
            Input::Emotion(kind) => Signal::Emotion(*kind),
            Input::Affinity => Signal::Affinity,
            Input::Trust => Signal::Trust,
            Input::Respect => Signal::Respect,
            Input::Familiarity => Signal::Familiarity,
            Input::Distance => Signal::Distance,
            Input::SocialProof => Signal::SocialProof,
            Input::Skill(name) => {
                Signal::Skill(require(self.skill_id(name), "skill", name, owner)?)
            }
            Input::Surplus(name) => {
                Signal::Surplus(require(self.good_id(name), "good", name, owner)?)
            }
            Input::Field(field) => Signal::Field(*field),
            Input::Script(source) => Signal::Script(scripts.expression(owner, source)?),
        };
        Ok(Term {
            signal,
            curve: term.curve,
        })
    }

    fn add_trigger(
        &mut self,
        def: TriggerDef,
        scripts: &mut ScriptCompiler,
        names: &mut HashSet<String>,
    ) -> Result<(), DefsError> {
        let owner = format!("trigger '{}'", def.id);
        if !names.insert(def.id.clone()) {
            return Err(DefsError::Duplicate {
                kind: "trigger",
                id: def.id,
            });
        }
        if def.cooldown_hours.is_nan() || def.cooldown_hours < 0.0 {
            return Err(invalid(&owner, "cooldown_hours must be 0 or more"));
        }
        let on = self.trigger_kind(&owner, &def.on)?;
        let condition = match def.condition.as_deref() {
            Some(source) => Some(scripts.expression(&format!("{owner} if"), source)?),
            None => None,
        };
        let then = scripts.statements(&owner, &def.then)?;
        self.triggers.push(TriggerSpec {
            on,
            condition,
            then,
            once: def.once,
            cooldown_hours: def.cooldown_hours,
        });
        Ok(())
    }

    fn trigger_kind(&self, owner: &str, on: &TriggerOn) -> Result<TriggerKind, DefsError> {
        Ok(match on {
            TriggerOn::Tick => TriggerKind::Tick,
            TriggerOn::PersonDied => TriggerKind::PersonDied,
            TriggerOn::Hour(hour) => {
                if !(0.0..24.0).contains(hour) {
                    return Err(invalid(owner, "an Hour trigger needs an hour in [0, 24)"));
                }
                TriggerKind::Hour(*hour)
            }
            TriggerOn::Interaction(name) => TriggerKind::Interaction(require(
                self.interaction_id(name),
                "interaction",
                name,
                owner,
            )?),
            TriggerOn::TieLabel(label) => TriggerKind::TieLabel(*label),
            TriggerOn::RumorReach { deed, fraction } => {
                let deed = require(self.deed_id(deed), "deed", deed, owner)?;
                if !(*fraction > 0.0 && *fraction <= 1.0) {
                    return Err(invalid(owner, "fraction must be in (0, 1]"));
                }
                TriggerKind::RumorReach {
                    deed,
                    fraction: *fraction,
                }
            }
            TriggerOn::NormViolated(deed) => {
                TriggerKind::NormViolated(require(self.deed_id(deed), "deed", deed, owner)?)
            }
            TriggerOn::NeedBelow { need, below } => TriggerKind::NeedBelow {
                need: require(self.need_id(need), "need", need, owner)?,
                below: *below,
            },
            TriggerOn::Activity(name) => {
                TriggerKind::Activity(require(self.activity_id(name), "activity", name, owner)?)
            }
            TriggerOn::DialogueNode { conversation, node } => TriggerKind::DialogueNode {
                conversation: conversation.clone(),
                node: node.clone(),
            },
            TriggerOn::Custom(name) => TriggerKind::Custom(name.clone()),
        })
    }

    fn add_conversation(
        &mut self,
        def: ConversationRuleDef,
        scripts: &mut ScriptCompiler,
    ) -> Result<(), DefsError> {
        let owner = format!("conversation rule for '{}'", def.conversation);
        let condition = match def.condition.as_deref() {
            Some(source) => Some(scripts.expression(&owner, source)?),
            None => None,
        };
        self.conversations.push(ConversationSpec {
            conversation: def.conversation,
            priority: def.priority,
            condition,
        });
        Ok(())
    }

    fn add_node_effect(
        &mut self,
        def: NodeEffectDef,
        scripts: &mut ScriptCompiler,
    ) -> Result<(), DefsError> {
        let owner = format!("node effect '{}/{}'", def.conversation, def.node);
        let then = scripts.statements(&owner, &def.then)?;
        self.node_effects.push(NodeEffectSpec {
            conversation: def.conversation,
            node: def.node,
            then,
        });
        Ok(())
    }
}

/// The per-person state [`size_person_state`] sizes, bundled to stay inside clippy's type-complexity limit.
type PersonStateQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Needs,
        &'static mut Signals,
        &'static mut Opinions,
        Option<&'static mut crate::Stock>,
    ),
    With<Person>,
>;

/// Keeps every person's `Needs`, `Signals`, `Opinions` and `Stock` the length of the defs (plan D2). A
/// short vector grows with the defs' starting values; a long one is an error and is truncated. `Mut`
/// changes only when a length actually differs.
///
/// **`Stock` is added here too**, as a `Component` when the pack declares goods, so a host that adds a good
/// to its definitions file does not also have to remember to spawn one.
pub fn size_person_state(
    defs: Res<SocietyDefs>,
    mut commands: Commands,
    mut people: PersonStateQuery,
) {
    let need_count = defs.needs.len();
    let signal_count = defs.signals.len();
    let topic_count = defs.topics.len();
    let good_count = defs.goods.len();
    for (entity, mut needs, mut signals, mut opinions, stock) in &mut people {
        // **Stock is sized with the rest of the per-person state**, so a host that adds a good to its
        // definitions file does not have to remember to spawn a component for it.
        match stock {
            Some(mut stock) => {
                if stock.0.len() != good_count {
                    stock.0.resize(good_count, 0.0);
                }
            }
            None if good_count > 0 => {
                commands
                    .entity(entity)
                    .insert(crate::Stock(vec![0.0; good_count]));
            }
            None => {}
        }
        let have = needs.0.len();
        if have > need_count {
            error!(
                "bevy_society: {entity} has {have} needs but the defs define {need_count}; truncating"
            );
            needs.0.truncate(need_count);
        } else if have < need_count {
            needs
                .0
                .extend(defs.needs.iter().skip(have).map(|spec| spec.initial));
        }
        let have = signals.0.len();
        if have > signal_count {
            error!(
                "bevy_society: {entity} has {have} signals but the defs define {signal_count}; truncating"
            );
            signals.0.truncate(signal_count);
        } else if have < signal_count {
            signals
                .0
                .extend(std::iter::repeat_n(0.0, signal_count - have));
        }
        let have = opinions.0.len();
        if have > topic_count {
            error!(
                "bevy_society: {entity} has {have} opinions but the defs define {topic_count}; truncating"
            );
            opinions.0.truncate(topic_count);
        } else if have < topic_count {
            opinions
                .0
                .extend(std::iter::repeat_n(Opinion::default(), topic_count - have));
        }
    }
}

/// A compiled need.
#[derive(Debug, Clone)]
pub(crate) struct NeedSpec {
    pub(crate) name: String,
    pub(crate) decay_per_day: f32,
    pub(crate) initial: f32,
}

/// A compiled topic.
#[derive(Debug, Clone)]
pub(crate) struct TopicSpec {
    pub(crate) name: String,
}

/// A compiled deed.
#[derive(Debug, Clone)]
pub(crate) struct DeedSpec {
    pub(crate) name: String,
    pub(crate) topic: TopicId,
    pub(crate) harm: f32,
}

/// A compiled faction.
#[derive(Debug, Clone)]
pub(crate) struct FactionSpec {
    pub(crate) name: String,
    pub(crate) ingroup_bias: f32,
}

/// A compiled activity.
#[derive(Debug, Clone)]
pub(crate) struct ActivitySpec {
    pub(crate) name: String,
    pub(crate) target: ActivityTarget,
    pub(crate) duration_hours: f32,
    pub(crate) rank: u8,
    pub(crate) weight: f32,
    pub(crate) considerations: Vec<Term>,
    pub(crate) satisfies: Vec<(NeedId, f32)>,
    pub(crate) interaction: Option<InteractionId>,
    /// Goods made per completed use.
    pub(crate) produces: Vec<(GoodId, f32)>,
    /// Goods used up per completed use.
    pub(crate) consumes: Vec<(GoodId, f32)>,
    pub(crate) barks: Vec<String>,
}

/// A compiled consideration: a signal through a curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Term {
    pub(crate) signal: Signal,
    pub(crate) curve: Curve,
}

/// A compiled signal, with ids resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// A need's level.
    Need(NeedId),
    /// A host-written signal value.
    Host(SignalId),
    /// Clock hour / 24.
    Hour,
    /// Mood pleasure.
    Pleasure,
    /// Mood arousal.
    Arousal,
    /// Mood dominance.
    Dominance,
    /// A personality trait.
    Trait(Trait),
    /// An emotion's intensity.
    Emotion(EmotionKind),
    /// Affinity toward the target.
    Affinity,
    /// Trust toward the target.
    Trust,
    /// Respect toward the target.
    Respect,
    /// Familiarity with the target.
    Familiarity,
    /// Distance to the target.
    Distance,
    /// Share of close ties doing the activity.
    SocialProof,
    /// A skill's level, read from the actor's personality.
    Skill(SkillId),
    /// The joint surplus a trade in this good would produce.
    Surplus(GoodId),
    /// A social field value.
    Field(FieldInput),
    /// A compiled Rhai expression.
    Script(ScriptId),
}

/// A compiled effect set.
#[derive(Debug, Clone)]
pub(crate) struct EffectsSpec {
    pub(crate) actor_to_target: TieDelta,
    pub(crate) target_to_actor: TieDelta,
    pub(crate) actor_emotion: Option<(EmotionKind, f32)>,
    pub(crate) target_emotion: Option<(EmotionKind, f32)>,
    pub(crate) actor_needs: Vec<(NeedId, f32)>,
    pub(crate) target_needs: Vec<(NeedId, f32)>,
    pub(crate) actor_gives: Vec<(GoodId, f32)>,
    pub(crate) target_gives: Vec<(GoodId, f32)>,
    pub(crate) importance: f32,
    pub(crate) script: Option<ScriptId>,
}

/// A compiled interaction.
#[derive(Debug, Clone)]
pub(crate) struct InteractionSpec {
    pub(crate) name: String,
    pub(crate) requires: Option<ScriptId>,
    pub(crate) accept: AcceptDef,
    pub(crate) on_accept: EffectsSpec,
    pub(crate) on_reject: EffectsSpec,
    pub(crate) influence: Vec<TopicId>,
    pub(crate) gossip: bool,
    pub(crate) deed: Option<DeedId>,
    pub(crate) contest: bool,
    pub(crate) barks: InteractionBarks,
}

/// A compiled trigger event.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TriggerKind {
    /// Every tick.
    Tick,
    /// The clock crossing an hour.
    Hour(f32),
    /// A person died; the actor binding is the deceased.
    PersonDied,
    /// An interaction resolving.
    Interaction(InteractionId),
    /// A tie label changing to this label.
    TieLabel(TieLabel),
    /// A rumor of the deed reaching a fraction.
    RumorReach {
        /// The deed.
        deed: DeedId,
        /// The fraction.
        fraction: f32,
    },
    /// A norm violation for the deed.
    NormViolated(DeedId),
    /// A need falling below a level.
    NeedBelow {
        /// The need.
        need: NeedId,
        /// The threshold.
        below: f32,
    },
    /// An activity starting.
    Activity(ActivityId),
    /// A dialogue node being entered.
    DialogueNode {
        /// The conversation.
        conversation: String,
        /// The node.
        node: String,
    },
    /// A host-fired named event.
    Custom(String),
}

/// A compiled trigger.
#[derive(Debug, Clone)]
pub(crate) struct TriggerSpec {
    pub(crate) on: TriggerKind,
    pub(crate) condition: Option<ScriptId>,
    pub(crate) then: ScriptId,
    pub(crate) once: bool,
    pub(crate) cooldown_hours: f32,
}

/// A compiled conversation rule.
#[derive(Debug, Clone)]
pub(crate) struct ConversationSpec {
    pub(crate) conversation: String,
    pub(crate) priority: i32,
    pub(crate) condition: Option<ScriptId>,
}

/// A compiled node effect.
#[derive(Debug, Clone)]
pub(crate) struct NodeEffectSpec {
    pub(crate) conversation: String,
    pub(crate) node: String,
    pub(crate) then: ScriptId,
}

fn invalid(owner: &str, message: impl Into<String>) -> DefsError {
    DefsError::Invalid {
        owner: owner.to_owned(),
        message: message.into(),
    }
}

fn require<T>(
    found: Option<T>,
    kind: &'static str,
    name: &str,
    owner: &str,
) -> Result<T, DefsError> {
    found.ok_or_else(|| DefsError::UnknownId {
        kind,
        id: name.to_owned(),
        owner: owner.to_owned(),
    })
}

/// Records a new id and returns its index. The index is the position the definition will take in
/// its `Vec`, so it must be the current length.
fn register(
    map: &mut HashMap<String, u16>,
    kind: &'static str,
    name: &str,
    index: usize,
) -> Result<u16, DefsError> {
    if name.is_empty() {
        return Err(invalid(kind, "an id must not be empty"));
    }
    if map.contains_key(name) {
        return Err(DefsError::Duplicate {
            kind,
            id: name.to_owned(),
        });
    }
    let id = u16::try_from(index).map_err(|_| {
        invalid(
            &format!("{kind} '{name}'"),
            "too many definitions of this kind (limit 65536)",
        )
    })?;
    map.insert(name.to_owned(), id);
    Ok(id)
}

fn check_target(owner: &str, target: &ActivityTarget) -> Result<(), DefsError> {
    if let ActivityTarget::Bound(name) = target
        && name.trim().is_empty()
    {
        return Err(invalid(owner, "a Bound target needs a name"));
    }
    if let ActivityTarget::Wander(radius) = target
        && (radius.is_nan() || *radius < 0.0)
    {
        return Err(invalid(owner, "Wander radius must be 0 or more"));
    }
    if matches!(target, ActivityTarget::Gradient(..)) && !FIELDS_ENABLED {
        return Err(invalid(
            owner,
            "a Gradient target needs the `fields` feature",
        ));
    }
    Ok(())
}

fn check_input(owner: &str, input: &Input, target: &ActivityTarget) -> Result<(), DefsError> {
    let person_only = matches!(
        input,
        Input::Affinity
            | Input::Trust
            | Input::Respect
            | Input::Familiarity
            | Input::SocialProof
            | Input::Surplus(_)
    );
    if person_only && !matches!(target, ActivityTarget::Person) {
        return Err(invalid(owner, format!("{input:?} needs a Person target")));
    }
    if matches!(input, Input::Distance)
        && !matches!(
            target,
            ActivityTarget::Place(_)
                | ActivityTarget::Home
                | ActivityTarget::Bound(_)
                | ActivityTarget::Person
        )
    {
        return Err(invalid(
            owner,
            "Distance needs a Place, Home, Bound or Person target",
        ));
    }
    if matches!(input, Input::Field(_)) && !FIELDS_ENABLED {
        return Err(invalid(owner, "field inputs need the `fields` feature"));
    }
    Ok(())
}

/// Checks a bark template: only `{self}`, `{target}`, `{subject}` and `{place}` may appear in braces.
fn check_template(owner: &str, text: &str) -> Result<(), DefsError> {
    let mut rest = text;
    while let Some(position) = rest.find(['{', '}']) {
        let tail = &rest[position..];
        if tail.starts_with('}') {
            return Err(invalid(owner, format!("stray '}}' in bark '{text}'")));
        }
        let Some(end) = tail.find('}') else {
            return Err(invalid(owner, format!("unclosed '{{' in bark '{text}'")));
        };
        let key = &tail[1..end];
        if !matches!(key, "self" | "target" | "subject" | "place") {
            return Err(invalid(
                owner,
                format!("unknown placeholder '{{{key}}}' in bark '{text}'"),
            ));
        }
        rest = &tail[end + 1..];
    }
    Ok(())
}

fn default_ingroup_bias() -> f32 {
    0.2
}

fn one() -> f32 {
    1.0
}

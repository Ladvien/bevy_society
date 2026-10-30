//! Save and restore the whole society, keyed by host-stable ids (issue #7).
//!
//! A host that autosaves at dawn and continues from the file needs the society to come back exactly
//! as it left: friendships, grudges, opinions, memories, rumors, the clock, the RNG's position and
//! the once/cooldown bookkeeping of every trigger. A society that reset on reload would not be one.
//!
//! **Nothing here stores an `Entity`.** An entity id is an index into one world's allocator and the
//! next session's is unrelated, so every reference to a person is a [`PersonKey`] — the id the host
//! already has for that person (Sanctuary's roster id) — and every reference to a place is a
//! [`PlaceKey`]. Bevy's `serialize` feature therefore stays off, and a save is plain, portable data.
//!
//! The two halves are:
//!
//! - [`capture`] reads the world into a [`SocietySave`] and never writes to it;
//! - [`restore`] validates the save completely, then writes it into a world the host has already
//!   spawned its persons and keyed places into. A save that cannot be applied returns a
//!   [`RestoreError`] instead of leaving the world half-written.
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_society::{SocietySave, capture, restore};
//! # fn f(world: &mut World) -> Result<(), bevy_society::RestoreError> {
//! let save: SocietySave = capture(world);
//! let json = serde_json::to_string(&save).expect("a save is plain data");
//! // … later, in a world with the host's persons spawned …
//! let save: SocietySave = serde_json::from_str(&json).expect("the file is a save");
//! let report = restore(world, save)?;
//! println!("{} person(s) came back fresh", report.fresh.len());
//! # Ok(())
//! # }
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::{Has, QueryData, With};
use bevy::ecs::world::World;
use bevy::log::warn;
use bevy::math::Vec2;
use rand_chacha::rand_core::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::behavior::{Destination, Doing, DoingTarget};
use crate::beliefs::{Knowledge, Opinions};
use crate::core::{
    Dormant, NextPersonId, Person, PersonId, PersonIndex, PersonKey, PlaceKey, SocietyClock,
    SocietyRng, SocietyTick,
};
use crate::defs::{ActivityId, DeedId, InteractionId, SocietyDefs};
use crate::economy::Prices;
use crate::groups::{FactionStats, Memberships, Status};
use crate::mind::Signals;
use crate::mind::{EmotionKind, Emotions, Memories, MemoryKind, Mood, Needs, Personality};
use crate::relationships::{Tie, Ties};
use crate::rules::{LastDays, NeedArmed, RumorReported, TriggerState};
use crate::rumor::{Rumor, RumorBook, RumorId};

/// Version of the [`SocietySave`] format this build writes and reads.
///
/// [`restore`] reads this version and the older ones [`SocietySave::upgrade`] knows a lossless step
/// for (currently 1). It refuses any other version rather than guessing at a layout, so a host that
/// ships a save and later upgrades the crate gets a `Version` error it can act on instead of a
/// silently wrong society.
pub const VERSION: u32 = 4;

// -------------------------------------------------------------------------------------------------
// The saved shape: plain fields, no `Entity`
// -------------------------------------------------------------------------------------------------

/// The names of every definition list, in definition order.
///
/// Ids are indices into these lists, which is why the ids alone cannot survive a defs file that
/// gained, lost or reordered a definition. [`restore`] compares this snapshot with the defs in the
/// world and refuses a mismatch instead of writing person 3's needs into person 3's opinions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamesSave {
    /// Need names, in definition order.
    pub needs: Vec<String>,
    /// Opinion-topic names, in definition order.
    pub topics: Vec<String>,
    /// Deed names, in definition order.
    pub deeds: Vec<String>,
    /// Faction names, in definition order.
    pub factions: Vec<String>,
    /// Activity names, in definition order.
    pub activities: Vec<String>,
}

/// The society clock and the tick accumulator, as they stood at capture.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ClockSave {
    /// Number of society ticks run so far.
    pub tick: u64,
    /// Game time in days.
    pub now_days: f64,
    /// Length of one society tick in game-days.
    pub dt_days: f32,
    /// Fixed-step seconds not yet spent on a tick.
    pub accumulator: f32,
    /// Whether a society tick was due in the step the save was taken.
    pub due: bool,
}

/// The one RNG's exact position: its seed, its stream and its offset in words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngSave {
    /// The 32-byte seed.
    pub seed: [u8; 32],
    /// The stream number.
    pub stream: u64,
    /// The offset from the start of the stream, in 32-bit words.
    pub word_pos: u128,
}

/// One active emotion, with the person it is about as a [`PersonKey`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmotionSave {
    /// Which emotion.
    pub kind: EmotionKind,
    /// Intensity in `[0, 1]`.
    pub intensity: f32,
    /// The person the emotion is about, when it is about one.
    pub toward: Option<u64>,
}

/// One directed tie, with its target as a [`PersonKey`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TieSave {
    /// The person this edge points at.
    pub target: u64,
    /// Liking in `[-1, 1]`.
    pub affinity: f32,
    /// Reliance in `[0, 1]`.
    pub trust: f32,
    /// Conferred esteem in `[-1, 1]`.
    pub respect: f32,
    /// Familiarity peak in `[0, 1]`.
    pub familiarity_peak: f32,
    /// Game-day of the last contact.
    pub last_contact_day: f32,
    /// The label as of the last refresh.
    pub label: crate::relationships::TieLabel,
}

/// What a memory is about, with every person reference as a [`PersonKey`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryKindSave {
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
        /// Who did it, when that person is still keyed.
        actor: Option<u64>,
    },
    /// A deed done to this person. `VERSION` 4.
    Suffered {
        /// The deed definition.
        deed: DeedId,
        /// Who did it, when that person is still keyed.
        actor: Option<u64>,
    },
    /// A rumor heard from someone.
    HeardRumor(RumorId),
    /// A conversation with the player.
    Talked,
    /// A free-form memory from a script.
    Custom(String),
}

/// One remembered event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySave {
    /// The game-day it happened.
    pub day: f32,
    /// What happened.
    pub kind: MemoryKindSave,
    /// The other person involved, by key.
    pub other: Option<u64>,
    /// Good (+) or bad (−) for the rememberer, in `[-1, 1]`.
    pub valence: f32,
    /// How much the memory matters, in `[0, 1]`.
    pub importance: f32,
}

/// What a [`Doing`] is directed at, with places and persons as host-stable keys.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DoingTargetSave {
    /// The activity needs no place or person.
    Itself,
    /// A point on the society plane.
    Point([f32; 2]),
    /// A keyed place.
    Place(u64),
    /// A keyed person.
    Person(u64),
}

/// The activity a person was performing at capture.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DoingSave {
    /// The activity definition.
    pub activity: ActivityId,
    /// What it is directed at.
    pub target: DoingTargetSave,
    /// Game-day the activity started, as its IEEE-754 bits.
    ///
    /// **Bits, not the number.** `serde_json` writes an `f64` with the shortest representation that
    /// round-trips, and for a few day values that shortest form reads back one ULP off — measured:
    /// `0.3906250029879933` came back as `0.3906250029879932`. A save must survive its own file format
    /// exactly, so the values a caller compares are stored as bits. `started_day()` and
    /// `arrived_day()` turn them back.
    pub started_day_bits: u64,
    /// Game-day the person reached the target as its IEEE-754 bits, or `None` while travelling.
    pub arrived_day_bits: Option<u64>,
}

impl DoingSave {
    /// The game-day the activity started.
    pub fn started_day(&self) -> f64 {
        f64::from_bits(self.started_day_bits)
    }

    /// The game-day the person reached the target, or `None` while travelling.
    pub fn arrived_day(&self) -> Option<f64> {
        self.arrived_day_bits.map(f64::from_bits)
    }
}

/// Everything the crate keeps per person, as plain fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonSave {
    /// The host's stable id for this person.
    pub key: u64,
    /// The crate's session id, restored as saved so `PersonIndex` and every script binding line up.
    pub person_id: u32,
    /// The five traits.
    pub personality: Personality,
    /// Pleasure, arousal and dominance.
    pub mood: Mood,
    /// Active emotions.
    pub emotions: Vec<EmotionSave>,
    /// Need levels.
    pub needs: Needs,
    /// Outgoing ties, dropped from the sort order they will be written back in.
    pub ties: Vec<TieSave>,
    /// Opinions on every topic.
    pub opinions: Opinions,
    /// What the person knows about rumors.
    pub knowledge: Knowledge,
    /// Memories, in recording order.
    pub memories: Vec<MemorySave>,
    /// Faction memberships.
    pub memberships: Memberships,
    /// The host-owned signal values, one per declared signal.
    pub signals: Signals,
    /// Standing.
    pub status: Status,
    /// Whether the person is out of their daily life.
    pub dormant: bool,
    /// Whether the person is dead. `Deceased` requires `Dormant`, so both are stored: the marker is
    /// re-inserted as it was, and `dormant` alone would silently revive a person on a reload.
    pub deceased: bool,
    /// The running activity, when its target could be keyed.
    pub doing: Option<DoingSave>,
    /// The walk target, when the person was walking.
    pub destination: Option<[f32; 2]>,
    /// The last ORCA velocity, when the steering feature is on.
    #[cfg(feature = "steering")]
    pub velocity: Option<[f32; 2]>,
    /// Goods held, one level per good in definition order. `VERSION` 2.
    #[serde(default)]
    pub stock: Vec<f32>,
    /// The game-day an [`Away`](crate::Away) absence began, as `f64` bits, or `None` for a person who is
    /// not away. The clock is restored with the people, so the absence keeps counting from before the
    /// save. `VERSION` 3.
    #[serde(default)]
    pub away_since_bits: Option<u64>,
    /// People this person has just harmed and the game-day each may be harmed again (as `f64` bits),
    /// in the order they were written: `Tuning::retaliation_refractory_hours`. Absent from an older file,
    /// which restores as no refractory running, the state it was saved without. A target without a
    /// `PersonKey` is left out.
    #[serde(default)]
    pub recent_harm: Vec<(u64, u64)>,
    /// Places this person recently failed to reach and the game-day each is forgiven (as `f64` bits), in
    /// the order they were written: `Tuning::failed_place_memory_hours`. Absent from an older file, which
    /// restores as nothing avoided. A place without a `PlaceKey` is left out.
    #[serde(default)]
    pub failed_places: Vec<(u64, u64)>,
    /// How close a walker had come to its target and the game-day it last got closer, as `f32` and `f64`
    /// bits: the stall window of `Tuning::stall_timeout_hours`. Absent from an older file, which restores
    /// as a fresh window, the state it was saved without. `VERSION` 4.
    #[serde(default)]
    pub travel_progress: Option<(u32, u64)>,
}

/// One rumor, with every person reference as a [`PersonKey`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RumorSave {
    /// The deed the rumor reports.
    pub deed: DeedId,
    /// The person the rumor is about.
    pub subject: u64,
    /// The person the deed was done to, when there was one.
    pub patient: Option<u64>,
    /// The person who started the rumor.
    pub origin: u64,
    /// Game-day the rumor was created.
    pub day: f32,
    /// Whether the rumor reports what really happened.
    pub truth: bool,
    /// Emotional charge in `[-1, 1]`.
    pub valence: f32,
    /// How many persons have heard it.
    pub known_by: u32,
    /// Whether the rumor still circulates.
    pub alive: bool,
}

/// The trigger bookkeeping: which triggers have fired once, which are cooling, what the clock's
/// previous instant was, which need thresholds are armed, and which (deed, rumor) reach triggers
/// have already fired.
///
/// A once-trigger that fired before the save must not fire again after it, and a cooling trigger
/// must not become available because the reload forgot when it last fired.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TriggersSave {
    /// `(trigger index, actor id)` pairs that have fired once.
    pub fired: Vec<(u32, i64)>,
    /// `(trigger index, actor id, last firing day)` for every cooling trigger.
    pub cooling: Vec<(u32, i64, f64)>,
    /// The clock's instant at the previous tick, which `Hour` triggers measure against.
    pub last_days: Option<f64>,
    /// Which need thresholds are armed, indexed by need and then by definition order.
    pub need_armed: Vec<Vec<bool>>,
    /// The `(deed id, rumor id)` pairs whose reach trigger has fired.
    pub rumor_reported: Vec<(u16, u32)>,
}

/// Every social grid's contents: the crowd grid's channels, then one entry per faction grid.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FieldsSave {
    /// The crowd grid's channels, in channel order.
    pub crowd: Vec<Vec<f32>>,
    /// One entry per faction grid, each holding that grid's channels.
    pub factions: Vec<Vec<Vec<f32>>>,
}

/// A whole society, as plain data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocietySave {
    /// Format version. See [`VERSION`].
    pub version: u32,
    /// The defs' names, which the ids in this save are indices into.
    pub names: NamesSave,
    /// The clock and the tick accumulator.
    pub clock: ClockSave,
    /// The RNG's position.
    pub rng: RngSave,
    /// Every keyed person, in `PersonIndex` order.
    pub persons: Vec<PersonSave>,
    /// Every rumor, in book order, so `RumorId`s stay stable.
    pub rumors: Vec<RumorSave>,
    /// The trigger bookkeeping.
    pub triggers: TriggersSave,
    /// The faction aggregates.
    pub faction_stats: FactionStats,
    /// [`Prices`] as `f32` bits, one per good, empty when the app has no `EconomyPlugin` (or the save
    /// is older than this field). Derived from the stores every tick, but a tick's snapshot (what
    /// `ctx.price` reads) is frozen *before* the price update, so a restore that left the old
    /// book in place made the first tick after it read a price the uninterrupted run did not.
    #[serde(default)]
    pub prices: Vec<u32>,
    /// The social grids, when the `fields` feature is on and the plugin is in the app.
    #[cfg(feature = "fields")]
    pub fields: Option<FieldsSave>,
}

impl SocietySave {
    /// Brings an older save up to [`VERSION`], so a host can migrate a file once and re-save it.
    ///
    /// [`restore`] calls this itself before it validates anything.
    ///
    /// - **Version 1 to 2.** The only difference is [`PersonSave::stock`], which a version-1 file
    ///   does not have and which deserializes as empty. An empty `stock` restores as *holding
    ///   nothing* of every good the defs declare (`Stock::get` is `0.0` for a good past the end of
    ///   the list), the same as a person who never produced anything, and a defs file that gained
    ///   goods since the save needs no entry for them. People who are `Dormant` or `Deceased` upgrade
    ///   exactly as the living do.
    ///
    /// - **Version 3 to 4.** The only difference is the `Suffered` memory kind, which no older file
    ///   holds; nothing needs rewriting.
    ///
    /// A save already at [`VERSION`] is returned unchanged.
    ///
    /// # Errors
    ///
    /// [`RestoreError::Version`] for a version newer than [`VERSION`], and for an older one this
    /// build has no step for (0).
    pub fn upgrade(mut self) -> Result<Self, RestoreError> {
        if self.version == 1 {
            self.version = 2;
        }
        if self.version == 2 {
            // `away_since_bits` is absent from a version-2 file and deserializes as `None`: nobody
            // was away, because the marker did not exist.
            self.version = 3;
        }
        if self.version == 3 {
            // Version 4 only adds `MemoryKindSave::Suffered`; no version-3 file contains one.
            self.version = 4;
        }
        if self.version != VERSION {
            return Err(RestoreError::Version {
                found: self.version,
                expected: VERSION,
            });
        }
        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------
// Errors and the report
// -------------------------------------------------------------------------------------------------

/// Why a [`SocietySave`] could not be applied. The world is untouched when one of these returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreError {
    /// The save was written by a different format version.
    Version {
        /// The version in the save.
        found: u32,
        /// The version this build writes and reads ([`VERSION`]).
        expected: u32,
    },
    /// One of the save's definition-name lists is not the list the defs now declare.
    DefsMismatch {
        /// Which list: `"needs"`, `"topics"`, `"deeds"`, `"factions"` or `"activities"`.
        kind: &'static str,
    },
    /// The world has no [`SocietyDefs`] or no [`SocietyClock`] yet, so there is nothing to restore
    /// into. Run the restore after the core plugin's startup has run.
    NotReady,
}

impl fmt::Display for RestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version { found, expected } => write!(
                f,
                "the save is version {found} but this build reads version {expected}"
            ),
            Self::DefsMismatch { kind } => write!(
                f,
                "the save's {kind} names are not the names the definitions declare"
            ),
            Self::NotReady => f.write_str(
                "the world is not ready for a restore: it needs SocietyDefs and SocietyClock",
            ),
        }
    }
}

impl std::error::Error for RestoreError {}

/// What [`restore`] did beyond the people it restored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RestoreReport {
    /// Keys in the save with no person in the world. Each is also warned about; the save's state
    /// for them is dropped.
    pub unknown_keys: Vec<u64>,
    /// Persons in the world with no entry in the save. They keep the state the host spawned them
    /// with and were given new ids after the restored ones.
    pub fresh: Vec<Entity>,
    /// Activities that could not be restored, because their place or person key did not resolve, or
    /// their activity id is not in the defs.
    pub dropped_activities: usize,
    /// The version the save was written at, when [`SocietySave::upgrade`] had to bring it up to
    /// [`VERSION`] first; `None` for a save that was already current.
    pub upgraded_from: Option<u32>,
}

// -------------------------------------------------------------------------------------------------
// Capture
// -------------------------------------------------------------------------------------------------

/// Every component `Person` requires, plus the optional movement state.
#[derive(QueryData)]
struct PersonCapture {
    entity: Entity,
    id: &'static PersonId,
    key: &'static PersonKey,
    personality: &'static Personality,
    mood: &'static Mood,
    emotions: &'static Emotions,
    needs: &'static Needs,
    ties: &'static Ties,
    opinions: &'static Opinions,
    knowledge: &'static Knowledge,
    memories: &'static Memories,
    memberships: &'static Memberships,
    status: &'static Status,
    dormant: Has<Dormant>,
    deceased: Has<crate::Deceased>,
    away: Option<&'static crate::AwaySince>,
    harmed: Option<&'static crate::behavior::RecentHarm>,
    failed: Option<&'static crate::behavior::FailedPlaces>,
    signals: &'static Signals,
    doing: Option<&'static Doing>,
    destination: Option<&'static Destination>,
    #[cfg(feature = "steering")]
    velocity: Option<&'static crate::steering::SocialVelocity>,
    /// Goods held, when the pack defines any.
    stock: Option<&'static crate::Stock>,
    progress: Option<&'static crate::behavior::TravelProgress>,
}

/// Entity to [`PlaceKey`], for the places the host keyed.
fn place_keys(world: &mut World) -> BTreeMap<Entity, u64> {
    let mut state = world.query_filtered::<(Entity, &PlaceKey), ()>();
    state
        .iter(world)
        .map(|(entity, key)| (entity, key.0))
        .collect()
}

/// `(key, entity)` for every keyed person, ascending by key and then by entity bits.
fn keyed_persons(world: &mut World) -> Vec<(u64, Entity)> {
    let mut state = world.query_filtered::<(Entity, &PersonKey), With<Person>>();
    let mut pairs: Vec<(u64, Entity)> = state
        .iter(world)
        .map(|(entity, key)| (key.0, entity))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.to_bits().cmp(&b.1.to_bits())));
    pairs
}

/// Reads the whole society into a [`SocietySave`].
///
/// **The world is not mutated.** It takes `&mut World` only because building a query needs it.
///
/// A person without a [`PersonKey`] cannot be addressed after a reload, so they are left out, and
/// one `warn!` per call names how many. Every reference *to* such a person is handled the same way:
/// a tie to them is dropped, an emotion or memory about them keeps no target, and an activity
/// directed at them is saved as no activity at all.
pub fn capture(world: &mut World) -> SocietySave {
    let defs = world
        .get_resource::<SocietyDefs>()
        .cloned()
        .unwrap_or_default();
    let clock = world
        .get_resource::<SocietyClock>()
        .copied()
        .unwrap_or_default();
    let tick = world
        .get_resource::<SocietyTick>()
        .copied()
        .unwrap_or_default();
    let rng = world.get_resource::<SocietyRng>().map_or(
        RngSave {
            seed: [0; 32],
            stream: 0,
            word_pos: 0,
        },
        |rng| RngSave {
            seed: rng.0.get_seed(),
            stream: rng.0.get_stream(),
            word_pos: rng.0.get_word_pos(),
        },
    );

    // The key of every person, so a reference to one can be written as a key. Sorted by key, then
    // by entity bits, so a world where two persons share a key still captures deterministically.
    let mut person_keys: BTreeMap<Entity, u64> = BTreeMap::new();
    let mut key_owner: BTreeMap<u64, Entity> = BTreeMap::new();
    for (key, entity) in keyed_persons(world) {
        if let Some(&owner) = key_owner.get(&key) {
            warn!(
                "bevy_society: two persons share the key {key}; the save addresses {owner:?} with \
                 it and cannot address {entity:?}"
            );
            continue;
        }
        key_owner.insert(key, entity);
        person_keys.insert(entity, key);
    }
    let place_keys = place_keys(world);

    // Persons, in `PersonId` order so the save is a function of the society and not of the world's
    // archetype layout.
    let mut counting = world.query_filtered::<Entity, With<Person>>();
    let total = counting.iter(world).count();
    let mut rows: Vec<(u32, u64, Entity, PersonSave)> = Vec::new();
    let mut state = world.query_filtered::<PersonCapture, With<Person>>();
    for person in state.iter(world) {
        // A person the host never keyed is not addressable after a reload, so it is left out and
        // counted below.
        let Some(&key) = person_keys.get(&person.entity) else {
            continue;
        };
        let mut ties: Vec<TieSave> = person
            .ties
            .0
            .iter()
            .filter_map(|tie| {
                let target = *person_keys.get(&tie.target)?;
                Some(TieSave {
                    target,
                    affinity: tie.affinity,
                    trust: tie.trust,
                    respect: tie.respect,
                    familiarity_peak: tie.familiarity_peak,
                    last_contact_day: tie.last_contact_day,
                    label: tie.label,
                })
            })
            .collect();
        ties.sort_by_key(|tie| tie.target);
        let emotions = person
            .emotions
            .0
            .iter()
            .map(|emotion| EmotionSave {
                kind: emotion.kind,
                intensity: emotion.intensity,
                toward: emotion.toward.and_then(|e| person_keys.get(&e).copied()),
            })
            .collect();
        let memories = person
            .memories
            .entries
            .iter()
            .map(|memory| MemorySave {
                day: memory.day,
                kind: match &memory.kind {
                    MemoryKind::Interaction {
                        id,
                        accepted,
                        initiated,
                    } => MemoryKindSave::Interaction {
                        id: *id,
                        accepted: *accepted,
                        initiated: *initiated,
                    },
                    MemoryKind::Witnessed { deed, actor } => MemoryKindSave::Witnessed {
                        deed: *deed,
                        actor: person_keys.get(actor).copied(),
                    },
                    MemoryKind::Suffered { deed, actor } => MemoryKindSave::Suffered {
                        deed: *deed,
                        actor: person_keys.get(actor).copied(),
                    },
                    MemoryKind::HeardRumor(rumor) => MemoryKindSave::HeardRumor(*rumor),
                    MemoryKind::Talked => MemoryKindSave::Talked,
                    MemoryKind::Custom(text) => MemoryKindSave::Custom(text.clone()),
                },
                other: memory.other.and_then(|e| person_keys.get(&e).copied()),
                valence: memory.valence,
                importance: memory.importance,
            })
            .collect();
        let doing = person.doing.and_then(|doing| {
            let target = match doing.target {
                DoingTarget::Itself => DoingTargetSave::Itself,
                DoingTarget::Point(point) => DoingTargetSave::Point([point.x, point.y]),
                DoingTarget::Place(place) => DoingTargetSave::Place(*place_keys.get(&place)?),
                DoingTarget::Person(other) => DoingTargetSave::Person(*person_keys.get(&other)?),
            };
            Some(DoingSave {
                activity: doing.activity,
                target,
                started_day_bits: doing.started_day.to_bits(),
                arrived_day_bits: doing.arrived_day.map(f64::to_bits),
            })
        });
        let save = PersonSave {
            key,
            person_id: person.id.0,
            personality: *person.personality,
            mood: *person.mood,
            emotions,
            needs: person.needs.clone(),
            ties,
            opinions: person.opinions.clone(),
            knowledge: person.knowledge.clone(),
            memories,
            memberships: person.memberships.clone(),
            status: *person.status,
            dormant: person.dormant,
            deceased: person.deceased,
            away_since_bits: person.away.map(|since| since.0.to_bits()),
            recent_harm: person.harmed.map_or_else(Vec::new, |harmed| {
                harmed
                    .0
                    .iter()
                    .filter_map(|(other, until)| Some((*person_keys.get(other)?, until.to_bits())))
                    .collect()
            }),
            failed_places: person.failed.map_or_else(Vec::new, |failed| {
                failed
                    .0
                    .iter()
                    .filter_map(|(place, until)| Some((*place_keys.get(place)?, until.to_bits())))
                    .collect()
            }),
            travel_progress: person
                .progress
                .map(|p| (p.best.to_bits(), p.improved_day.to_bits())),
            signals: person.signals.clone(),
            doing,
            destination: person
                .destination
                .map(|destination| [destination.0.x, destination.0.y]),
            #[cfg(feature = "steering")]
            velocity: person.velocity.map(|velocity| [velocity.0.x, velocity.0.y]),
            stock: person.stock.map_or_else(Vec::new, |stock| stock.0.clone()),
        };
        rows.push((person.id.0, key, person.entity, save));
    }
    drop(state);
    rows.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.to_bits().cmp(&b.2.to_bits()))
    });

    // Every person the query skipped, which is an unkeyed person or one the host removed a required
    // component from. One warning names the count rather than one per person.
    let skipped = total.saturating_sub(rows.len());
    if skipped > 0 {
        warn!(
            "bevy_society: {skipped} person(s) were not saved: they have no PersonKey, or they are \
             missing a component Person requires. A saved society cannot address them after a reload."
        );
    }

    let rumors = world
        .get_resource::<RumorBook>()
        .map(|book| {
            book.rumors
                .iter()
                .map(|rumor| RumorSave {
                    deed: rumor.deed,
                    subject: person_keys.get(&rumor.subject).copied().unwrap_or(u64::MAX),
                    patient: rumor.patient.and_then(|e| person_keys.get(&e).copied()),
                    origin: person_keys.get(&rumor.origin).copied().unwrap_or(u64::MAX),
                    day: rumor.day,
                    truth: rumor.truth,
                    valence: rumor.valence,
                    known_by: rumor.known_by,
                    alive: rumor.alive,
                })
                .collect()
        })
        .unwrap_or_default();

    SocietySave {
        version: VERSION,
        names: names_of(&defs),
        clock: ClockSave {
            tick: clock.tick,
            now_days: clock.now_days,
            dt_days: clock.dt_days,
            accumulator: tick.accumulator,
            due: tick.due,
        },
        rng,
        persons: rows.into_iter().map(|(_, _, _, save)| save).collect(),
        rumors,
        triggers: capture_triggers(world),
        faction_stats: world
            .get_resource::<FactionStats>()
            .cloned()
            .unwrap_or_default(),
        prices: world
            .get_resource::<Prices>()
            .map(|prices| prices.0.iter().map(|price| price.to_bits()).collect())
            .unwrap_or_default(),
        #[cfg(feature = "fields")]
        fields: capture_fields(world),
    }
}

/// The defs' names, in definition order.
fn names_of(defs: &SocietyDefs) -> NamesSave {
    NamesSave {
        needs: defs.needs.iter().map(|spec| spec.name.clone()).collect(),
        topics: defs.topics.iter().map(|spec| spec.name.clone()).collect(),
        deeds: defs.deeds.iter().map(|spec| spec.name.clone()).collect(),
        factions: defs.factions.iter().map(|spec| spec.name.clone()).collect(),
        activities: defs
            .activities
            .iter()
            .map(|spec| spec.name.clone())
            .collect(),
    }
}

/// The trigger bookkeeping, as plain vectors so a JSON map never needs a tuple key.
fn capture_triggers(world: &World) -> TriggersSave {
    let mut save = TriggersSave::default();
    if let Some(state) = world.get_resource::<TriggerState>() {
        save.fired = state
            .fired_set()
            .iter()
            .map(|&(index, actor)| (index as u32, actor))
            .collect();
        save.cooling = state
            .cooling_map()
            .iter()
            .map(|(&(index, actor), day)| (index as u32, actor, *day))
            .collect();
    }
    if let Some(last) = world.get_resource::<LastDays>() {
        save.last_days = last.0;
    }
    if let Some(armed) = world.get_resource::<NeedArmed>() {
        save.need_armed = armed.0.clone();
    }
    if let Some(reported) = world.get_resource::<RumorReported>() {
        save.rumor_reported = reported
            .0
            .iter()
            .map(|&(deed, rumor)| (deed, rumor))
            .collect();
    }
    save
}

/// Every social grid's channels, when the fields feature and plugin are in the app.
#[cfg(feature = "fields")]
fn capture_fields(world: &World) -> Option<FieldsSave> {
    let fields = world.get_resource::<crate::fields::SocialFields>()?;
    Some(FieldsSave {
        crowd: fields.crowd.channels().to_vec(),
        factions: fields
            .factions
            .iter()
            .map(|grid| grid.channels().to_vec())
            .collect(),
    })
}

// -------------------------------------------------------------------------------------------------
// Restore
// -------------------------------------------------------------------------------------------------

/// Applies a [`SocietySave`] to a world the host has already spawned its persons and keyed places
/// into.
///
/// A save from an older version is first brought up with [`SocietySave::upgrade`] and the report says
/// so in `upgraded_from`.
///
/// **Nothing is written until everything is validated**, so a save from an unknown version, or one
/// written against a defs file that has since been edited, leaves the world exactly as it was.
///
/// What comes back:
///
/// - every person in the save whose [`PersonKey`] is in the world gets its saved `PersonId` and its
///   saved components back, with ties re-sorted by target so `Ties`' binary search still holds;
/// - a person in the world with no entry in the save keeps the state the host spawned them with and
///   is given a new id after the highest restored one, in its current id order;
/// - a rumor that named a person who is no longer keyed keeps `Entity::PLACEHOLDER` there and stops
///   circulating, because nothing can spread a rumor about somebody the world does not have;
/// - the clock, the tick accumulator, the RNG's exact position and every trigger's
///   once/cooldown bookkeeping are set back to what the save recorded, which is what makes the
///   ticks after a reload identical to the ticks that would have followed without one.
///
/// # Errors
///
/// See [`RestoreError`]. Each is returned before anything is written.
pub fn restore(world: &mut World, save: SocietySave) -> Result<RestoreReport, RestoreError> {
    // --- validate -------------------------------------------------------------------------------
    let written_at = save.version;
    let save = save.upgrade()?;
    let Some(defs) = world.get_resource::<SocietyDefs>().cloned() else {
        return Err(RestoreError::NotReady);
    };
    let names = names_of(&defs);
    for (kind, saved, current) in [
        ("needs", &save.names.needs, &names.needs),
        ("topics", &save.names.topics, &names.topics),
        ("deeds", &save.names.deeds, &names.deeds),
        ("factions", &save.names.factions, &names.factions),
        ("activities", &save.names.activities, &names.activities),
    ] {
        if saved != current {
            return Err(RestoreError::DefsMismatch { kind });
        }
    }
    if world.get_resource::<SocietyClock>().is_none() {
        return Err(RestoreError::NotReady);
    }

    // --- resolve keys to entities ----------------------------------------------------------------
    let mut person_by_key: BTreeMap<u64, Entity> = BTreeMap::new();
    // Deterministic when two persons share a key: the lowest entity bits win.
    for (key, entity) in keyed_persons(world) {
        if person_by_key.insert(key, entity).is_some() {
            warn!(
                "bevy_society: two persons share the key {key}; the restore uses the first and \
                 leaves the second fresh"
            );
        }
    }
    let place_by_key: BTreeMap<u64, Entity> = {
        let mut state = world.query_filtered::<(Entity, &PlaceKey), ()>();
        let mut pairs: Vec<(u64, Entity)> = state.iter(world).map(|(e, k)| (k.0, e)).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.to_bits().cmp(&b.1.to_bits())));
        let mut map = BTreeMap::new();
        for (key, entity) in pairs {
            if map.insert(key, entity).is_some() {
                warn!("bevy_society: two places share the key {key}; the restore uses the first");
            }
        }
        map
    };

    let mut report = RestoreReport {
        upgraded_from: (written_at != VERSION).then_some(written_at),
        ..RestoreReport::default()
    };
    let mut unknown: BTreeSet<u64> = BTreeSet::new();
    for person in &save.persons {
        if !person_by_key.contains_key(&person.key) {
            unknown.insert(person.key);
        }
    }
    for key in &unknown {
        warn!(
            "bevy_society: the save has a person keyed {key} with no person in the world; its state \
             was skipped"
        );
    }
    report.unknown_keys = unknown.into_iter().collect();

    // --- write the restored persons --------------------------------------------------------------
    let mut used_ids: BTreeSet<u32> = BTreeSet::new();
    let mut restored: BTreeSet<Entity> = BTreeSet::new();
    let mut fresh_entities: Vec<Entity> = Vec::new();
    for person in &save.persons {
        let Some(&entity) = person_by_key.get(&person.key) else {
            continue;
        };
        if !used_ids.insert(person.person_id) {
            warn!(
                "bevy_society: the save gives id {} to more than one person; the second is left fresh",
                person.person_id
            );
            fresh_entities.push(entity);
            continue;
        }
        restored.insert(entity);
        write_person(
            world,
            entity,
            person,
            &person_by_key,
            &place_by_key,
            &defs,
            &mut report,
        );
    }

    // --- persons the save does not know ----------------------------------------------------------
    // Every person in the world, keyed or not: an unkeyed person is exactly the case the host most
    // needs told about, and it is not in `person_by_key`.
    let mut all_persons: Vec<Entity> = {
        let mut state = world.query_filtered::<Entity, With<Person>>();
        state.iter(world).collect()
    };
    all_persons.sort_by_key(|entity| {
        let id = world.get::<PersonId>(*entity).map_or(u32::MAX, |id| id.0);
        (id, entity.to_bits())
    });
    for entity in all_persons {
        if !restored.contains(&entity) && !fresh_entities.contains(&entity) {
            warn!(
                "bevy_society: {entity:?} is a person the save does not know, so it keeps the state \
                 the host spawned it with and is given a new id"
            );
            fresh_entities.push(entity);
        }
    }
    let next_id = used_ids
        .last()
        .map_or(0, |highest| highest.saturating_add(1));
    let mut next = next_id;
    for &entity in &fresh_entities {
        while used_ids.contains(&next) {
            next = next.saturating_add(1);
        }
        used_ids.insert(next);
        if let Ok(mut em) = world.get_entity_mut(entity) {
            em.insert(PersonId(next));
        }
        next = next.saturating_add(1);
    }
    report.fresh = fresh_entities;

    // --- rebuild the index ----------------------------------------------------------------------
    // **Every live person**, not only the keyed ones: the index is what every simulation loop walks,
    // so a person left out of it would stop taking part in the society entirely. The id just written
    // to each of them is the order.
    let mut ordered: Vec<(PersonId, Entity)> = {
        let mut state = world.query_filtered::<(Entity, &PersonId), With<Person>>();
        state
            .iter(world)
            .map(|(entity, id)| (*id, entity))
            .collect()
    };
    ordered.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.to_bits().cmp(&b.1.to_bits())));
    if let Some(index) = world.get_resource_mut::<PersonIndex>() {
        let mut index = index;
        index.sorted = ordered.clone();
        index.by_id.clear();
        for &(id, entity) in &ordered {
            index.by_id.insert(id, entity);
        }
    }
    if let Some(mut next_person) = world.get_resource_mut::<NextPersonId>() {
        next_person.0 = used_ids
            .last()
            .map_or(0, |highest| highest.saturating_add(1));
    }

    // --- rumors ---------------------------------------------------------------------------------
    if let Some(mut book) = world.get_resource_mut::<RumorBook>() {
        let mut rumors = Vec::with_capacity(save.rumors.len());
        for rumor in &save.rumors {
            let subject = person_by_key.get(&rumor.subject).copied();
            let origin = person_by_key.get(&rumor.origin).copied();
            let patient = rumor
                .patient
                .and_then(|key| person_by_key.get(&key).copied());
            let resolved = subject.is_some()
                && origin.is_some()
                && patient.is_some() == rumor.patient.is_some();
            rumors.push(Rumor {
                deed: rumor.deed,
                subject: subject.unwrap_or(Entity::PLACEHOLDER),
                patient,
                origin: origin.unwrap_or(Entity::PLACEHOLDER),
                day: rumor.day,
                truth: rumor.truth,
                valence: rumor.valence,
                known_by: rumor.known_by,
                alive: rumor.alive && resolved,
            });
        }
        book.rumors = rumors;
    }

    // --- triggers -------------------------------------------------------------------------------
    if let Some(mut state) = world.get_resource_mut::<TriggerState>() {
        let fired: BTreeSet<(usize, i64)> = save
            .triggers
            .fired
            .iter()
            .map(|&(index, actor)| (index as usize, actor))
            .collect();
        let cooling: BTreeMap<(usize, i64), f64> = save
            .triggers
            .cooling
            .iter()
            .map(|&(index, actor, day)| ((index as usize, actor), day))
            .collect();
        state.replace(fired, cooling);
    }
    if let Some(mut last) = world.get_resource_mut::<LastDays>() {
        last.0 = save.triggers.last_days;
    }
    if let Some(mut armed) = world.get_resource_mut::<NeedArmed>() {
        armed.0 = save.triggers.need_armed.clone();
    }
    if let Some(mut reported) = world.get_resource_mut::<RumorReported>() {
        reported.0 = save.triggers.rumor_reported.iter().copied().collect();
    }

    // --- clock, tick and rng --------------------------------------------------------------------
    if let Some(mut clock) = world.get_resource_mut::<SocietyClock>() {
        clock.tick = save.clock.tick;
        clock.now_days = save.clock.now_days;
        clock.dt_days = save.clock.dt_days;
    }
    if let Some(mut tick) = world.get_resource_mut::<SocietyTick>() {
        tick.accumulator = save.clock.accumulator;
        tick.due = save.clock.due;
    }
    if let Some(mut rng) = world.get_resource_mut::<SocietyRng>() {
        let mut generator = rand_chacha::ChaCha8Rng::from_seed(save.rng.seed);
        generator.set_stream(save.rng.stream);
        generator.set_word_pos(save.rng.word_pos);
        rng.0 = generator;
    } else {
        warn!("bevy_society: no SocietyRng to restore; the stream continues where it was");
    }

    // --- aggregates and fields ------------------------------------------------------------------
    if let Some(mut stats) = world.get_resource_mut::<FactionStats>() {
        stats.0 = save.faction_stats.0.clone();
    }
    if !save.prices.is_empty()
        && let Some(mut prices) = world.get_resource_mut::<Prices>()
    {
        prices.0 = save
            .prices
            .iter()
            .map(|&bits| f32::from_bits(bits))
            .collect();
    }
    #[cfg(feature = "fields")]
    restore_fields(world, save.fields.as_ref());

    Ok(report)
}

/// Writes one saved person's components back.
fn write_person(
    world: &mut World,
    entity: Entity,
    person: &PersonSave,
    persons: &BTreeMap<u64, Entity>,
    places: &BTreeMap<u64, Entity>,
    defs: &SocietyDefs,
    report: &mut RestoreReport,
) {
    let mut ties: Vec<Tie> = person
        .ties
        .iter()
        .filter_map(|tie| {
            let target = *persons.get(&tie.target)?;
            Some(Tie {
                target,
                affinity: tie.affinity,
                trust: tie.trust,
                respect: tie.respect,
                familiarity_peak: tie.familiarity_peak,
                last_contact_day: tie.last_contact_day,
                label: tie.label,
            })
        })
        .collect();
    ties.sort_by_key(|tie| tie.target.to_bits());
    let emotions = Emotions(
        person
            .emotions
            .iter()
            .map(|emotion| crate::mind::ActiveEmotion {
                kind: emotion.kind,
                intensity: emotion.intensity,
                toward: emotion.toward.and_then(|key| persons.get(&key).copied()),
            })
            .collect(),
    );
    let memories = Memories {
        entries: person
            .memories
            .iter()
            .map(|memory| crate::mind::Memory {
                day: memory.day,
                kind: match &memory.kind {
                    MemoryKindSave::Interaction {
                        id,
                        accepted,
                        initiated,
                    } => MemoryKind::Interaction {
                        id: *id,
                        accepted: *accepted,
                        initiated: *initiated,
                    },
                    MemoryKindSave::Witnessed { deed, actor } => MemoryKind::Witnessed {
                        deed: *deed,
                        actor: actor
                            .and_then(|key| persons.get(&key).copied())
                            .unwrap_or(Entity::PLACEHOLDER),
                    },
                    MemoryKindSave::Suffered { deed, actor } => MemoryKind::Suffered {
                        deed: *deed,
                        actor: actor
                            .and_then(|key| persons.get(&key).copied())
                            .unwrap_or(Entity::PLACEHOLDER),
                    },
                    MemoryKindSave::HeardRumor(rumor) => MemoryKind::HeardRumor(*rumor),
                    MemoryKindSave::Talked => MemoryKind::Talked,
                    MemoryKindSave::Custom(text) => MemoryKind::Custom(text.clone()),
                },
                other: memory.other.and_then(|key| persons.get(&key).copied()),
                valence: memory.valence,
                importance: memory.importance,
            })
            .collect(),
    };
    let doing = person.doing.and_then(|doing| {
        if usize::from(doing.activity.0) >= defs.activities.len() {
            return None;
        }
        let target = match doing.target {
            DoingTargetSave::Itself => DoingTarget::Itself,
            DoingTargetSave::Point([x, y]) => DoingTarget::Point(Vec2::new(x, y)),
            DoingTargetSave::Place(key) => DoingTarget::Place(*places.get(&key)?),
            DoingTargetSave::Person(key) => DoingTarget::Person(*persons.get(&key)?),
        };
        Some(Doing {
            activity: doing.activity,
            target,
            started_day: doing.started_day(),
            arrived_day: doing.arrived_day(),
        })
    });
    if person.doing.is_some() && doing.is_none() {
        report.dropped_activities += 1;
    }

    let Ok(mut em) = world.get_entity_mut(entity) else {
        return;
    };
    em.insert((
        PersonId(person.person_id),
        person.personality,
        person.mood,
        emotions,
        person.needs.clone(),
        Ties(ties),
        person.opinions.clone(),
        person.knowledge.clone(),
        memories,
        person.memberships.clone(),
        person.status,
    ));
    // Goods come back exactly as saved, so a village's stores survive a reload the way its ties do.
    if person.stock.is_empty() {
        em.remove::<crate::Stock>();
    } else {
        em.insert(crate::Stock(person.stock.clone()));
    }
    if person.dormant {
        em.insert(Dormant);
    } else {
        em.remove::<Dormant>();
    }
    if person.deceased {
        em.insert(crate::Deceased);
    } else {
        em.remove::<crate::Deceased>();
    }
    match person.away_since_bits {
        // `AwaySince` in the same bundle as `Away`, so the add observer finds it and keeps it.
        Some(bits) => {
            em.insert((crate::AwaySince(f64::from_bits(bits)), crate::Away));
        }
        None => {
            em.remove::<crate::Away>();
        }
    }
    em.insert(person.signals.clone());
    if person.recent_harm.is_empty() {
        em.remove::<crate::behavior::RecentHarm>();
    } else {
        em.insert(crate::behavior::RecentHarm(
            person
                .recent_harm
                .iter()
                .filter_map(|&(key, until)| Some((*persons.get(&key)?, f64::from_bits(until))))
                .collect(),
        ));
    }
    if person.failed_places.is_empty() {
        em.remove::<crate::behavior::FailedPlaces>();
    } else {
        em.insert(crate::behavior::FailedPlaces(
            person
                .failed_places
                .iter()
                .filter_map(|&(key, until)| Some((*places.get(&key)?, f64::from_bits(until))))
                .collect(),
        ));
    }
    match person.travel_progress {
        Some((best, improved)) => {
            em.insert(crate::behavior::TravelProgress {
                best: f32::from_bits(best),
                improved_day: f64::from_bits(improved),
            });
        }
        None => {
            em.remove::<crate::behavior::TravelProgress>();
        }
    }
    if let Some(doing) = doing {
        em.insert(doing);
    } else {
        em.remove::<Doing>();
    }
    match person.destination {
        Some([x, y]) => {
            em.insert(Destination(Vec2::new(x, y)));
        }
        None => {
            em.remove::<Destination>();
        }
    }
    #[cfg(feature = "steering")]
    match person.velocity {
        Some([x, y]) => {
            em.insert(crate::steering::SocialVelocity(Vec2::new(x, y)));
        }
        None => {
            em.remove::<crate::steering::SocialVelocity>();
        }
    }
}

/// Puts every saved channel back and refreshes the cached peaks the field readers normalise by.
#[cfg(feature = "fields")]
fn restore_fields(world: &mut World, save: Option<&FieldsSave>) {
    let Some(save) = save else {
        return;
    };
    let Some(mut fields) = world.get_resource_mut::<crate::fields::SocialFields>() else {
        warn!(
            "bevy_society: the save has social fields but the world has no SocialFields; skipped"
        );
        return;
    };
    for (channel, values) in save.crowd.iter().enumerate() {
        if !fields.crowd.set_channel(channel, values) {
            warn!("bevy_society: saved crowd channel {channel} does not match the grid; skipped");
        }
    }
    for (index, grid) in save.factions.iter().enumerate() {
        let Some(target) = fields.factions.get_mut(index) else {
            warn!(
                "bevy_society: the save has faction grid {index} but the world has fewer; skipped"
            );
            continue;
        };
        for (channel, values) in grid.iter().enumerate() {
            if !target.set_channel(channel, values) {
                warn!(
                    "bevy_society: saved faction grid {index} channel {channel} does not match; \
                     skipped"
                );
            }
        }
    }
    fields.refresh_peaks();
}

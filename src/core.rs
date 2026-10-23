//! Core: persons, stable ids, the society clock, the one RNG, the system sets and perception.
//!
//! Every other plugin in the crate runs inside the [`SocietySystems`] sets configured here, and
//! every simulation loop walks [`PersonIndex::sorted`], so the order of persons is the order of
//! spawning and a deterministic spawn sequence gives deterministic ids.

use std::collections::HashMap;

use bevy::app::{App, FixedUpdate, Plugin, Startup};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::lifecycle::{Add, Remove};
use bevy::ecs::observer::On;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::ecs::world::World;
use bevy::log::error;
use bevy::math::{Vec2, Vec3};
use bevy::reflect::Reflect;
use bevy::time::{Fixed, Time};
use bevy::transform::components::{GlobalTransform, Transform};
use rand_chacha::ChaCha8Rng;

use crate::beliefs::{Knowledge, Opinions};
use crate::defs::{SocietyDefs, size_person_state};
use crate::groups::{Memberships, Status};
use crate::mind::{Emotions, Memories, Mood, Needs, Personality};
use crate::relationships::Ties;
use crate::rumor::RumorBook;

/// Which two world axes the society lives on. Positions are projected onto these axes.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SocietyPlane {
    /// Ground plane of a 3D world: `(x, z)`.
    #[default]
    XZ,
    /// Side-on 2D world: `(x, y)`.
    XY,
}

impl SocietyPlane {
    /// Projects a world-space position onto the society plane.
    pub fn project(self, p: Vec3) -> Vec2 {
        match self {
            Self::XZ => Vec2::new(p.x, p.z),
            Self::XY => Vec2::new(p.x, p.y),
        }
    }

    /// Lifts a plane position back into world space at height zero (the other axis).
    pub fn lift(self, p: Vec2) -> Vec3 {
        match self {
            Self::XZ => Vec3::new(p.x, 0.0, p.y),
            Self::XY => Vec3::new(p.x, p.y, 0.0),
        }
    }
}

/// How witnesses judge a rule-breaking actor's trust (see the reputation note).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssessmentNorm {
    /// Image scoring (Nowak & Sigmund 1998, DOI 10.1038/31225): trust follows the deed alone.
    ImageScoring,
    /// Standing (Ohtsuki & Iwasa 2004, DOI 10.1016/j.jtbi.2004.06.005): a justified refusal of a
    /// distrusted recipient does not lower the refuser's standing.
    #[default]
    Standing,
}

/// Every model rate of the crate, one named field each. Rates are per game-day unless the name says
/// otherwise.
///
/// A default either comes from a paper, in which case the field's doc says so, or is a design
/// default chosen so a society of a few dozen people visibly moves within a game-day. The research
/// notes record which is which.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Tuning {
    /// Half-life of an emotion's intensity, in game-days (ALMA-style decay; Gebhard 2005,
    /// DOI 10.1145/1082473.1082478).
    pub emotion_half_life_days: f32,
    /// How fast mood is pulled toward the emotion centre, per game-day, scaled by emotion intensity.
    /// Mechanism: ALMA's mood update (Gebhard 2005, DOI 10.1145/1082473.1082478); the baseline it
    /// returns to is the Mehrabian 1996 personality mapping (DOI 10.1080/00049539608259510).
    pub mood_pull_per_day: f32,
    /// How fast mood returns to its personality baseline, per game-day. Mechanism: ALMA's mood
    /// update (Gebhard 2005, DOI 10.1145/1082473.1082478).
    pub mood_return_per_day: f32,
    /// Strength of pleasure contagion from contacts, per game-day (Hatfield 1993,
    /// DOI 10.1111/1467-8721.ep10770953).
    pub contagion_per_day: f32,
    /// Power-law forgetting time scale of familiarity, in game-days (Anderson & Schooler 1991,
    /// DOI 10.1111/j.1467-9280.1991.tb00174.x; Murre & Dros 2015, DOI 10.1371/journal.pone.0120644).
    pub familiarity_tau_days: f32,
    /// Power-law forgetting exponent of familiarity. Design default (no source paper gives this
    /// number); mechanism: Murre & Dros 2015, DOI 10.1371/journal.pone.0120644.
    pub familiarity_decay_exp: f32,
    /// Familiarity gained by contact, per game-day (mere exposure; Zajonc 1968,
    /// DOI 10.1037/h0025848).
    pub exposure_per_day: f32,
    /// Affinity gained by contact, per game-day, scaled by `1 - |affinity|` (Zajonc 1968,
    /// DOI 10.1037/h0025848).
    pub mere_exposure_per_day: f32,
    /// Relaxation of affinity toward zero, per game-day, never crossing zero. Design default (no
    /// source paper gives this number).
    pub affinity_relax_per_day: f32,
    /// Triads sampled per tick for structural balance (Heider 1946, DOI 10.1080/00223980.1946.9917275).
    pub balance_samples_per_tick: usize,
    /// Rate at which a balanced triad pulls the third edge (Cartwright & Harary 1956,
    /// DOI 10.1037/h0046049).
    pub balance_rate_per_day: f32,
    /// Familiarity added to two people by a mutual close friend's introduction. Design default (no
    /// source paper gives this number); mechanism: triadic closure, Kossinets & Watts 2006,
    /// DOI 10.1126/science.1116869.
    pub introduction_familiarity: f32,
    /// Base bounded-confidence threshold. Design default (no source paper gives this number);
    /// mechanism: Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
    pub confidence_base: f32,
    /// Extra confidence per unit of openness. Design default (no source paper gives this number);
    /// mechanism: Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
    pub confidence_openness: f32,
    /// Convergence rate of opinions inside the confidence bound. Design default (no source paper
    /// gives this number); mechanism: Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
    pub influence_mu: f32,
    /// Affinity gained from similar opinions (similarity attraction; Byrne 1961,
    /// DOI 10.1037/h0044721).
    pub similarity_gain: f32,
    /// Pull toward an anchored opinion (Friedkin & Johnsen 1990, DOI 10.1080/0022250X.1990.9990069).
    pub anchor_pull: f32,
    /// Conformity rate toward a strong group majority. Design default (no source paper gives this
    /// number); mechanism: Nowak, Szamrej & Latané 1990, DOI 10.1037/0033-295X.97.3.362; Asch 1956,
    /// DOI 10.1037/h0093718.
    pub conformity_per_day: f32,
    /// Trust gain per unit of deed harm from a witness. Design default (no source paper gives this
    /// number); mechanism: image scoring, Nowak & Sigmund 1998, DOI 10.1038/31225.
    pub witness_gain: f32,
    /// Absolute harm above which a deed becomes a rumor. Design default (no source paper gives this
    /// number); mechanism: gossip as reputation, Sommerfeld et al. 2007, DOI 10.1073/pnas.0704598104.
    pub rumor_threshold: f32,
    /// Trust gained from gossip, per unit of valence times belief. Design default (no source paper
    /// gives this number); mechanism: Dunbar 2004, DOI 10.1037/1089-2680.8.2.100.
    pub gossip_trust_gain: f32,
    /// Affinity gained from gossip, per unit of valence times belief. Design default (no source
    /// paper gives this number); mechanism: Dunbar 2004, DOI 10.1037/1089-2680.8.2.100.
    pub gossip_affinity_gain: f32,
    /// Affinity gained between persons with no shared faction. Design default (no source paper gives
    /// this number); mechanism: intergroup contact, Pettigrew & Tropp 2006,
    /// DOI 10.1037/0022-3514.90.5.751.
    pub contact_gain: f32,
    /// Steepness of the contest logistic. Design default (no source paper gives this number);
    /// mechanism: Bonabeau et al. 1995, DOI 10.1016/0378-4371(95)00064-e.
    pub contest_steepness: f32,
    /// Options below this fraction of the best score are dropped. Design default (no source paper
    /// gives this number); mechanism: dual-utility selection, Dill 2015, DOI 10.1201/b18373-5.
    pub tier_cutoff: f32,
    /// Hours a person may travel toward an activity before giving up.
    pub travel_timeout_hours: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            emotion_half_life_days: 0.1,
            mood_pull_per_day: 4.0,
            mood_return_per_day: 1.0,
            contagion_per_day: 0.3,
            familiarity_tau_days: 1.0,
            familiarity_decay_exp: 0.5,
            exposure_per_day: 2.0,
            mere_exposure_per_day: 0.05,
            affinity_relax_per_day: 0.01,
            balance_samples_per_tick: 8,
            balance_rate_per_day: 0.5,
            introduction_familiarity: 0.1,
            confidence_base: 0.2,
            confidence_openness: 0.6,
            influence_mu: 0.3,
            similarity_gain: 0.05,
            anchor_pull: 0.1,
            conformity_per_day: 0.2,
            witness_gain: 0.2,
            rumor_threshold: 0.3,
            gossip_trust_gain: 0.3,
            gossip_affinity_gain: 0.2,
            contact_gain: 0.02,
            contest_steepness: 4.0,
            tier_cutoff: 0.25,
            travel_timeout_hours: 2.0,
        }
    }
}

/// Global settings of the society. Insert before adding [`SocietyCorePlugin`] to override defaults.
#[derive(Resource, Reflect, Debug, Clone)]
pub struct SocietySettings {
    /// Seed of the one [`SocietyRng`] stream.
    pub seed: u64,
    /// Seconds of fixed-step time per society tick.
    pub tick_secs: f32,
    /// Seconds of real time that make one game-day.
    pub seconds_per_day: f32,
    /// Hour of the game-day at which the clock starts.
    pub start_hour: f32,
    /// Plane on which positions are projected.
    pub plane: SocietyPlane,
    /// Radius within which a person perceives others (metres on the plane).
    pub perception_radius: f32,
    /// Radius within which two persons are in contact (conversation, conformity, exposure).
    pub contact_radius: f32,
    /// Distance at which a person counts as arrived at an interaction target.
    pub interaction_radius: f32,
    /// Maximum ties a person keeps (Dunbar 1992, DOI 10.1016/0047-2484(92)90081-J).
    pub max_ties: usize,
    /// Size of the close-tie layer (Zhou et al. 2005, DOI 10.1098/rspb.2004.2970).
    pub close_ties: usize,
    /// Memories kept per person before eviction. Design default (no source paper gives this
    /// number).
    pub memory_capacity: usize,
    /// Assessment norm used when a witness judges a deed.
    pub assessment: AssessmentNorm,
    /// Probability that an interaction or activity produces a bark.
    pub bark_chance: f32,
    /// Every model rate.
    pub tuning: Tuning,
}

impl Default for SocietySettings {
    fn default() -> Self {
        Self {
            seed: 0x5EED_5EED,
            tick_secs: 0.25,
            seconds_per_day: 240.0,
            start_hour: 8.0,
            plane: SocietyPlane::XZ,
            perception_radius: 8.0,
            contact_radius: 4.0,
            interaction_radius: 1.5,
            max_ties: 150,
            close_ties: 15,
            memory_capacity: 64,
            assessment: AssessmentNorm::Standing,
            bark_chance: 0.35,
            tuning: Tuning::default(),
        }
    }
}

/// The society clock. `now_days` is the continuous game time in days; `dt_days` is the length of
/// one society tick in days.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq)]
pub struct SocietyClock {
    /// Number of society ticks run so far.
    pub tick: u64,
    /// Game time in days.
    pub now_days: f64,
    /// Length of one society tick in game-days.
    pub dt_days: f32,
}

impl SocietyClock {
    /// Hour of the current game-day, in `[0, 24)`.
    pub fn hour(&self) -> f32 {
        (self.now_days.rem_euclid(1.0) * 24.0) as f32
    }

    /// Whole game-days elapsed.
    pub fn day(&self) -> u32 {
        self.now_days.floor() as u32
    }
}

/// Accumulates fixed-step time and says whether a society tick is due this step.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq)]
pub struct SocietyTick {
    /// Fixed-step time not yet spent on a tick, in seconds.
    pub accumulator: f32,
    /// True during the fixed steps in which a society tick runs.
    pub due: bool,
}

/// The only randomness in the crate: a ChaCha8 stream seeded from [`SocietySettings::seed`].
#[derive(Resource, Clone, Debug)]
pub struct SocietyRng(pub ChaCha8Rng);

/// Next [`PersonId`] to hand out. Ids follow spawn order.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NextPersonId(pub u32);

/// Stable, deterministic id of a person, assigned in spawn order.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonId(pub u32);

/// Marker of a simulated person. Adding it gives the person every per-person component.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[require(
    Personality,
    Mood,
    Emotions,
    Needs,
    Ties,
    Opinions,
    Knowledge,
    Memories,
    Memberships,
    Status,
    Transform,
    GlobalTransform
)]
pub struct Person;

/// Marker: this person chooses its own activities. A player-controlled person is a [`Person`]
/// without this marker.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Autonomous;

/// The home of a person, used by the `Home` activity target.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Home(pub Entity);

/// A place people can go to. `capacity` limits how many persons may be doing an activity there.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Place {
    /// Maximum number of persons whose activity targets this place at once.
    pub capacity: u32,
}

/// Tags that let activities find a place (`ActivityTarget::Place(tag)`).
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq, Eq)]
pub struct PlaceTags(pub Vec<String>);

/// Persons in spawn order, with lookup by id. Maintained by the add and remove observers only.
#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct PersonIndex {
    /// Entity of each live person by id (lookup only; never iterated).
    pub by_id: HashMap<PersonId, Entity>,
    /// Live persons in ascending id order. Every simulation loop iterates this.
    pub sorted: Vec<(PersonId, Entity)>,
}

/// Per-tick perception: positions and neighbourhoods, indexed by position in
/// [`PersonIndex::sorted`]. Neighbour lists are ascending by index and exclude the person itself.
#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct Perception {
    /// Entity at each index.
    pub persons: Vec<Entity>,
    /// Plane position at each index.
    pub pos: Vec<Vec2>,
    /// Persons within `perception_radius`, with their distance.
    pub near: Vec<Vec<(u32, f32)>>,
    /// Persons within `contact_radius`.
    pub contact: Vec<Vec<u32>>,
}

/// System sets of one society tick, chained in this order inside `FixedUpdate`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SocietySystems {
    /// Advances the clock. Runs every fixed step.
    Clock,
    /// Builds [`Perception`]. Runs on society ticks only.
    Perceive,
    /// Emotions, mood, needs and memories.
    Mind,
    /// Daily-life decisions.
    Decide,
    /// Social exchanges.
    Interact,
    /// Opinion and rumor propagation.
    Propagate,
    /// Ties, balance, labels, groups and status.
    Relate,
    /// Triggers and scripted effects.
    Rules,
}

/// Run condition: true on fixed steps in which a society tick is due.
pub fn society_tick_due(tick: Res<SocietyTick>) -> bool {
    tick.due
}

/// Advances [`SocietyClock`] by one tick each time enough fixed-step time has accumulated.
fn advance_clock(
    settings: Res<SocietySettings>,
    fixed: Res<Time<Fixed>>,
    mut tick: ResMut<SocietyTick>,
    mut clock: ResMut<SocietyClock>,
) {
    tick.accumulator += fixed.delta_secs();
    if tick.accumulator + 1e-6 >= settings.tick_secs {
        tick.accumulator -= settings.tick_secs;
        tick.due = true;
        clock.tick += 1;
        clock.dt_days = settings.tick_secs / settings.seconds_per_day;
        clock.now_days += f64::from(clock.dt_days);
    } else {
        tick.due = false;
    }
}

/// Seeds the RNG and sets the clock to the configured start hour.
fn setup_society(
    mut commands: Commands,
    settings: Res<SocietySettings>,
    mut clock: ResMut<SocietyClock>,
) {
    commands.insert_resource(SocietyRng(bevy_emerge_rng::seeded(settings.seed)));
    clock.now_days = f64::from(settings.start_hour) / 24.0;
}

/// Gives each new [`Person`] the next id and records it in the index.
fn on_person_added(
    add: On<Add<Person>>,
    mut commands: Commands,
    mut next: ResMut<NextPersonId>,
    mut index: ResMut<PersonIndex>,
) {
    let entity = add.entity;
    let id = PersonId(next.0);
    match next.0.checked_add(1) {
        Some(n) => next.0 = n,
        None => {
            error!("bevy_society: PersonId overflow; person {entity:?} gets no id");
            return;
        }
    }
    commands.entity(entity).insert(id);
    index.by_id.insert(id, entity);
    index.sorted.push((id, entity));
}

/// Removes a despawned person from the index, from every other person's ties and from the rumor
/// book, and drops every rumor about it from every other person's knowledge.
///
/// A rumor whose subject leaves the world stops circulating, so nothing can ever spread it again;
/// leaving it in other persons' `Knowledge` would keep a dead rumor selectable by
/// `Knowledge::best_telling`.
fn on_person_removed(
    remove: On<Remove<Person>>,
    mut index: ResMut<PersonIndex>,
    mut ties: Query<&mut Ties>,
    rumors: Option<ResMut<RumorBook>>,
    mut knowledge: Query<&mut Knowledge>,
) {
    let entity = remove.entity;
    if let Some(pos) = index.sorted.iter().position(|(_, e)| *e == entity) {
        let (id, _) = index.sorted.remove(pos);
        index.by_id.remove(&id);
    }
    for mut own in &mut ties {
        own.0.retain(|tie| tie.target != entity);
    }
    if let Some(mut book) = rumors {
        for retired in book.retire_about(entity) {
            for mut owned in &mut knowledge {
                owned.forget(retired);
            }
        }
    }
}

/// Rebuilds [`Perception`] by brute force over [`PersonIndex::sorted`].
fn perceive(
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    transforms: Query<&GlobalTransform>,
    mut perception: ResMut<Perception>,
) {
    let n = index.sorted.len();
    let Perception {
        persons,
        pos,
        near,
        contact,
    } = &mut *perception;
    persons.clear();
    pos.clear();
    for (_, entity) in &index.sorted {
        persons.push(*entity);
        // A person without a transform yet sits at the origin for this tick.
        pos.push(
            transforms
                .get(*entity)
                .map(|t| settings.plane.project(t.translation()))
                .unwrap_or(Vec2::ZERO),
        );
    }
    near.resize_with(n, Vec::new);
    contact.resize_with(n, Vec::new);
    for list in near.iter_mut() {
        list.clear();
    }
    for list in contact.iter_mut() {
        list.clear();
    }
    let perception_radius = settings.perception_radius;
    let contact_radius = settings.contact_radius;
    for (i, pi) in pos.iter().enumerate() {
        for (j, pj) in pos.iter().enumerate() {
            if i == j {
                continue;
            }
            let d = pi.distance(*pj);
            if d <= perception_radius {
                near[i].push((j as u32, d));
            }
            if d <= contact_radius {
                contact[i].push(j as u32);
            }
        }
    }
}

/// Fails loudly when a society plugin is added before [`SocietyCorePlugin`]. This is one of the
/// only sanctioned panics in the crate; the check runs in `finish()`, after every plugin is built.
pub(crate) fn require_core(app: &App, plugin: &str) {
    if !app.is_plugin_added::<SocietyCorePlugin>() {
        panic!("bevy_society: add SocietyCorePlugin (or SocietyPlugins) before {plugin}");
    }
}

/// Script writes waiting to be applied (plan D8).
///
/// Every evaluation queues its actions here and one exclusive system drains the queue once per
/// tick, after [`SocietySystems::Rules`]. One application point is what keeps a script from
/// changing the world another trigger in the same tick reads, and what makes the order of
/// application independent of which system ran first.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct PendingActions(pub(crate) Vec<crate::script::ScriptAction>);

/// Queues script actions for the next application point. Plugins outside this module use it, so a
/// script effect queued in `Update` (a dialogue node) lands in the same place as one queued in
/// `FixedUpdate`.
pub(crate) fn push_script_actions(
    pending: &mut PendingActions,
    actions: impl IntoIterator<Item = crate::script::ScriptAction>,
) {
    pending.0.extend(actions);
}

/// Applies every queued script action, in queue order (plan D8).
///
/// This is the crate's one exclusive system: `fx` can write to any person component, so it needs the
/// whole world. It runs after [`SocietySystems::Rules`] on society ticks, and it resolves every
/// [`PersonId`] through [`PersonIndex::by_id`], skipping an action whose person is unknown with a
/// `warn!`.
fn apply_script_actions(world: &mut World) {
    let actions = {
        let Some(mut pending) = world.get_resource_mut::<PendingActions>() else {
            return;
        };
        std::mem::take(&mut pending.0)
    };
    if actions.is_empty() {
        return;
    }
    let now = world
        .get_resource::<SocietyClock>()
        .map_or(0.0, |clock| clock.now_days);
    for action in actions {
        crate::script_apply::apply(world, action, now);
    }
}

/// Persons, ids, clock, RNG, sets and perception. Add it before every other society plugin, or use
/// [`crate::SocietyPlugins`], which does.
pub struct SocietyCorePlugin;

impl Plugin for SocietyCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SocietyPlane>()
            .register_type::<AssessmentNorm>()
            .register_type::<Tuning>()
            .register_type::<SocietySettings>()
            .register_type::<SocietyClock>()
            .register_type::<SocietyTick>()
            .register_type::<NextPersonId>()
            .register_type::<PersonId>()
            .register_type::<Person>()
            .register_type::<Autonomous>()
            .register_type::<Home>()
            .register_type::<Place>()
            .register_type::<PlaceTags>()
            .register_type::<PersonIndex>()
            .register_type::<Perception>()
            .init_resource::<SocietySettings>()
            .init_resource::<SocietyClock>()
            .init_resource::<SocietyTick>()
            .init_resource::<NextPersonId>()
            .init_resource::<PersonIndex>()
            .init_resource::<Perception>()
            .init_resource::<SocietyDefs>()
            .init_resource::<crate::script::ScriptEngine>()
            .init_resource::<crate::snapshot::SnapshotCell>()
            .init_resource::<PendingActions>()
            .add_message::<crate::interact::Interact>()
            .add_message::<crate::interact::SocietyBark>()
            .add_message::<crate::script::ScriptEvent>()
            .add_message::<crate::script::ScriptError>()
            .add_observer(on_person_added)
            .add_observer(on_person_removed)
            .add_systems(Startup, setup_society)
            .configure_sets(
                FixedUpdate,
                (
                    SocietySystems::Clock,
                    SocietySystems::Perceive,
                    SocietySystems::Mind,
                    SocietySystems::Decide,
                    SocietySystems::Interact,
                    SocietySystems::Propagate,
                    SocietySystems::Relate,
                    SocietySystems::Rules,
                )
                    .chain(),
            )
            .configure_sets(
                FixedUpdate,
                (
                    SocietySystems::Perceive,
                    SocietySystems::Mind,
                    SocietySystems::Decide,
                    SocietySystems::Interact,
                    SocietySystems::Propagate,
                    SocietySystems::Relate,
                    SocietySystems::Rules,
                )
                    .run_if(society_tick_due),
            )
            .add_systems(
                FixedUpdate,
                (advance_clock, size_person_state).in_set(SocietySystems::Clock),
            )
            .add_systems(
                FixedUpdate,
                (perceive, crate::snapshot::build_snapshot)
                    .chain()
                    .in_set(SocietySystems::Perceive),
            )
            // One application point for every script write, after the rules have run.
            .add_systems(
                FixedUpdate,
                apply_script_actions
                    .after(SocietySystems::Rules)
                    .run_if(society_tick_due),
            );
    }
}

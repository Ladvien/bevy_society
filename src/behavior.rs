//! Daily-life data types: what a person is doing and where it is walking to.
//!
//! The decision and arrival systems that fill these components belong to `BehaviorPlugin`. This
//! module only defines the shared shapes, so the steering plugin and game code can read them
//! without depending on the behaviour systems.

use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_emerge_rng::DetRng;

use crate::Tuning;
use crate::core::{Perception, PersonIndex, SocietyClock, SocietySettings};
use crate::defs::{ActivityId, ActivityTarget, Signal, SocietyDefs, Term};
use crate::groups::Memberships;
use crate::mind::{Emotions, Mood, Needs, Personality};
use crate::relationships::Ties;

/// The target of a [`Doing`] activity.
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum DoingTarget {
    /// The activity needs no place or person (for example `idle`, or `sleep` at home).
    Itself,
    /// A point on the society plane, chosen by a wander option.
    Point(Vec2),
    /// A place entity, chosen by a tag option.
    Place(Entity),
    /// Another person, chosen by a person option.
    Person(Entity),
}

/// The activity a person is currently performing.
///
/// A person re-decides only when its `Doing` ends. While `arrived_day` is `None` the person is
/// travelling; the day it is set, the activity's needs and duration start counting.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct Doing {
    /// The activity definition this person is following.
    pub activity: ActivityId,
    /// What the activity is directed at.
    pub target: DoingTarget,
    /// Game day the activity started.
    pub started_day: f64,
    /// Game day the person reached the target, or `None` while still travelling.
    pub arrived_day: Option<f64>,
}

/// Where a travelling person is walking to, on the society plane.
///
/// Inserted and updated while travelling and removed on arrival or when the activity ends. Games
/// without the steering plugin move persons themselves toward this point.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Default)]
#[reflect(Component)]
pub struct Destination(pub Vec2);

// -------------------------------------------------------------------------------------------------
// Messages
// -------------------------------------------------------------------------------------------------

/// A social exchange finished (plan D7).
///
/// `refused_by_rule` distinguishes "the target said no" from "the `requires` script said no", which a
/// host usually wants to treat differently.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct InteractionResolved {
    /// The person who started the exchange.
    pub actor: Entity,
    /// The person it was directed at.
    pub target: Entity,
    /// Name of the `InteractionDef` that ran.
    pub interaction: String,
    /// Whether the target accepted.
    pub accepted: bool,
    /// Whether a `requires` script refused it before the acceptance draw.
    pub refused_by_rule: bool,
    /// The winner of a contest, when the interaction had one.
    pub winner: Option<Entity>,
}

/// A person began doing an activity (plan D7).
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ActivityStarted {
    /// The person.
    pub person: Entity,
    /// Name of the activity.
    pub activity: String,
    /// The person it is directed at, when it is a `Person` target.
    pub target: Option<Entity>,
}

/// A person stopped doing an activity (plan D7).
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ActivityEnded {
    /// The person.
    pub person: Entity,
    /// Name of the activity.
    pub activity: String,
    /// Whether the activity ran its full duration rather than being interrupted or timing out.
    pub completed: bool,
}

// -------------------------------------------------------------------------------------------------
// Queues
// -------------------------------------------------------------------------------------------------

/// [`Interact`](crate::Interact) requests waiting for the next society tick.
///
/// Drained on every fixed step, so a request written between two ticks is not dropped by Bevy's
/// two-swap message buffer.
#[derive(Resource, Debug, Clone, Default)]
pub struct InteractQueue(pub(crate) Vec<crate::interact::Interact>);

/// Exchanges that became due because a person arrived at a `Person` target (plan D7), in
/// [`PersonIndex::sorted`] order.
#[derive(Resource, Debug, Clone, Default)]
pub struct ArrivalQueue(pub(crate) Vec<crate::interact::Interact>);

/// Drain [`Interact`](crate::Interact) into [`InteractQueue`].
///
/// Ungated, in [`crate::SocietySystems::Clock`]: a game writes an `Interact` whenever it likes, and a
/// society tick is sixteen fixed steps away, so a tick-gated drain would lose most requests.
fn queue_interacts(
    mut reader: MessageReader<crate::interact::Interact>,
    mut queue: Option<ResMut<InteractQueue>>,
) {
    let Some(queue) = queue.as_mut() else {
        return;
    };
    for message in reader.read() {
        queue.0.push(message.clone());
    }
}

// -------------------------------------------------------------------------------------------------
// Plugin
// -------------------------------------------------------------------------------------------------

/// Runs daily life and social exchanges (plan D7): autonomous people choose activities, travel,
/// arrive, satisfy needs, and exchange with each other.
///
/// Requires [`crate::SocietyCorePlugin`]. Decision and exchange are exclusive systems: a decision
/// reads ties, needs, personality, mood, emotions, places and the clock for one person and writes
/// that person's `Doing`; an exchange can touch the actor, the target and every witness, which no
/// single `Query` can borrow at once.
pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Doing>()
            .register_type::<DoingTarget>()
            .register_type::<Destination>()
            .init_resource::<InteractQueue>()
            .init_resource::<ArrivalQueue>()
            .init_resource::<crate::rumor::RumorBook>()
            .add_message::<InteractionResolved>()
            .add_message::<ActivityStarted>()
            .add_message::<ActivityEnded>()
            .add_message::<crate::interact::Interact>()
            .add_message::<crate::interact::SocietyBark>()
            .add_message::<crate::script::ScriptError>()
            .add_message::<crate::rumor::RumorLearned>()
            .add_message::<crate::reputation::NormViolated>()
            .add_systems(
                FixedUpdate,
                queue_interacts.in_set(crate::SocietySystems::Clock),
            )
            .add_systems(
                FixedUpdate,
                (end_activities, decide_activities)
                    .chain()
                    .in_set(crate::SocietySystems::Decide),
            )
            .add_systems(
                FixedUpdate,
                (arrive_and_satisfy, resolve_interactions)
                    .chain()
                    .in_set(crate::SocietySystems::Interact),
            )
            .add_systems(
                FixedUpdate,
                activity_barks.in_set(crate::SocietySystems::Interact),
            )
            .add_systems(
                FixedUpdate,
                report_to_triggers
                    .after(crate::SocietySystems::Interact)
                    .before(crate::SocietySystems::Rules),
            );
    }

    fn finish(&self, app: &mut App) {
        crate::core::require_core(app, "BehaviorPlugin");
        // Both resources are present: `SocietyCorePlugin` initialized them.
        let settings = app.world().resource::<SocietySettings>().clone();
        let defs = app.world().resource::<SocietyDefs>();
        warn_short_activities(defs, &settings);
        warn_unscored_fields(defs);
    }
}

/// Warns that a definition relies on the social-field inputs, which the decision does not score.
///
/// A `Signal::Field` consideration currently scores 0 in every decision, and a
/// `ActivityTarget::Gradient` produces no targets, so an activity that depends on either is accepted
/// by the definitions file and can never be chosen. That is silent, and it is the same shape of
/// defect as a `Script` consideration scoring nothing: a host writes a definition, gets a society
/// where that activity never happens, and has no signal.
///
/// The fields themselves work — `SocialFieldPlugin` deposits, evaporates and exposes `field_value`
/// and `gradient_target` — so this is a wiring gap in this crate, not a missing dependency, and the
/// warning names it rather than pretending the feature exists.
fn warn_unscored_fields(defs: &SocietyDefs) {
    let mut offenders: Vec<&str> = Vec::new();
    for (name, signals) in defs.activity_signals() {
        // An activity with *both* a Field input and a Gradient target is one offender, not two: it was
        // counted twice before, which made the warning overstate how many definitions were affected.
        let reads_a_field = signals
            .iter()
            .any(|signal| matches!(signal, crate::defs::Signal::Field(_)));
        let walks_a_gradient = defs.activity_targets_gradient(name);
        if reads_a_field || walks_a_gradient {
            offenders.push(name);
        }
    }
    if offenders.is_empty() {
        return;
    }
    warn!(
        "bevy_society: {} definitions rely on social-field inputs (`Field` considerations or a \
         `Gradient` target), which the decision does not score yet: those activities will never be \
         chosen. The field grids and `SocialFields::field_value` work; the decision is what does not \
         read them. See `docs/loop/needs-decision/` — {}",
        offenders.len(),
        offenders.join(", ")
    );
}

/// Warns about activities that are shorter than a couple of society ticks.
///
/// `ActivitySpec::duration_hours` is game hours and one tick is `dt_days`, so the two only
/// correspond for a given `seconds_per_day`: a 0.1-hour `snub` lasts four ticks in the default
/// 240-second day and **one** tick in a four-second day. An activity that ends on the tick it
/// starts degenerates the society — a person re-decides every tick and repeats whatever scored
/// highest — and that is silent. Measured, a crowd ran `snub` 38 times per person per game-day
/// before anyone noticed.
///
/// The crate cannot fix the mismatch (which clock walking and durations belong to is a design
/// decision), but it can say so at startup instead of leaving a host to discover it in a plot.
fn warn_short_activities(defs: &SocietyDefs, settings: &SocietySettings) {
    let tick_hours = f64::from(settings.tick_secs) / f64::from(settings.seconds_per_day) * 24.0;
    if tick_hours <= 0.0 {
        return;
    }
    let mut degenerate: Vec<String> = Vec::new();
    for (index, activity) in defs.activities.iter().enumerate() {
        // The reserved idle activity is a one-hour fallback and is meant to be short.
        if index == usize::from(crate::defs::IDLE_ACTIVITY.0) {
            continue;
        }
        let ticks = f64::from(activity.duration_hours) / tick_hours;
        if ticks < 2.0 {
            degenerate.push(format!(
                "{} ({:.1}h = {:.1} ticks)",
                activity.name, activity.duration_hours, ticks
            ));
        }
    }
    if degenerate.is_empty() {
        return;
    }
    warn!(
        "bevy_society: {} activities are shorter than two society ticks at seconds_per_day {} \
         (one tick is {:.3} game-hours), so their people will re-decide every tick: {}",
        degenerate.len(),
        settings.seconds_per_day,
        tick_hours,
        degenerate.join(", ")
    );
}

// -------------------------------------------------------------------------------------------------
// Arrival
// -------------------------------------------------------------------------------------------------

/// Marks people who have reached their target and feeds the needs they are satisfying (plan D7).
///
/// A person has arrived within `interaction_radius` of the target. For a `Person` target the
/// destination is the target's position offset `0.75 · interaction_radius` toward the actor, so two
/// people stop short of each other rather than occupying the same point.
#[allow(clippy::too_many_arguments)]
fn arrive_and_satisfy(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    defs: Res<SocietyDefs>,
    index: Res<PersonIndex>,
    perception: Res<Perception>,
    transforms: Query<&GlobalTransform>,
    mut commands: Commands,
    mut people: Query<(&mut Doing, &mut Needs)>,
    mut started: MessageWriter<ActivityStarted>,
    mut arrivals: Option<ResMut<ArrivalQueue>>,
) {
    let now = clock.now_days;
    let dt_hours = f64::from(clock.dt_days) * 24.0;
    let radius = settings.interaction_radius;
    let plane = settings.plane;

    for (slot, &(_, person)) in index.sorted.iter().enumerate() {
        let Ok((mut doing, mut needs)) = people.get_mut(person) else {
            continue;
        };
        let Some(spec) = defs.activities.get(usize::from(doing.activity.0)).cloned() else {
            continue;
        };
        let here = perception.pos.get(slot).copied().unwrap_or(Vec2::ZERO);

        if doing.arrived_day.is_none() {
            let Some(target_pos) = target_position(
                doing.target,
                person,
                here,
                &transforms,
                &index,
                plane,
                radius,
            ) else {
                // A `Person` target that despawned: the activity is over.
                continue;
            };
            if here.distance(target_pos) > radius {
                continue;
            }
            doing.arrived_day = Some(now);
            commands.entity(person).remove::<Destination>();
            let person_target = match doing.target {
                DoingTarget::Person(other) => Some(other),
                _ => None,
            };
            started.write(ActivityStarted {
                person,
                activity: spec.name.clone(),
                target: person_target,
            });
            if let Some(arrivals) = arrivals.as_mut()
                && let Some(interaction) = spec.interaction
            {
                arrivals.0.push(crate::interact::Interact {
                    actor: person,
                    target: match doing.target {
                        DoingTarget::Person(other) => other,
                        _ => person,
                    },
                    interaction: defs.interaction_name(interaction).to_owned(),
                });
            }
        }

        if doing.arrived_day.is_some() {
            // `satisfies` is a per-game-hour delta, so a tick is `dt_days · 24` hours of it.
            for &(need, delta) in &spec.satisfies {
                if let Some(level) = needs.0.get_mut(usize::from(need.0)) {
                    *level = (*level + f64::from(delta) as f32 * dt_hours as f32).clamp(0.0, 1.0);
                }
            }
        }
    }
}

/// Where a `Doing` target is on the society plane, or `None` when a person target is gone.
fn target_position(
    target: DoingTarget,
    _actor: Entity,
    actor_pos: Vec2,
    transforms: &Query<&GlobalTransform>,
    index: &PersonIndex,
    plane: crate::SocietyPlane,
    interaction_radius: f32,
) -> Option<Vec2> {
    let of = |entity: Entity| -> Option<Vec2> {
        let _ = index;
        transforms
            .get(entity)
            .ok()
            .map(|transform| plane.project(transform.translation()))
    };
    match target {
        DoingTarget::Itself | DoingTarget::Point(_) => Some(actor_pos),
        DoingTarget::Place(place) => of(place),
        DoingTarget::Person(other) => {
            let there = of(other)?;
            // Stop short of the other person rather than walking into them.
            let toward = (actor_pos - there).normalize_or_zero();
            Some(there + toward * 0.75 * interaction_radius)
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Ending
// -------------------------------------------------------------------------------------------------

/// Ends every activity that is over (plan D7): its duration passed, its `Person` target left
/// perception, or it never arrived within `travel_timeout_hours`.
///
/// Gathers first and writes after, so the messages and the removals happen outside any borrow of the
/// people it read.
#[allow(clippy::too_many_arguments)]
fn end_activities(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    defs: Res<SocietyDefs>,
    index: Res<PersonIndex>,
    perception: Res<Perception>,
    mut commands: Commands,
    people: Query<&Doing>,
    mut ended: MessageWriter<ActivityEnded>,
) {
    let now = clock.now_days;
    let timeout_hours = f64::from(settings.tuning.travel_timeout_hours);
    let mut endings: Vec<(Entity, String, bool)> = Vec::new();

    for &(_, person) in &index.sorted {
        let Ok(doing) = people.get(person) else {
            continue;
        };
        let Some(spec) = defs.activities.get(usize::from(doing.activity.0)) else {
            continue;
        };
        let (finished, completed) = match doing.arrived_day {
            None => ((now - doing.started_day) * 24.0 > timeout_hours, false),
            Some(arrived) => {
                let done = (now - arrived) * 24.0 >= f64::from(spec.duration_hours);
                if done {
                    (true, true)
                } else if let DoingTarget::Person(target) = doing.target {
                    (!in_perception(&perception, &index, person, target), false)
                } else {
                    (false, false)
                }
            }
        };
        if finished {
            endings.push((person, spec.name.clone(), completed));
        }
    }

    for (person, activity, completed) in endings {
        ended.write(ActivityEnded {
            person,
            activity,
            completed,
        });
        commands.entity(person).remove::<(Doing, Destination)>();
    }
}

/// Whether `target` is within perception of `person`, compared by index position.
fn in_perception(
    perception: &Perception,
    index: &PersonIndex,
    person: Entity,
    target: Entity,
) -> bool {
    let Some(from) = index.sorted.iter().position(|(_, e)| *e == person) else {
        return false;
    };
    let Some(to) = index.sorted.iter().position(|(_, e)| *e == target) else {
        return false;
    };
    perception
        .near
        .get(from)
        .is_some_and(|list| list.iter().any(|(j, _)| *j as usize == to))
}

// -------------------------------------------------------------------------------------------------
// Deciding
// -------------------------------------------------------------------------------------------------

/// One place as a decision sees it (plan D7): its tag set and how full it is.
struct PlaceView {
    entity: Entity,
    tags: Vec<String>,
    capacity: u32,
    used: u32,
}

/// Everything scoring one candidate needs, copied out of the world once per decision (plan D7).
///
/// Copied rather than borrowed because a decision ends by writing `Doing` on the very person it
/// scored, and a `&Ties` from the world cannot coexist with `&mut World`.
struct Decision {
    position: Vec2,
    needs: Vec<f32>,
    personality: Personality,
    mood: [f32; 3],
    emotions: Emotions,
    ties: Vec<crate::relationships::Tie>,
    /// Neighbours with their plane positions, for `Distance`.
    near: Vec<(Entity, Vec2)>,
    /// The person's close ties at the tick start, for `SocialProof`.
    close_ties: Vec<Entity>,
    /// What each close tie is doing, so `SocialProof` can count who else chose this activity.
    close_doing: Vec<(Entity, u16)>,
    hour: f32,
    perception_radius: f32,
}

/// The components a decision reads for one person.
#[derive(QueryData)]
#[query_data(mutable)]
struct Decider<'w> {
    ties: &'w Ties,
    needs: &'w Needs,
    personality: &'w Personality,
    mood: &'w Mood,
    emotions: &'w Emotions,
    home: Option<&'w crate::Home>,
}

/// Everything a `Signal::Script` consideration needs to be evaluated (plan D7).
///
/// The defs and the snapshot are the two things a script reads; the queue is where its actions go so
/// they land at the same application point as every other script's.
struct ScriptContext<'a> {
    defs: &'a SocietyDefs,
    snapshot: &'a std::sync::Arc<crate::snapshot::SocietySnapshot>,
    engine: &'a crate::script::ScriptEngine,
    actor: i64,
    target: i64,
}

/// One option a decision is weighing: an activity and what it points at.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    activity: u16,
    target: DoingTarget,
    weight: f32,
    score: f32,
}

/// Chooses an activity for every autonomous person that has none (plan D7).
///
/// Each consideration `s` is compensated to `s′ = s + (1 − s)·(1 − 1/n)·s` and the option scores
/// `weight · Π s′`. Options are cut to the highest `rank` that has any positive score, then to
/// `tier_cutoff × best`, then one is drawn by weight from a single `unit()` draw.
///
/// A person with no positive option takes the reserved idle activity for one game-hour. Without that
/// floor a stalled society would re-decide every tick and drain the RNG stream doing nothing.
#[allow(clippy::too_many_arguments)]
fn decide_activities(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    defs: Res<SocietyDefs>,
    perception: Res<Perception>,
    index: Res<PersonIndex>,
    mut rng: ResMut<crate::SocietyRng>,
    mut commands: Commands,
    people: Query<Decider>,
    places: Query<(Entity, &crate::Place, &crate::PlaceTags)>,
    doing: Query<&Doing>,
    autonomous: Query<(), With<crate::Autonomous>>,
    mut started: MessageWriter<ActivityStarted>,
    engine: Option<Res<crate::script::ScriptEngine>>,
    cell: Res<crate::snapshot::SnapshotCell>,
    transforms: Query<(Entity, &Transform)>,
) {
    use bevy_emerge_rng::DetRng;

    let now = clock.now_days;
    let tuning = &settings.tuning;
    let tier_cutoff = tuning.tier_cutoff;
    let hour = clock.hour();

    // Occupancy: how many people already target each place.
    let mut used: Vec<(Entity, u32)> = Vec::new();
    for doing in &doing {
        if let DoingTarget::Place(place) = doing.target {
            match used.iter_mut().find(|(p, _)| *p == place) {
                Some((_, n)) => *n += 1,
                None => used.push((place, 1)),
            }
        }
    }
    let mut place_views: Vec<PlaceView> = places
        .iter()
        .map(|(entity, place, tags)| PlaceView {
            entity,
            tags: tags.0.clone(),
            capacity: place.capacity,
            used: used
                .iter()
                .find(|(p, _)| *p == entity)
                .map_or(0, |(_, n)| *n),
        })
        .collect();
    place_views.sort_by_key(|p| p.entity.to_bits());

    // What everyone is doing, for `SocialProof`.
    let mut all_doing: Vec<(Entity, u16)> = Vec::new();
    for &(_, entity) in &index.sorted {
        if let Ok(doing) = doing.get(entity) {
            all_doing.push((entity, doing.activity.0));
        }
    }

    // The engine and the tick's snapshot are the same for every consideration in this pass, so they
    // are read once and shared by reference.
    let snapshot = cell.0.clone();
    // Plane positions of everything a destination can name, gathered once: `Commands` cannot read
    // the world while a plan is being written.
    let mut positions: Vec<(Entity, Vec2)> = transforms
        .iter()
        .map(|(entity, transform): (Entity, &Transform)| {
            (entity, settings.plane.project(transform.translation))
        })
        .collect();
    positions.sort_by_key(|(e, _)| e.to_bits());
    let position_of = |entity: Entity| {
        positions
            .binary_search_by_key(&entity.to_bits(), |(e, _)| e.to_bits())
            .ok()
            .map(|i| positions[i].1)
    };

    let mut plans: Vec<(
        Entity,
        ActivityId,
        DoingTarget,
        bool,
        String,
        Option<Entity>,
    )> = Vec::new();

    for (slot, &(_, person)) in index.sorted.iter().enumerate() {
        if autonomous.get(person).is_err() || doing.get(person).is_ok() {
            continue;
        }
        let Ok(view) = people.get(person) else {
            continue;
        };
        let (ties, needs, personality, mood, emotions, home) = (
            view.ties,
            view.needs,
            view.personality,
            view.mood,
            view.emotions,
            view.home,
        );
        let position = perception.pos.get(slot).copied().unwrap_or(Vec2::ZERO);
        let near: Vec<(Entity, Vec2)> = perception
            .near
            .get(slot)
            .map(|list| {
                list.iter()
                    .filter_map(|(j, _)| {
                        let j = *j as usize;
                        let other = perception.persons.get(j).copied()?;
                        let pos = perception.pos.get(j).copied()?;
                        (other != person).then_some((other, pos))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let decision = Decision {
            position,
            needs: needs.0.clone(),
            personality: *personality,
            mood: mood.as_array(),
            emotions: emotions.clone(),
            ties: ties.0.clone(),
            near: near.clone(),
            close_ties: ties.close_targets(now as f32, tuning, settings.close_ties),
            close_doing: all_doing.clone(),
            hour,
            perception_radius: settings.perception_radius,
        };

        // The actor is fixed for this person; the target changes with every option, so it is bound
        // per candidate in `score_option` rather than here.
        let actor_id = index
            .sorted
            .iter()
            .find(|(_, e)| *e == person)
            // `-1`, not `0`, when the person is missing from the index: `0` is a real person id and a
            // script would read that person's state instead of finding nobody.
            .map_or(-1, |(id, _)| i64::from(id.0));
        // The scripts read the tick-start snapshot and the compiled definitions; both are the same
        // for every option, so they are borrowed once and only the target is rebound per candidate.
        let script_ready = match (&engine, &snapshot) {
            (Some(engine), Some(snapshot)) => Some((snapshot, engine)),
            _ => None,
        };
        let mut best: Option<(u8, Vec<Candidate>)> = None;
        for (i, spec) in defs.activities.iter().enumerate() {
            // Index 0 is the reserved idle activity: it is the fallback, never a candidate.
            if i == 0 {
                continue;
            }
            let targets = targets_for(
                &spec.target,
                position,
                &near,
                &place_views,
                home.map(|h| h.0),
                &mut rng.0,
            );
            for target in targets {
                let target_id = match target {
                    DoingTarget::Person(other) => index
                        .sorted
                        .iter()
                        .find(|(_, e)| *e == other)
                        .map_or(-1, |(id, _)| i64::from(id.0)),
                    _ => -1,
                };
                let script_ref = script_ready.map(|(snapshot, engine)| ScriptContext {
                    defs: &defs,
                    snapshot,
                    engine,
                    actor: actor_id,
                    target: target_id,
                });
                let score = score_option(
                    &spec.considerations,
                    &decision,
                    target,
                    i as u16,
                    script_ref.as_ref(),
                );
                if score <= 0.0 {
                    continue;
                }
                let candidate = Candidate {
                    activity: i as u16,
                    target,
                    weight: spec.weight,
                    score,
                };
                match &mut best {
                    Some((rank, list)) if *rank == spec.rank => list.push(candidate),
                    Some((rank, list)) if spec.rank > *rank => {
                        *rank = spec.rank;
                        *list = vec![candidate];
                    }
                    Some(_) => {}
                    None => best = Some((spec.rank, vec![candidate])),
                }
            }
        }

        let picked = best.and_then(|(_, list)| {
            let top = list.iter().map(|c| c.score).fold(0.0_f32, f32::max);
            let floor = top * tier_cutoff;
            let viable: Vec<&Candidate> = list.iter().filter(|c| c.score >= floor).collect();
            let total: f32 = viable.iter().map(|c| c.weight * c.score).sum();
            if total <= 0.0 {
                return None;
            }
            let mut roll = rng.0.unit() as f32 * total;
            for candidate in &viable {
                roll -= candidate.weight * candidate.score;
                if roll <= 0.0 {
                    return Some(**candidate);
                }
            }
            viable.last().map(|c| **c)
        });

        match picked {
            Some(candidate) => {
                let activity = ActivityId(candidate.activity);
                let name = defs.activity_name(activity).to_owned();
                let target = candidate.target;
                plans.push((
                    person,
                    activity,
                    target,
                    false,
                    name,
                    match target {
                        DoingTarget::Person(other) => Some(other),
                        _ => None,
                    },
                ));
            }
            None => plans.push((
                person,
                crate::defs::IDLE_ACTIVITY,
                DoingTarget::Itself,
                true,
                defs.activity_name(crate::defs::IDLE_ACTIVITY).to_owned(),
                None,
            )),
        }
    }

    for (person, activity, target, in_place, name, person_target) in plans {
        let mut entity = commands.entity(person);
        entity.insert(Doing {
            activity,
            target,
            started_day: now,
            arrived_day: in_place.then_some(now),
        });
        // Every target that is somewhere other than where the person stands needs a destination:
        // a `Point` is that point, a `Place` is its transform, and a `Person` is that person's
        // position offset toward the actor. Setting one only for `Point` meant a village whose
        // activities all target places never walked anywhere.
        match target {
            DoingTarget::Point(point) => {
                entity.insert(Destination(point));
            }
            DoingTarget::Place(place) => {
                if let Some(at) = position_of(place) {
                    entity.insert(Destination(at));
                }
            }
            DoingTarget::Person(other) => {
                if let (Some(theirs), Some(mine)) = (position_of(other), position_of(person)) {
                    let toward = (mine - theirs).normalize_or_zero();
                    entity.insert(Destination(
                        theirs + toward * 0.75 * settings.interaction_radius,
                    ));
                }
            }
            DoingTarget::Itself => {}
        }
        if in_place {
            started.write(ActivityStarted {
                person,
                activity: name,
                target: person_target,
            });
        }
    }
}

impl ScriptContext<'_> {
    /// Evaluates one `Signal::Script` consideration through the sandboxed engine.
    ///
    /// The result is clamped to `[0, 1]` like every other consideration input. A failure is an
    /// `error!` naming the script, and scores 0 — the rule the rest of the crate follows: a failed
    /// consideration scores 0 rather than aborting the decision.
    fn eval(
        &self,
        id: crate::script::ScriptId,
        decision: &Decision,
        target: DoingTarget,
    ) -> Option<f32> {
        let _ = (decision, target);
        let bindings = crate::script::Bindings {
            actor: self.actor,
            target: self.target,
            ..crate::script::Bindings::default()
        };
        match self
            .engine
            .eval_f32(&self.defs.scripts, id, self.snapshot, bindings)
        {
            Ok(value) => Some(value.clamp(0.0, 1.0)),
            Err(err) => {
                error!(
                    "bevy_society: script {} failed: {err}",
                    crate::script::ScriptEngine::owner(&self.defs.scripts, id)
                );
                None
            }
        }
    }
}

/// The targets an activity offers this decision (plan D7), with full places excluded.
///
/// `Wander` draws its point here, from the two `unit()` draws the SPEC names: radius · √u₁ at angle
/// 2πu₂, which is uniform over the disc rather than bunched at its centre.
fn targets_for(
    target: &ActivityTarget,
    position: Vec2,
    near: &[(Entity, Vec2)],
    places: &[PlaceView],
    home: Option<Entity>,
    rng: &mut rand_chacha::ChaCha8Rng,
) -> Vec<DoingTarget> {
    use bevy_emerge_rng::DetRng;
    match target {
        ActivityTarget::Itself => vec![DoingTarget::Itself],
        ActivityTarget::Home => home.map_or_else(Vec::new, |h| vec![DoingTarget::Place(h)]),
        ActivityTarget::Wander(radius) => {
            let r = radius * (rng.unit() as f32).sqrt();
            let angle = std::f32::consts::TAU * rng.unit() as f32;
            vec![DoingTarget::Point(
                position + Vec2::new(r * angle.cos(), r * angle.sin()),
            )]
        }
        ActivityTarget::Place(tag) => places
            .iter()
            .filter(|place| place.used < place.capacity && place.tags.iter().any(|t| t == tag))
            .map(|place| DoingTarget::Place(place.entity))
            .collect(),
        ActivityTarget::Person => near.iter().map(|(e, _)| DoingTarget::Person(*e)).collect(),
        // A gradient target needs the social-field grids, which only the `fields` feature builds.
        ActivityTarget::Gradient(..) => Vec::new(),
    }
}

/// Scores one option: `weight · Π s′` over its considerations (plan D7).
///
/// The compensation `s′ = s + (1 − s)·(1 − 1/n)·s` is a design choice. Game AI Pro 3 ch. 13
/// multiplies raw scores; compensation keeps an activity described with five considerations
/// comparable to one described with two, instead of losing purely for being described more fully.
fn score_option(
    considerations: &[Term],
    decision: &Decision,
    target: DoingTarget,
    activity: u16,
    scripts: Option<&ScriptContext<'_>>,
) -> f32 {
    let n = considerations.len();
    if n == 0 {
        return 1.0;
    }
    let mut product = 1.0_f32;
    for term in considerations {
        let s = term.curve.eval(signal_value(
            term.signal,
            decision,
            target,
            activity,
            scripts,
        ));
        let compensated = s + (1.0 - s) * (1.0 - 1.0 / n as f32) * s;
        product *= compensated;
    }
    product
}

/// One consideration's normalised `[0, 1]` input (plan D7).
///
/// **A zero is final.** Scoring is multiplicative, so one consideration at 0 zeroes the whole option
/// and the compensation term cannot rescue it — `s + (1-s)(1-1/n)s` is still 0 at `s = 0`. A
/// definition that wants "almost never" must say `0.01`, not `0`; a definition that wants "never"
/// says `0`. This is deliberate: a need that is entirely unmet should make its activity impossible
/// now, not unlikely.
///
/// Every arm returns a real value computed from the decision's copied state. `Field` and `Script`
/// read the social-field grids and the Rhai snapshot, which a decision does not hold; they score 0
/// and a host that wants them should say so through a definition the decision can score instead.
fn signal_value(
    signal: Signal,
    decision: &Decision,
    target: DoingTarget,
    activity: u16,
    scripts: Option<&ScriptContext<'_>>,
) -> f32 {
    let tie = match target {
        DoingTarget::Person(other) => decision.ties.iter().find(|t| t.target == other).copied(),
        _ => None,
    };
    let value = match signal {
        Signal::Need(id) => decision
            .needs
            .get(usize::from(id.0))
            .copied()
            .unwrap_or(0.0),
        Signal::Hour => decision.hour / 24.0,
        Signal::Pleasure => (decision.mood[0] + 1.0) / 2.0,
        Signal::Arousal => (decision.mood[1] + 1.0) / 2.0,
        Signal::Dominance => (decision.mood[2] + 1.0) / 2.0,
        Signal::Trait(which) => decision.personality.value(which),
        Signal::Emotion(kind) => decision.emotions.intensity_of(kind),
        Signal::Affinity => (tie.map_or(0.0, |t| t.affinity) + 1.0) / 2.0,
        Signal::Trust => tie.map_or(0.5, |t| t.trust),
        Signal::Respect => (tie.map_or(0.0, |t| t.respect) + 1.0) / 2.0,
        Signal::Familiarity => match target {
            DoingTarget::Person(other) => decision
                .ties
                .iter()
                .find(|t| t.target == other)
                .map_or(0.0, |t| t.familiarity_peak),
            _ => 0.0,
        },
        Signal::Distance => {
            let there = match target {
                DoingTarget::Person(other) => decision
                    .near
                    .iter()
                    .find(|(e, _)| *e == other)
                    .map(|(_, p)| *p),
                // A place or point is somewhere in the world; without its position in the decision
                // the honest value is "as far as it can see", not a confident zero.
                DoingTarget::Place(_) | DoingTarget::Point(_) | DoingTarget::Itself => None,
            };
            there.map_or(1.0, |p| {
                (decision.position.distance(p) / decision.perception_radius).min(1.0)
            })
        }
        Signal::SocialProof => {
            if decision.close_ties.is_empty() {
                0.0
            } else {
                let following = decision
                    .close_ties
                    .iter()
                    .filter(|t| {
                        decision
                            .close_doing
                            .iter()
                            .any(|(e, a)| e == *t && *a == activity)
                    })
                    .count();
                following as f32 / decision.close_ties.len() as f32
            }
        }
        Signal::Script(id) => match scripts {
            Some(context) => context.eval(id, decision, target).unwrap_or(0.0),
            // A decision made with no scripting context (no plugin, no snapshot yet) cannot score a
            // script consideration. It scores 0 and the activity is skipped for this tick, which is
            // the same answer a failed script gives.
            None => 0.0,
        },
        // A field input needs the stigmergy grids, which only the `fields` feature builds.
        Signal::Field(_) => 0.0,
    };
    value.clamp(0.0, 1.0)
}

// -------------------------------------------------------------------------------------------------
// Exchanging
// -------------------------------------------------------------------------------------------------

/// Resolves every due social exchange (plan D7): the autonomous arrivals first, then the game's
/// [`Interact`](crate::interact::Interact) requests in arrival order.
///
/// Exclusive because one exchange can touch the actor, the target and every witness. No single
/// `Query` can borrow that set, and splitting the steps across systems would make the RNG draw order
/// depend on system order rather than on [`PersonIndex::sorted`] order.
fn resolve_interactions(world: &mut World) {
    // One gather scope: every read the exchanges need is copied out before any of them writes.
    let (now, settings, defs, perception, index, due) = {
        let now = world.resource::<SocietyClock>().now_days;
        let settings = world.resource::<SocietySettings>().clone();
        let defs = world.resource::<SocietyDefs>().clone();
        let perception = world.resource::<Perception>().clone();
        let index = world.resource::<PersonIndex>().sorted.clone();

        // Arrivals first, then game requests, both in the order they were queued.
        let mut due: Vec<crate::interact::Interact> = {
            let mut arrivals = world.remove_resource::<ArrivalQueue>().unwrap_or_default();
            std::mem::take(&mut arrivals.0)
        };
        let mut queue = world.remove_resource::<InteractQueue>().unwrap_or_default();
        due.extend(std::mem::take(&mut queue.0));
        world.insert_resource(queue);
        world.insert_resource(ArrivalQueue::default());
        (now, settings, defs, perception, index, due)
    };

    for request in due {
        exchange(world, &request, &defs, &settings, &perception, &index, now);
    }
}

/// One exchange, start to finish (plan D7).
#[allow(clippy::too_many_arguments)]
fn exchange(
    world: &mut World,
    request: &crate::interact::Interact,
    defs: &SocietyDefs,
    settings: &SocietySettings,
    perception: &Perception,
    index: &[(crate::PersonId, Entity)],
    now: f64,
) {
    let (actor, target) = (request.actor, request.target);
    let Some(id) = defs.interaction_id(&request.interaction) else {
        error!(
            "bevy_society: Interact names unknown interaction '{}'; ignored",
            request.interaction
        );
        return;
    };
    let Some(spec) = defs.interactions.get(usize::from(id.0)).cloned() else {
        return;
    };
    if world.get::<Ties>(actor).is_none() || world.get::<Ties>(target).is_none() {
        return;
    }

    // (1) `requires` first: a script that says no ends it with no draw and no effect.
    if let Some(requires) = spec.requires
        && !condition_holds(world, defs, requires, actor, target)
    {
        world.write_message(InteractionResolved {
            actor,
            target,
            interaction: spec.name.clone(),
            accepted: false,
            refused_by_rule: true,
            winner: None,
        });
        return;
    }

    // (2) The acceptance draw.
    let target_ties = world.get::<Ties>(target).cloned().unwrap_or_default();
    let target_personality = world
        .get::<Personality>(target)
        .copied()
        .unwrap_or_default();
    let target_mood = world.get::<Mood>(target).copied().unwrap_or_default();
    let toward_actor = target_ties
        .get(actor)
        .copied()
        .unwrap_or_else(|| crate::relationships::Tie::new(actor));
    let shared_faction = {
        let mine = world.get::<Memberships>(actor).cloned().unwrap_or_default();
        let theirs = world
            .get::<Memberships>(target)
            .cloned()
            .unwrap_or_default();
        mine.shares_with(&theirs)
    };
    let accept = &spec.accept;
    let probability = (accept.base
        + accept.affinity * toward_actor.affinity
        + accept.trust * (toward_actor.trust - 0.5)
        + accept.agreeableness * (target_personality.agreeableness - 0.5)
        + accept.pleasure * target_mood.pleasure
        + if shared_faction { accept.ingroup } else { 0.0 })
    .clamp(0.02, 0.98);
    let roll = world.resource_mut::<crate::SocietyRng>().0.unit() as f32;
    let accepted = roll < probability;

    // (3) The effects for the outcome.
    let effects = if accepted {
        &spec.on_accept
    } else {
        &spec.on_reject
    };
    apply_effects(world, effects, actor, target, &spec.name, now);

    let mut winner = None;
    if accepted {
        // (4) Exchange proper: influence, gossip, contest, deed, closure, contact.
        influence(world, &spec, actor, target, &settings.tuning);
        if spec.gossip {
            tell(world, actor, target, settings, now);
        }
        if spec.contest {
            winner = contest(world, actor, target, &settings.tuning, perception, index);
        }
        if let Some(deed) = spec.deed {
            witness(
                world, deed, actor, target, defs, settings, perception, index, now,
            );
        }
        close_the_triad(world, actor, target, perception, index, settings, now);
        if !shared_faction
            && spec.deed.is_none_or(|d| {
                defs.deeds
                    .get(usize::from(d.0))
                    .is_none_or(|s| s.harm >= 0.0)
            })
        {
            intergroup(world, actor, target, &settings.tuning);
        }
    } else {
        // (5) Rejection penalty.
        if let Some(mut ties) = world.get_mut::<Ties>(actor) {
            let tie = ties.entry(target);
            tie.affinity = (tie.affinity - 0.05).clamp(-1.0, 1.0);
        }
        world.write_message(crate::mind::Appraise {
            person: actor,
            kind: crate::mind::EmotionKind::Distress,
            intensity: 0.3,
            toward: Some(target),
        });
    }

    // (6) Memories, both sides.
    let harm = spec
        .deed
        .and_then(|d| defs.deeds.get(usize::from(d.0)))
        .map_or(0.0, |spec| spec.harm);
    let magnitude = 0.3 + harm.abs();
    let actor_valence = if accepted { magnitude } else { -magnitude };
    let target_valence = if harm < 0.0 {
        -magnitude
    } else if accepted {
        magnitude
    } else {
        -magnitude
    };
    record(
        world,
        actor,
        target,
        id,
        accepted,
        true,
        actor_valence,
        effects.importance,
        now,
    );
    record(
        world,
        target,
        actor,
        id,
        accepted,
        false,
        target_valence,
        effects.importance,
        now,
    );

    // (7) Exposure for the pair, both ways.
    expose(world, actor, target, settings, now);
    expose(world, target, actor, settings, now);

    // (8) The report.
    world.write_message(InteractionResolved {
        actor,
        target,
        interaction: spec.name.clone(),
        accepted,
        refused_by_rule: false,
        winner,
    });

    // (9) Barks.
    bark(world, &spec, actor, target, accepted, settings);
}

/// (3) Applies one effect set: tie deltas both ways, emotions, needs, and the optional script.
fn apply_effects(
    world: &mut World,
    effects: &crate::defs::EffectsSpec,
    actor: Entity,
    target: Entity,
    _interaction: &str,
    _now: f64,
) {
    // Tie deltas, clamped exactly as `fx.adjust` clamps them.
    for (owner, other, delta) in [
        (actor, target, effects.actor_to_target),
        (target, actor, effects.target_to_actor),
    ] {
        let Some(mut ties) = world.get_mut::<Ties>(owner) else {
            continue;
        };
        let tie = ties.entry(other);
        tie.affinity = (tie.affinity + delta.affinity).clamp(-1.0, 1.0);
        tie.trust = (tie.trust + delta.trust).clamp(0.0, 1.0);
        tie.respect = (tie.respect + delta.respect).clamp(-1.0, 1.0);
        if delta.familiarity != 0.0 {
            tie.familiarity_peak = (tie.familiarity_peak + delta.familiarity).clamp(0.0, 1.0);
        }
    }
    for (person, emotion) in [
        (actor, effects.actor_emotion),
        (target, effects.target_emotion),
    ] {
        if let Some((kind, intensity)) = emotion {
            world.write_message(crate::mind::Appraise {
                person,
                kind,
                intensity,
                toward: Some(if person == actor { target } else { actor }),
            });
        }
    }
    for (person, deltas) in [
        (actor, &effects.actor_needs),
        (target, &effects.target_needs),
    ] {
        let Some(mut needs) = world.get_mut::<Needs>(person) else {
            continue;
        };
        for &(need, delta) in deltas {
            if let Some(level) = needs.0.get_mut(usize::from(need.0)) {
                *level = (*level + delta).clamp(0.0, 1.0);
            }
        }
    }
    // A failed script applies none of its actions, so it runs as one unit through the normal
    // application point rather than writing the world here.
    if let Some(script) = effects.script {
        run_effect(world, script, actor, target);
    }
}

/// (4a) Opinion exchange over the interaction's topics, both directions (plan D5).
fn influence(
    world: &mut World,
    spec: &crate::defs::InteractionSpec,
    actor: Entity,
    target: Entity,
    tuning: &Tuning,
) {
    if spec.influence.is_empty() {
        return;
    }
    let Some(a_opinions) = world.get::<crate::Opinions>(actor).cloned() else {
        return;
    };
    let Some(b_opinions) = world.get::<crate::Opinions>(target).cloned() else {
        return;
    };
    let a_openness = world.get::<Personality>(actor).map_or(0.5, |p| p.openness);
    let b_openness = world.get::<Personality>(target).map_or(0.5, |p| p.openness);
    // Both sides' pre-step values, so neither direction sees the other's write.
    let a_values = crate::beliefs::snapshot_values(&a_opinions, &spec.influence);
    let b_values = crate::beliefs::snapshot_values(&b_opinions, &spec.influence);
    // Each side's opinions and ties are cloned out and written back: `exchange_opinions` needs both
    // at once, which two `get_mut` calls on one world cannot provide.
    for (holder, other, openness, other_values) in [
        (actor, target, a_openness, b_values),
        (target, actor, b_openness, a_values),
    ] {
        let (Some(mut opinions), Some(mut ties)) = (
            world.get::<crate::Opinions>(holder).cloned(),
            world.get::<Ties>(holder).cloned(),
        ) else {
            continue;
        };
        crate::beliefs::exchange_opinions(
            &mut opinions,
            &mut ties,
            openness,
            other,
            &other_values,
            tuning,
        );
        if let Some(mut slot) = world.get_mut::<crate::Opinions>(holder) {
            *slot = opinions;
        }
        if let Some(mut slot) = world.get_mut::<Ties>(holder) {
            *slot = ties;
        }
    }
}

/// (4b) One telling (plan D5): the teller picks its heaviest rumor and the listener learns it, or the
/// teller stifles because the listener already knew.
fn tell(world: &mut World, teller: Entity, listener: Entity, settings: &SocietySettings, now: f64) {
    let Some(teller_knowledge) = world.get::<crate::Knowledge>(teller).cloned() else {
        return;
    };
    let trust_in_teller = world
        .get::<Ties>(listener)
        .and_then(|ties| ties.get(teller).map(|tie| tie.trust))
        .unwrap_or(0.5);
    let close = world
        .get::<Ties>(listener)
        .map(|ties| ties.close_targets(now as f32, &settings.tuning, settings.close_ties))
        .unwrap_or_default();

    // Clone the listener's state out, run the telling, write it back: `gossip` needs four of the
    // listener's components at once, which four `get_mut` calls on one world cannot provide.
    let Some(mut listener_knowledge) = world.get::<crate::Knowledge>(listener).cloned() else {
        return;
    };
    let Some(mut listener_ties) = world.get::<Ties>(listener).cloned() else {
        return;
    };
    let mut listener_emotions = world.get::<Emotions>(listener).cloned().unwrap_or_default();
    let mut listener_memories = world
        .get::<crate::mind::Memories>(listener)
        .cloned()
        .unwrap_or_default();

    let mut book = world
        .remove_resource::<crate::RumorBook>()
        .unwrap_or_default();
    let outcome = crate::rumor::gossip(
        &mut book,
        &teller_knowledge,
        &mut listener_knowledge,
        &mut listener_ties,
        &mut listener_emotions,
        &mut listener_memories,
        listener,
        trust_in_teller,
        &close,
        settings,
        now as f32,
    );

    if let Some(mut slot) = world.get_mut::<crate::Knowledge>(listener) {
        *slot = listener_knowledge;
    }
    if let Some(mut slot) = world.get_mut::<Ties>(listener) {
        *slot = listener_ties;
    }
    if let Some(mut slot) = world.get_mut::<Emotions>(listener) {
        *slot = listener_emotions;
    }
    if let Some(mut slot) = world.get_mut::<crate::mind::Memories>(listener) {
        *slot = listener_memories;
    }

    // The DK rule: a telling to someone who already knows stifles the teller.
    if let crate::rumor::Gossip::Stifled(id) = outcome
        && let Some(mut knowledge) = world.get_mut::<crate::Knowledge>(teller)
        && let Some(known) = knowledge.get_mut(id)
    {
        known.spreader = false;
    }
    if let crate::rumor::Gossip::Learned { rumor, .. } = outcome {
        world.write_message(crate::rumor::RumorLearned {
            person: listener,
            rumor,
        });
    }
    world.insert_resource(book);
}

/// (4c) A contest between the two (plan D6), with every witness of the actor raising its respect for
/// the winner.
fn contest(
    world: &mut World,
    actor: Entity,
    target: Entity,
    tuning: &Tuning,
    perception: &Perception,
    index: &[(crate::PersonId, Entity)],
) -> Option<Entity> {
    let dominance = |world: &World, e: Entity| world.get::<Mood>(e).map_or(0.0, |m| m.dominance);
    let standing =
        |world: &World, e: Entity| world.get::<crate::Status>(e).map_or(0.0, |s| s.standing);
    let a_dom = dominance(world, actor);
    let a_stand = standing(world, actor);
    let t_dom = dominance(world, target);
    let t_stand = standing(world, target);

    // Both contestants' state comes out of the world, because `resolve_contest` needs both at once
    // and two `get_mut` calls on one world cannot coexist.
    let mut a_ties = world.get::<Ties>(actor).cloned()?;
    let mut t_ties = world.get::<Ties>(target).cloned()?;
    let mut a_emotions = world.get::<Emotions>(actor).cloned()?;
    let mut t_emotions = world.get::<Emotions>(target).cloned()?;

    let resolved = crate::groups::resolve_contest(
        &mut world.resource_mut::<crate::SocietyRng>().0,
        tuning.contest_steepness,
        crate::groups::Contestant {
            entity: actor,
            dominance: a_dom,
            standing: a_stand,
            ties: &mut a_ties,
            emotions: &mut a_emotions,
        },
        crate::groups::Contestant {
            entity: target,
            dominance: t_dom,
            standing: t_stand,
            ties: &mut t_ties,
            emotions: &mut t_emotions,
        },
    );

    if let Some(mut ties) = world.get_mut::<Ties>(actor) {
        *ties = a_ties;
    }
    if let Some(mut ties) = world.get_mut::<Ties>(target) {
        *ties = t_ties;
    }
    if let Some(mut emotions) = world.get_mut::<Emotions>(actor) {
        *emotions = a_emotions;
    }
    if let Some(mut emotions) = world.get_mut::<Emotions>(target) {
        *emotions = t_emotions;
    }

    // Every neighbour that can see the actor raises its respect for the winner.
    let actor_slot = index.iter().position(|(_, e)| *e == actor)?;
    let witnesses: Vec<Entity> = perception
        .near
        .get(actor_slot)
        .map(|list| {
            list.iter()
                .filter_map(|(j, _)| perception.persons.get(*j as usize).copied())
                .filter(|w| *w != actor && *w != target)
                .collect()
        })
        .unwrap_or_default();
    for watcher in witnesses {
        if let Some(mut ties) = world.get_mut::<Ties>(watcher) {
            crate::groups::witness_contest(&mut ties, resolved);
        }
    }
    Some(resolved)
}

/// (4d) One deed and its witnesses (plan D5): each witness judges it, a harsh judgement is a
/// `NormViolated`, and a heavy enough deed becomes a rumor every witness already knows.
#[allow(clippy::too_many_arguments)]
fn witness(
    world: &mut World,
    deed: crate::defs::DeedId,
    actor: Entity,
    patient: Entity,
    defs: &SocietyDefs,
    settings: &SocietySettings,
    perception: &Perception,
    index: &[(crate::PersonId, Entity)],
    now: f64,
) {
    let Some(deed_spec) = defs.deeds.get(usize::from(deed.0)).cloned() else {
        return;
    };
    let Some(actor_slot) = index.iter().position(|(_, e)| *e == actor) else {
        return;
    };
    let witnesses: Vec<Entity> = perception
        .near
        .get(actor_slot)
        .map(|list| {
            list.iter()
                .filter_map(|(j, _)| perception.persons.get(*j as usize).copied())
                .filter(|w| *w != actor && *w != patient)
                .collect()
        })
        .unwrap_or_default();

    let mut approvals: Vec<(Entity, f32)> = Vec::new();
    for watcher in witnesses {
        let opinions = world
            .get::<crate::Opinions>(watcher)
            .cloned()
            .unwrap_or_default();
        let personality = world
            .get::<Personality>(watcher)
            .copied()
            .unwrap_or_default();
        let Some(mut ties) = world.get::<Ties>(watcher).cloned() else {
            continue;
        };
        let mut emotions = world.get::<Emotions>(watcher).cloned().unwrap_or_default();
        let mut memories = world
            .get::<crate::mind::Memories>(watcher)
            .cloned()
            .unwrap_or_default();
        let trust_in_patient = ties.get(patient).map(|tie| tie.trust);
        let reaction = crate::reputation::witness_reaction(
            &mut ties,
            &mut emotions,
            &mut memories,
            &opinions,
            &personality,
            deed,
            deed_spec.topic,
            deed_spec.harm,
            actor,
            Some(patient),
            trust_in_patient,
            settings,
            now,
        );
        // Write every component back, unconditionally: a lost `Knowledge` restore is silent amnesia.
        if let Some(mut slot) = world.get_mut::<Ties>(watcher) {
            *slot = ties;
        }
        if let Some(mut slot) = world.get_mut::<Emotions>(watcher) {
            *slot = emotions;
        }
        if let Some(mut slot) = world.get_mut::<crate::mind::Memories>(watcher) {
            *slot = memories;
        }
        if reaction.violated {
            world.write_message(crate::reputation::NormViolated {
                witness: watcher,
                actor,
                deed: deed_spec.name.clone(),
            });
        }
        approvals.push((watcher, reaction.approval));
    }

    // The patient and the actor react to the deed itself.
    let gain = {
        let conscientiousness = world
            .get::<Personality>(patient)
            .map_or(0.5, |p| p.conscientiousness);
        crate::reputation::trust_gain(&settings.tuning, conscientiousness)
    };
    if let Some(mut ties) = world.get::<Ties>(patient).cloned() {
        let mut emotions = world.get::<Emotions>(patient).cloned().unwrap_or_default();
        crate::reputation::patient_reaction(&mut ties, &mut emotions, deed_spec.harm, gain, actor);
        if let Some(mut slot) = world.get_mut::<Ties>(patient) {
            *slot = ties;
        }
        if let Some(mut slot) = world.get_mut::<Emotions>(patient) {
            *slot = emotions;
        }
    }
    if let Some(opinions) = world.get::<crate::Opinions>(actor).cloned()
        && let Some(mut emotions) = world.get::<Emotions>(actor).cloned()
    {
        crate::reputation::actor_reaction(&mut emotions, &opinions, deed_spec.topic, patient);
        if let Some(mut slot) = world.get_mut::<Emotions>(actor) {
            *slot = emotions;
        }
    }

    // A rumor, when the deed is heavy enough or a witness felt strongly.
    let heavy = deed_spec.harm.abs() >= settings.tuning.rumor_threshold;
    let strong = approvals.iter().any(|(_, a)| a.abs() >= 0.5);
    if (heavy || strong) && !approvals.is_empty() {
        let mut book = world
            .remove_resource::<crate::RumorBook>()
            .unwrap_or_default();
        let mut known: Vec<(Entity, crate::Knowledge)> = Vec::new();
        for (watcher, _) in &approvals {
            if let Some(knowledge) = world.get::<crate::Knowledge>(*watcher) {
                known.push((*watcher, knowledge.clone()));
            }
        }
        let id = crate::reputation::spread_witnessed_rumor(
            &mut book,
            &mut known,
            actor,
            Some(patient),
            deed,
            deed_spec.harm,
            now as f32,
            true,
        );
        for (watcher, knowledge) in known {
            if let Some(mut slot) = world.get_mut::<crate::Knowledge>(watcher) {
                *slot = knowledge;
            }
        }
        if let Some(rumor) = id {
            for (watcher, _) in &approvals {
                world.write_message(crate::rumor::RumorLearned {
                    person: *watcher,
                    rumor,
                });
            }
        }
        world.insert_resource(book);
    }
}

/// (4e) Triadic closure (plan D4): every close tie of each party in contact gains familiarity with
/// the other.
fn close_the_triad(
    world: &mut World,
    actor: Entity,
    target: Entity,
    perception: &Perception,
    index: &[(crate::PersonId, Entity)],
    settings: &SocietySettings,
    now: f64,
) {
    for (holder, other) in [(actor, target), (target, actor)] {
        let (Some(slot), Some(other_slot)) = (
            index.iter().position(|(_, e)| *e == holder),
            index.iter().position(|(_, e)| *e == other),
        ) else {
            continue;
        };
        let in_contact = perception
            .contact
            .get(slot)
            .is_some_and(|list| list.iter().any(|j| *j as usize == other_slot));
        if !in_contact {
            continue;
        }
        let Some(ties) = world.get::<Ties>(holder).cloned() else {
            continue;
        };
        let close = ties.close_targets(now as f32, &settings.tuning, settings.close_ties);
        for friend in close {
            if friend == other {
                continue;
            }
            if let Some(mut friend_ties) = world.get_mut::<Ties>(friend) {
                crate::relationships::introduce_toward(
                    &mut friend_ties,
                    other,
                    now as f32,
                    &settings.tuning,
                );
            }
        }
    }
}

/// (4f) Intergroup contact (plan D6): an accepted exchange with no harmful deed between people who
/// share no faction adds affinity both ways.
fn intergroup(world: &mut World, actor: Entity, target: Entity, tuning: &Tuning) {
    let Some(mut a_ties) = world.get::<Ties>(actor).cloned() else {
        return;
    };
    let Some(mut t_ties) = world.get::<Ties>(target).cloned() else {
        return;
    };
    {
        let mut a_to_t = *a_ties.entry(target);
        let mut t_to_a = *t_ties.entry(actor);
        crate::groups::intergroup_contact(&mut a_to_t, &mut t_to_a, tuning);
        *a_ties.entry(target) = a_to_t;
        *t_ties.entry(actor) = t_to_a;
    }
    if let Some(mut ties) = world.get_mut::<Ties>(actor) {
        *ties = a_ties;
    }
    if let Some(mut ties) = world.get_mut::<Ties>(target) {
        *ties = t_ties;
    }
}

/// (6) Records one side's memory of the exchange (plan D7).
#[allow(clippy::too_many_arguments)]
fn record(
    world: &mut World,
    person: Entity,
    other: Entity,
    interaction: crate::defs::InteractionId,
    accepted: bool,
    initiated: bool,
    valence: f32,
    importance: f32,
    now: f64,
) {
    let capacity = world.resource::<SocietySettings>().memory_capacity;
    if let Some(mut memories) = world.get_mut::<crate::mind::Memories>(person) {
        memories.record(
            crate::mind::Memory {
                day: now as f32,
                kind: crate::mind::MemoryKind::Interaction {
                    id: interaction,
                    accepted,
                    initiated,
                },
                other: Some(other),
                valence: valence.clamp(-1.0, 1.0),
                importance: importance.clamp(0.0, 1.0),
            },
            now,
            capacity,
        );
    }
}

/// (7) One direction of mere exposure for the pair (plan D4).
fn expose(world: &mut World, holder: Entity, other: Entity, settings: &SocietySettings, now: f64) {
    let bias = {
        let mine = world
            .get::<Memberships>(holder)
            .cloned()
            .unwrap_or_default();
        let theirs = world.get::<Memberships>(other).cloned().unwrap_or_default();
        let defs = world.resource::<SocietyDefs>().clone();
        crate::groups::shared_faction_bias(&mine, &theirs, &defs)
    };
    let dt = world_dt(world);
    let tuning = settings.tuning.clone();
    if let Some(mut ties) = world.get_mut::<Ties>(holder) {
        crate::relationships::expose_toward(&mut ties, other, bias, now as f32, dt, &tuning);
    }
}

/// The length of one society tick in game-days.
fn world_dt(world: &World) -> f32 {
    world.resource::<SocietyClock>().dt_days
}

/// (9) Barks (plan D7): the actor's line, then an independent draw for the target's.
fn bark(
    world: &mut World,
    spec: &crate::defs::InteractionSpec,
    actor: Entity,
    target: Entity,
    accepted: bool,
    settings: &SocietySettings,
) {
    let chance = settings.bark_chance;
    let actor_roll = world.resource_mut::<crate::SocietyRng>().0.unit() as f32;
    if actor_roll < chance && !spec.barks.actor.is_empty() {
        let index = world
            .resource_mut::<crate::SocietyRng>()
            .0
            .below(spec.barks.actor.len());
        let line = spec.barks.actor[index].clone();
        say(world, actor, target, &line);
    }
    let lines = if accepted {
        spec.barks.accepted.clone()
    } else {
        spec.barks.rejected.clone()
    };
    let target_roll = world.resource_mut::<crate::SocietyRng>().0.unit() as f32;
    if target_roll < chance && !lines.is_empty() {
        let index = world
            .resource_mut::<crate::SocietyRng>()
            .0
            .below(lines.len());
        let line = lines[index].clone();
        say(world, target, actor, &line);
    }
}

/// Writes one [`SocietyBark`](crate::SocietyBark), rendering `{self}`, `{target}` and `{place}`.
fn say(world: &mut World, speaker: Entity, target: Entity, template: &str) {
    let name_of = |entity: Entity| {
        world
            .get::<Name>(entity)
            .map_or_else(|| "someone".to_owned(), |n| n.as_str().to_owned())
    };
    let place = match world.get::<Doing>(speaker).map(|d| d.target) {
        Some(DoingTarget::Place(place)) => world
            .get::<Name>(place)
            .map_or_else(|| "somewhere".to_owned(), |n| n.as_str().to_owned()),
        _ => "somewhere".to_owned(),
    };
    let text = template
        .replace("{self}", &name_of(speaker))
        .replace("{target}", &name_of(target))
        .replace("{place}", &place);
    let feeling = world.get::<Emotions>(speaker).map_or(
        crate::interact::Feeling::Neutral,
        crate::interact::Feeling::from_emotions,
    );
    world.write_message(crate::interact::SocietyBark {
        speaker,
        text,
        thought: false,
        feeling,
    });
}

// -------------------------------------------------------------------------------------------------
// Scripts
// -------------------------------------------------------------------------------------------------

/// Evaluates an interaction's `requires` condition against the tick-start snapshot.
///
/// A missing snapshot (before the first tick) or a failed script counts as "does not hold", which is
/// what the rule set says a failed condition means: it is false, not an error the caller must handle.
fn condition_holds(
    world: &mut World,
    defs: &SocietyDefs,
    condition: crate::script::ScriptId,
    actor: Entity,
    target: Entity,
) -> bool {
    let Some(snapshot) = world
        .get_resource::<crate::snapshot::SnapshotCell>()
        .and_then(|cell| cell.0.clone())
    else {
        return false;
    };
    let engine = match world.remove_resource::<crate::script::ScriptEngine>() {
        Some(engine) => engine,
        None => return false,
    };
    let bindings = crate::script::Bindings {
        actor: person_id(world, actor),
        target: person_id(world, target),
        ..crate::script::Bindings::default()
    };
    let outcome = engine.eval_bool(&defs.scripts, condition, &snapshot, bindings);
    world.insert_resource(engine);
    match outcome {
        Ok(holds) => holds,
        Err(err) => {
            let owner = crate::script::ScriptEngine::owner(&defs.scripts, condition).to_owned();
            error!("bevy_society: script {owner} failed: {err}");
            world.write_message(crate::script::ScriptError {
                owner,
                message: err,
            });
            false
        }
    }
}

/// Runs an effect's script and queues its actions for the single application point.
///
/// The actions are queued rather than applied here: one application point per tick is what keeps a
/// script from changing what another part of the same tick reads.
fn run_effect(world: &mut World, script: crate::script::ScriptId, actor: Entity, target: Entity) {
    let Some(defs) = world.get_resource::<SocietyDefs>().cloned() else {
        return;
    };
    let Some(snapshot) = world
        .get_resource::<crate::snapshot::SnapshotCell>()
        .and_then(|cell| cell.0.clone())
    else {
        return;
    };
    let Some(engine) = world.remove_resource::<crate::script::ScriptEngine>() else {
        return;
    };
    let bindings = crate::script::Bindings {
        actor: person_id(world, actor),
        target: person_id(world, target),
        ..crate::script::Bindings::default()
    };
    let outcome = engine.run(&defs.scripts, script, &snapshot, bindings);
    world.insert_resource(engine);
    match outcome {
        Ok(actions) => {
            if let Some(mut pending) = world.get_resource_mut::<crate::core::PendingActions>() {
                pending.0.extend(actions);
            }
        }
        Err(err) => {
            let owner = crate::script::ScriptEngine::owner(&defs.scripts, script).to_owned();
            error!("bevy_society: script {owner} failed: {err}");
            world.write_message(crate::script::ScriptError {
                owner,
                message: err,
            });
        }
    }
}

/// A person's id as the script bindings expect, or −1 when it is not a live person.
fn person_id(world: &World, entity: Entity) -> i64 {
    world
        .get_resource::<PersonIndex>()
        .and_then(|index| {
            index
                .sorted
                .iter()
                .find(|(_, e)| *e == entity)
                .map(|(id, _)| i64::from(id.0))
        })
        .unwrap_or(-1)
}

// -------------------------------------------------------------------------------------------------
// Activity barks
// -------------------------------------------------------------------------------------------------

/// One thought line per activity start, with `bark_chance` (plan D7).
///
/// Reads the [`ActivityStarted`] messages the decision wrote, so the draw happens once per start in
/// `ActivityStarted` order — the same order every other per-person loop uses.
fn activity_barks(
    mut starts: MessageReader<ActivityStarted>,
    defs: Res<SocietyDefs>,
    settings: Res<SocietySettings>,
    mut rng: ResMut<crate::SocietyRng>,
    names: Query<&Name>,
    doing: Query<&Doing>,
    mut barks: MessageWriter<crate::interact::SocietyBark>,
) {
    for start in starts.read() {
        let Some(spec) = defs.activity_id(&start.activity) else {
            continue;
        };
        let Some(activity) = defs.activities.get(usize::from(spec.0)) else {
            continue;
        };
        if activity.barks.is_empty() || (rng.0.unit() as f32) >= settings.bark_chance {
            continue;
        }
        let line = activity.barks[rng.0.below(activity.barks.len())].clone();
        let of = |entity: Entity| {
            names
                .get(entity)
                .map_or_else(|_| "someone".to_owned(), |n| n.as_str().to_owned())
        };
        let place = doing
            .get(start.person)
            .ok()
            .and_then(|d| match d.target {
                DoingTarget::Place(place) => names.get(place).ok(),
                _ => None,
            })
            .map_or_else(|| "somewhere".to_owned(), |n| n.as_str().to_owned());
        let mut text = line
            .replace("{self}", &of(start.person))
            .replace("{place}", &place);
        if let Some(target) = start.target {
            text = text.replace("{target}", &of(target));
        }
        barks.write(crate::interact::SocietyBark {
            speaker: start.person,
            text,
            thought: true,
            feeling: crate::interact::Feeling::from_emotions(&crate::mind::Emotions::default()),
        });
    }
}

// -------------------------------------------------------------------------------------------------
// Reporting to the trigger queue
// -------------------------------------------------------------------------------------------------

/// Pushes a [`TriggerEvent`](crate::rules) for each exchange and activity this tick, so the trigger
/// queue sees them in the same tick they happened (plan D7).
///
/// Runs after the exchange set finishes. `Option<ResMut<…>>` because the behaviour layer is usable
/// without `RulesPlugin`: a host that took only the mind and behaviour plugins still gets its
/// activities, just no triggers.
fn report_to_triggers(
    mut exchanges: MessageReader<InteractionResolved>,
    mut activities: MessageReader<ActivityStarted>,
    defs: Option<Res<SocietyDefs>>,
    index: Option<Res<PersonIndex>>,
    mut queue: Option<ResMut<crate::rules::TriggerQueue>>,
) {
    let (Some(defs), Some(index), Some(queue)) = (defs, index, queue.as_mut()) else {
        for _ in exchanges.read() {}
        for _ in activities.read() {}
        return;
    };
    let id_of = |entity: Entity| {
        index
            .sorted
            .iter()
            .find(|(_, e)| *e == entity)
            .map_or(-1, |(id, _)| i64::from(id.0))
    };
    for exchange in exchanges.read() {
        let Some(id) = defs.interaction_id(&exchange.interaction) else {
            continue;
        };
        queue.0.push(crate::rules::TriggerEvent::Interaction {
            interaction: id,
            actor: id_of(exchange.actor),
            target: id_of(exchange.target),
            accepted: exchange.accepted,
        });
    }
    for activity in activities.read() {
        let Some(id) = defs.activity_id(&activity.activity) else {
            continue;
        };
        queue.0.push(crate::rules::TriggerEvent::Activity {
            activity: id,
            actor: id_of(activity.person),
            target: activity.target.map_or(-1, id_of),
        });
    }
}

//! Rhai triggers and their evaluation (plan D8, `RulesPlugin`).
//!
//! Two things happen here. First, every system in the tick that has something to report pushes a
//! [`TriggerEvent`] onto the [`TriggerQueue`] resource; the game does the same with a [`FireTrigger`]
//! message, which an ungated drain moves onto the queue. Second, [`evaluate_triggers`] walks the
//! queue in a fixed order and runs every matching trigger's `if` and `then`.
//!
//! Trigger bodies never touch the world. Their `fx` writes are queued and applied by
//! [`crate::apply_script_actions`] at one point per tick, after this set, so a script that changes a
//! tie cannot change what another trigger in the same tick sees.

use std::collections::{BTreeMap, BTreeSet};

use bevy::app::{App, FixedUpdate, Plugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Query, Res, ResMut, SystemParam};
use bevy::ecs::world::World;
use bevy::log::error;
use bevy::transform::components::Transform;

use crate::core::{
    PendingActions, PersonId, PersonIndex, SocietyClock, SocietySettings, SocietySystems,
    require_core,
};
use crate::defs::{
    ActivityId, DefsError, DefsFile, InteractionId, SocietyDefs, TriggerKind, TriggerSpec,
};
use crate::mind::Needs;
use crate::relationships::{TieLabel, TieLabelChanged};
use crate::reputation::NormViolated;
use crate::rumor::RumorBook;
use crate::script::{Bindings, ScriptEngine, ScriptError};
use crate::snapshot::SnapshotCell;

/// A named event the game fires at the society (plan D8): the `Custom` triggers with this name run,
/// with `actor` and `target` bound when the game names them.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FireTrigger {
    /// The trigger name to match.
    pub name: String,
    /// The person the event is about, if any.
    pub actor: Option<Entity>,
    /// The person the event is directed at, if any.
    pub target: Option<Entity>,
}

/// A trigger ran: its event matched, its `area` and `if` held and its `then` finished without error.
///
/// Written once per firing, in the order the triggers ran, so an editor can list which triggers
/// fired against a fast-forwarded society. A trigger whose `then` failed writes a [`ScriptError`]
/// instead. `actor` and `target` are the people the event bound, `None` when it bound none or the
/// person has since despawned.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct TriggerFired {
    /// The authored `TriggerDef::id`.
    pub trigger: String,
    /// The person the event was about, if it bound one.
    pub actor: Option<PersonId>,
    /// The person the event was directed at, if it bound one.
    pub target: Option<PersonId>,
}

/// One event waiting for this tick's trigger evaluation (plan D8).
///
/// Persons are carried as [`PersonId`](crate::PersonId)s, not entities, so an event survives a
/// despawn without resolving to whatever took the entity's slot. A despawned person simply fails to
/// resolve when the trigger runs, and its triggers see `actor = -1`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TriggerEvent {
    /// A custom event, queued from [`FireTrigger`].
    Custom {
        /// The name to match.
        name: String,
        /// The actor, or −1.
        actor: i64,
        /// The target, or −1.
        target: i64,
    },
    /// A person died; the actor binding is the deceased.
    Deceased {
        /// The deceased, by `PersonId`.
        actor: i64,
    },
    /// An interaction resolved (plan D7).
    Interaction {
        /// The interaction's id.
        interaction: InteractionId,
        /// The actor, or −1.
        actor: i64,
        /// The target, or −1.
        target: i64,
        /// Whether the target accepted.
        accepted: bool,
    },
    /// An activity started (plan D7).
    Activity {
        /// The activity's id.
        activity: ActivityId,
        /// The person, or −1.
        actor: i64,
        /// The person it is directed at, or −1.
        target: i64,
    },
    /// A dialogue node was entered (plan D11).
    Dialogue {
        /// The conversation's name.
        conversation: String,
        /// The node's name.
        node: String,
        /// The non-player person, or −1.
        actor: i64,
        /// The player person, or −1.
        target: i64,
    },
}

/// Events waiting for this tick's evaluation, in push order (plan D8).
///
/// Filled by the systems that observe them and by the ungated [`FireTrigger`] drain; drained by
/// [`evaluate_triggers`] whatever else happens, so an event is never replayed on a later tick.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct TriggerQueue(pub(crate) Vec<TriggerEvent>);

/// Which triggers have already fired and which are cooling down (plan D8).
///
/// `once` is keyed by (trigger index, actor), so an actor-less event uses −1 and a `once` trigger on
/// `Tick` fires once for the whole run. `cooldown_hours` keeps the game-day of the last firing under
/// the same key.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct TriggerState {
    /// Triggers that have fired once, by (trigger index, actor).
    fired: BTreeSet<(usize, i64)>,
    /// Last firing day by (trigger index, actor).
    cooling: BTreeMap<(usize, i64), f64>,
}

impl TriggerState {
    fn may_fire(&self, index: usize, actor: i64, spec: &TriggerSpec, now_days: f64) -> bool {
        let key = (index, actor);
        if spec.once && self.fired.contains(&key) {
            return false;
        }
        if spec.cooldown_hours > 0.0
            && let Some(&last) = self.cooling.get(&key)
            && (now_days - last) * 24.0 < f64::from(spec.cooldown_hours)
        {
            return false;
        }
        true
    }

    /// The once-fired keys, for [`capture`](crate::capture).
    #[cfg(feature = "serde")]
    pub(crate) fn fired_set(&self) -> &BTreeSet<(usize, i64)> {
        &self.fired
    }

    /// The cooling map, for [`capture`](crate::capture).
    #[cfg(feature = "serde")]
    pub(crate) fn cooling_map(&self) -> &BTreeMap<(usize, i64), f64> {
        &self.cooling
    }

    /// Replaces both maps wholesale, for [`restore`](crate::restore).
    #[cfg(feature = "serde")]
    pub(crate) fn replace(
        &mut self,
        fired: BTreeSet<(usize, i64)>,
        cooling: BTreeMap<(usize, i64), f64>,
    ) {
        self.fired = fired;
        self.cooling = cooling;
    }

    fn record(&mut self, index: usize, actor: i64, spec: &TriggerSpec, now_days: f64) {
        let key = (index, actor);
        if spec.once {
            self.fired.insert(key);
        }
        if spec.cooldown_hours > 0.0 {
            self.cooling.insert(key, now_days);
        }
    }
}

/// The clock's `now_days` at the previous tick, so `Hour` triggers fire on the crossing rather than
/// for every tick inside the hour.
///
/// **Days, not an hour**, because the previous hour is a whole number only for a clock that steps
/// whole hours: at `dt_days = 1/960` it is `8.002`, and `ceil` of it re-crosses hour 9 on every tick
/// inside hour 9. Keeping the previous *instant* and asking which hour boundaries lie in
/// `(previous, now]` is exact for any step size, which is what makes a fractional `Hour(5.5)` fire at
/// all — the old whole-hour walk could never emit one.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct LastDays(pub(crate) Option<f64>);

/// Which need thresholds are currently armed (plan D8): a `NeedBelow` trigger fires on crossing
/// below and re-arms once the need is back above `below + 0.05`, so a level hovering at the
/// threshold does not fire every tick.
///
/// Indexed by need, then by `NeedBelow` trigger definition order.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct NeedArmed(pub(crate) Vec<Vec<bool>>);

/// The (deed, rumor) pairs whose `RumorReach` trigger has already fired (plan D8).
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct RumorReported(pub(crate) BTreeSet<(u16, u32)>);

/// Applies an edited definitions file to a running society (see [`SocietyDefs::reload`] for what
/// may change) and keeps the `once` and cooldown bookkeeping of every trigger that survives the
/// edit, matched by its authored `id`. A trigger that was renamed or removed forgets its state; a
/// new one starts clean. A refused file leaves the world untouched.
///
/// Takes `&mut World`, so call it between updates (an editor's apply step), not from a system.
pub fn reload_defs(world: &mut World, file: DefsFile) -> Result<(), DefsError> {
    let Some(mut defs) = world.get_resource::<SocietyDefs>().cloned() else {
        return Err(DefsError::Invalid {
            owner: "reload".to_owned(),
            message: "no SocietyDefs resource to reload".to_owned(),
        });
    };
    let old_ids: Vec<String> = defs.triggers.iter().map(|t| t.id.clone()).collect();
    defs.reload(file)?;
    let new_index: BTreeMap<&str, usize> = defs
        .triggers
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();
    let remap = |old: usize| {
        old_ids
            .get(old)
            .and_then(|id| new_index.get(id.as_str()))
            .copied()
    };
    if let Some(mut state) = world.get_resource_mut::<TriggerState>() {
        state.fired = std::mem::take(&mut state.fired)
            .into_iter()
            .filter_map(|(i, actor)| remap(i).map(|n| (n, actor)))
            .collect();
        state.cooling = std::mem::take(&mut state.cooling)
            .into_iter()
            .filter_map(|((i, actor), day)| remap(i).map(|n| ((n, actor), day)))
            .collect();
    }
    world.insert_resource(defs);
    Ok(())
}

/// Runs triggers (plan D8). Every other plugin that observes an event pushes a trigger event onto
/// the trigger queue, and the game fires custom events with [`FireTrigger`]. Requires
/// [`crate::SocietyCorePlugin`].
pub struct RulesPlugin;

impl Plugin for RulesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TriggerQueue>()
            .init_resource::<TriggerState>()
            .init_resource::<LastDays>()
            .init_resource::<NeedArmed>()
            .init_resource::<RumorReported>()
            .add_message::<FireTrigger>()
            .add_message::<TriggerFired>()
            .add_systems(
                FixedUpdate,
                queue_fired_triggers.in_set(SocietySystems::Clock),
            )
            .add_systems(FixedUpdate, evaluate_triggers.in_set(SocietySystems::Rules));
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "RulesPlugin");
    }
}

/// Moves every pending [`FireTrigger`] onto the queue as a [`TriggerEvent::Custom`], in message
/// order. Runs in the ungated clock set: a custom event fired between society ticks must not be
/// dropped by the message buffer.
fn queue_fired_triggers(
    mut reader: MessageReader<FireTrigger>,
    index: Res<PersonIndex>,
    mut queue: Option<ResMut<TriggerQueue>>,
) {
    let Some(queue) = queue.as_mut() else {
        return;
    };
    for msg in reader.read() {
        queue.0.push(TriggerEvent::Custom {
            name: msg.name.clone(),
            actor: id_of(&index, msg.actor),
            target: id_of(&index, msg.target),
        });
    }
}

/// The [`PersonId`](crate::PersonId) of `entity` as an `i64`, or −1 when it is not a live person.
pub(crate) fn id_of(index: &PersonIndex, entity: Option<Entity>) -> i64 {
    let Some(entity) = entity else {
        return -1;
    };
    index
        .sorted
        .iter()
        .find(|(_, e)| *e == entity)
        .map_or(-1, |(id, _)| i64::from(id.0))
}

/// Every threshold a `NeedBelow` trigger names for need `n`, in definition order.
fn thresholds(defs: &SocietyDefs, need: u16) -> Vec<f32> {
    defs.triggers
        .iter()
        .filter_map(|def| match def.on {
            TriggerKind::NeedBelow { need: id, below } if id.0 == need => Some(below),
            _ => None,
        })
        .collect()
}

/// The trigger bookkeeping one tick reads and writes (plan D8).
///
/// Bundled because `evaluate_triggers` already has too many parameters for Bevy's limit once each
/// piece of state is its own argument.
#[derive(SystemParam)]
struct TriggerBookkeeping<'w> {
    queue: Option<ResMut<'w, TriggerQueue>>,
    state: Option<ResMut<'w, TriggerState>>,
    last_days: Option<ResMut<'w, LastDays>>,
    armed: Option<ResMut<'w, NeedArmed>>,
    reported: Option<ResMut<'w, RumorReported>>,
    pending: Option<ResMut<'w, PendingActions>>,
}

/// Evaluates every trigger matching this tick's events (plan D8).
///
/// The event order is fixed so the same inputs give the same society:
/// 1. `Tick`, actor −1;
/// 2. `Hour(h)` for every hour crossed since the previous tick, wrap-aware, actor −1;
/// 3. queued events, in push order;
/// 4. `TieLabelChanged` in message order, matched when the new label is the trigger's;
/// 5. `NormViolated` in message order, actor = actor, target = witness;
/// 6. `NeedBelow` for persons in sorted order and needs in id order;
/// 7. `RumorReach` for rumors in id order, the first time a living rumor of that deed reaches its
///    fraction.
///
/// Within one event, triggers are evaluated in definition order.
#[allow(clippy::too_many_arguments)]
fn evaluate_triggers(
    clock: Res<SocietyClock>,
    defs: Res<SocietyDefs>,
    engine: Option<Res<ScriptEngine>>,
    cell: Res<SnapshotCell>,
    index: Res<PersonIndex>,
    books: Res<RumorBook>,
    mut book: TriggerBookkeeping,
    mut label_changes: MessageReader<TieLabelChanged>,
    mut violations: MessageReader<NormViolated>,
    needs: Query<&Needs>,
    settings: Res<SocietySettings>,
    places: Query<&Transform>,
    memberships: Query<&crate::groups::Memberships>,
    mut errors: MessageWriter<ScriptError>,
    mut fired: MessageWriter<TriggerFired>,
) {
    let (Some(engine), Some(queue), Some(state), Some(pending)) = (
        engine,
        book.queue.as_mut(),
        book.state.as_mut(),
        book.pending.as_mut(),
    ) else {
        return;
    };
    let Some(snapshot) = cell.0.clone() else {
        return;
    };
    let now = clock.now_days;
    let mut events: Vec<(EventKey, Bindings)> = Vec::new();

    events.push((EventKey::Tick, Bindings::of_actor(None)));

    if let Some(last) = book.last_days.as_mut() {
        match last.0 {
            // The baseline tick: it says where the clock starts and fires nothing, so a trigger
            // standing at the start hour does not fire on the first tick of every run.
            None => last.0 = Some(now),
            Some(previous) => {
                if now > previous {
                    events.push((
                        EventKey::Hours {
                            from: previous,
                            to: now,
                        },
                        Bindings::of_actor(None),
                    ));
                }
                last.0 = Some(now);
            }
        }
    }

    events.extend(queue.0.drain(..).map(|event| match event {
        TriggerEvent::Deceased { actor } => (
            EventKey::PersonDied,
            Bindings {
                actor,
                ..Bindings::default()
            },
        ),
        TriggerEvent::Custom {
            name,
            actor,
            target,
        } => (
            EventKey::Custom(name),
            Bindings {
                actor,
                target,
                ..Bindings::default()
            },
        ),
        TriggerEvent::Interaction {
            interaction,
            actor,
            target,
            accepted,
        } => (
            EventKey::Interaction(interaction),
            Bindings {
                actor,
                target,
                accepted: Some(accepted),
                ..Bindings::default()
            },
        ),
        TriggerEvent::Activity {
            activity,
            actor,
            target,
        } => (
            EventKey::Activity(activity),
            Bindings {
                actor,
                target,
                ..Bindings::default()
            },
        ),
        TriggerEvent::Dialogue {
            conversation,
            node,
            actor,
            target,
        } => (
            EventKey::Dialogue(conversation, node),
            Bindings {
                actor,
                target,
                ..Bindings::default()
            },
        ),
    }));

    for change in label_changes.read() {
        events.push((
            EventKey::Label(change.new),
            Bindings {
                actor: id_of(&index, Some(change.from)),
                target: id_of(&index, Some(change.to)),
                ..Bindings::default()
            },
        ));
    }

    for violation in violations.read() {
        events.push((
            EventKey::Violation(defs.deed_id(&violation.deed).map(|d| d.0)),
            Bindings {
                actor: id_of(&index, Some(violation.actor)),
                target: id_of(&index, Some(violation.witness)),
                ..Bindings::default()
            },
        ));
    }

    // Needs crossing below, in sorted person order and need order.
    let need_count = defs.needs.len();
    for n in 0..need_count {
        let below = thresholds(&defs, n as u16);
        if below.is_empty() {
            continue;
        }
        let Some(armed) = book.armed.as_mut() else {
            break;
        };
        if armed.0.len() < need_count {
            armed.0.resize(need_count, Vec::new());
        }
        if armed.0[n].len() != below.len() {
            armed.0[n] = vec![true; below.len()];
        }
        for &(_, entity) in &index.sorted {
            let Ok(levels) = needs.get(entity) else {
                continue;
            };
            let level = levels.level(n);
            for (slot, &threshold) in below.iter().enumerate() {
                let is_armed = armed.0[n][slot];
                if level < threshold && is_armed {
                    armed.0[n][slot] = false;
                    events.push((
                        EventKey::Need(n as u16, threshold),
                        Bindings {
                            actor: id_of(&index, Some(entity)),
                            ..Bindings::default()
                        },
                    ));
                } else if level > threshold + 0.05 {
                    armed.0[n][slot] = true;
                }
            }
        }
    }

    // Rumor reach, in rumor order.
    if let Some(reported) = book.reported.as_mut() {
        let living = index.sorted.len();
        for (i, rumor) in books.rumors.iter().enumerate() {
            if !rumor.alive || living == 0 {
                continue;
            }
            let reach = rumor.known_by as f32 / living as f32;
            let id = crate::rumor::RumorId(i as u32);
            for def in &defs.triggers {
                let TriggerKind::RumorReach { deed, fraction } = def.on else {
                    continue;
                };
                if deed != rumor.deed || reach < fraction {
                    continue;
                }
                if !reported.0.insert((deed.0, id.0)) {
                    continue;
                }
                events.push((
                    EventKey::RumorReach(deed.0),
                    Bindings {
                        subject: id_of(&index, Some(rumor.subject)),
                        rumor: i64::from(id.0),
                        ..Bindings::default()
                    },
                ));
            }
        }
    }

    for (event, bindings) in events {
        for (which, spec) in defs.triggers.iter().enumerate() {
            if !event.matches(&spec.on) || !state.may_fire(which, bindings.actor, spec, now) {
                continue;
            }
            if (!spec.factions.is_empty() || !spec.persons.is_empty())
                && !actor_selected(&index, &memberships, bindings.actor, spec)
            {
                continue;
            }
            if let Some(area) = spec.area
                && !actor_in_area(&index, &places, &settings, bindings.actor, area)
            {
                continue;
            }
            if let Some(condition) = spec.condition
                && !condition_holds(&engine, &defs, condition, &snapshot, bindings, &mut errors)
            {
                continue;
            }
            match engine.run(&defs.scripts, spec.then, &snapshot, bindings) {
                Ok(actions) => {
                    pending.0.extend(actions);
                    fired.write(TriggerFired {
                        trigger: spec.id.clone(),
                        actor: person_id(bindings.actor),
                        target: person_id(bindings.target),
                    });
                }
                Err(err) => report(&defs, spec.then, err, &mut errors),
            }
            state.record(which, bindings.actor, spec, now);
        }
    }
}

/// The [`PersonId`] a binding carries, `None` for the `-1` of "no person".
fn person_id(bound: i64) -> Option<PersonId> {
    u32::try_from(bound).ok().map(PersonId)
}

/// Whether the actor is one of the trigger's `persons` or belongs to one of its `factions`; a
/// trigger that lists both takes either. No such person is not selected.
fn actor_selected(
    index: &PersonIndex,
    memberships: &Query<&crate::groups::Memberships>,
    actor: i64,
    spec: &crate::defs::TriggerSpec,
) -> bool {
    let Some(&(id, entity)) = index.sorted.iter().find(|(id, _)| i64::from(id.0) == actor) else {
        return false;
    };
    spec.persons.contains(&id.0)
        || memberships
            .get(entity)
            .is_ok_and(|m| spec.factions.iter().any(|f| m.contains(*f)))
}

/// Whether the person with id `actor` stands inside `area` now. No such person is outside.
fn actor_in_area(
    index: &PersonIndex,
    transforms: &Query<&Transform>,
    settings: &SocietySettings,
    actor: i64,
    area: crate::defs::TriggerArea,
) -> bool {
    let Some(&(_, entity)) = index.sorted.iter().find(|(id, _)| i64::from(id.0) == actor) else {
        return false;
    };
    let Ok(transform) = transforms.get(entity) else {
        return false;
    };
    let p = settings.plane.project(transform.translation);
    area.contains([p.x, p.y])
}

/// Whether a trigger's `if` holds. An error is an `error!` and a `ScriptError`, and the condition
/// counts as false (plan D8).
fn condition_holds(
    engine: &ScriptEngine,
    defs: &SocietyDefs,
    condition: crate::script::ScriptId,
    snapshot: &std::sync::Arc<crate::snapshot::SocietySnapshot>,
    bindings: Bindings,
    errors: &mut MessageWriter<ScriptError>,
) -> bool {
    match engine.eval_bool(&defs.scripts, condition, snapshot, bindings) {
        Ok(holds) => holds,
        Err(err) => {
            report(defs, condition, err, errors);
            false
        }
    }
}

/// Logs a script failure and writes its `ScriptError`, naming the authored path that failed.
pub(crate) fn report(
    defs: &SocietyDefs,
    which: crate::script::ScriptId,
    err: String,
    errors: &mut MessageWriter<ScriptError>,
) {
    let owner = ScriptEngine::owner(&defs.scripts, which).to_owned();
    error!("bevy_society: script {owner} failed: {err}");
    errors.write(ScriptError {
        owner,
        message: err,
    });
}

/// The part of an event a trigger's `on` is matched against.
enum EventKey {
    Tick,
    /// A span of game time `(from, to]` in days, during which some hour boundary may have been crossed.
    /// One event per tick rather than one per hour: a host clock that jumps from 05:00 to 06:00 in one
    /// frame still has to fire an `Hour` trigger once, and `matches` asks the question per trigger so
    /// the answer is about the trigger's own hour, fractional or not.
    Hours {
        from: f64,
        to: f64,
    },
    Custom(String),
    /// A person died; the bindings' actor is the deceased.
    PersonDied,
    Interaction(InteractionId),
    Activity(ActivityId),
    Label(TieLabel),
    Violation(Option<u16>),
    Dialogue(String, String),
    Need(u16, f32),
    RumorReach(u16),
}

impl EventKey {
    fn matches(&self, on: &TriggerKind) -> bool {
        match (self, on) {
            (Self::Tick, TriggerKind::Tick) => true,
            // Does the hour `want` lie in `(from, to]`? Scaled to hours, `want` is crossed when
            // `floor((to*24 - want)/24) > floor((from*24 - want)/24)` — the count of whole days between
            // `want` and the instant, which increments exactly once as the clock passes `want` and
            // never again until the next day. Written this way it holds for a fractional `want`, for a
            // span longer than a day (a slow tick, a host jump across midnight), and for a span that
            // crosses midnight going up.
            (Self::Hours { from, to }, TriggerKind::Hour(want)) => {
                let want = f64::from(*want);
                ((to * 24.0 - want) / 24.0).floor() > ((from * 24.0 - want) / 24.0).floor()
            }
            (Self::Custom(name), TriggerKind::Custom(want)) => name == want,
            (Self::PersonDied, TriggerKind::PersonDied) => true,
            (Self::Interaction(id), TriggerKind::Interaction(want)) => id == want,
            (Self::Activity(id), TriggerKind::Activity(want)) => id == want,
            (Self::Label(label), TriggerKind::TieLabel(want)) => label == want,
            (Self::Violation(deed), TriggerKind::NormViolated(want)) => deed == &Some(want.0),
            (
                Self::Dialogue(conversation, node),
                TriggerKind::DialogueNode {
                    conversation: want_conversation,
                    node: want_node,
                },
            ) => conversation == want_conversation && node == want_node,
            (Self::Need(n, threshold), TriggerKind::NeedBelow { need, below }) => {
                *n == need.0 && (threshold - below).abs() < 1e-6
            }
            (Self::RumorReach(deed), TriggerKind::RumorReach { deed: want, .. }) => *deed == want.0,
            _ => false,
        }
    }
}

/// What [`test_trigger`] found: whether one trigger would run for the given people, and what its
/// `then` would write.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerTest {
    /// The actor passes the trigger's `factions` / `persons` filter (true when it lists none).
    pub selected: bool,
    /// The actor stands inside the trigger's `area` (true when it has none).
    pub in_area: bool,
    /// Whether the `if` script held; `None` when the trigger has no `if`, or an earlier check
    /// already refused it so the `if` did not run.
    pub condition: Option<bool>,
    /// The trigger would fire: selected, in its area, its `if` held and its `then` ran cleanly.
    /// The `once` and cooldown bookkeeping is not consulted: this asks about the script, not the
    /// history.
    pub would_fire: bool,
    /// Each write the `then` queued, in order, as the `Debug` text of the action (for example
    /// `Adjust { person: 0, other: 1, field: "trust", delta: 0.2 }`). Empty unless `would_fire`.
    pub effects: Vec<String>,
}

/// Why [`test_trigger`] could not answer.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerTestError {
    /// The world has no compiled definitions or script engine: the society plugins are not added.
    NoSociety,
    /// No society tick has run, so there is no snapshot for the script to read.
    NoSnapshot,
    /// No trigger has this authored `id`.
    UnknownTrigger(String),
    /// A script failed at run time (a missing person, a wrong type, a sandbox limit).
    Script {
        /// The authored path of the script that failed.
        owner: String,
        /// The engine's message.
        message: String,
    },
}

impl std::fmt::Display for TriggerTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSociety => write!(f, "no SocietyDefs or ScriptEngine: add SocietyPlugins"),
            Self::NoSnapshot => write!(
                f,
                "no snapshot yet: run a society tick (or fast_forward) before testing a trigger"
            ),
            Self::UnknownTrigger(id) => write!(f, "no trigger with id {id:?}"),
            Self::Script { owner, message } => write!(f, "script {owner} failed: {message}"),
        }
    }
}

impl std::error::Error for TriggerTestError {}

/// Dry-runs the trigger authored as `trigger` against the society as the last tick froze it, as if
/// its event had just happened to `actor` and `target`, and reports what it would write without
/// writing it.
///
/// Nothing changes: the trigger's `once` and cooldown state, the pending actions, the messages
/// and the world are all untouched, and the event the trigger listens for is not required to have
/// happened. Call it between updates, after a [`fast_forward`](crate::fast_forward), to see how an
/// edited script would behave on the society it will run in.
///
/// # Errors
/// [`TriggerTestError`] when the society is not installed, no tick has run, `trigger` is not an
/// authored id, or a script fails.
pub fn test_trigger(
    world: &mut World,
    trigger: &str,
    actor: Option<PersonId>,
    target: Option<PersonId>,
) -> Result<TriggerTest, TriggerTestError> {
    let (Some(defs), Some(engine)) = (
        world.get_resource::<SocietyDefs>(),
        world.get_resource::<ScriptEngine>(),
    ) else {
        return Err(TriggerTestError::NoSociety);
    };
    let snapshot = world
        .get_resource::<SnapshotCell>()
        .and_then(|cell| cell.0.clone())
        .ok_or(TriggerTestError::NoSnapshot)?;
    let spec = defs
        .triggers
        .iter()
        .find(|t| t.id == trigger)
        .ok_or_else(|| TriggerTestError::UnknownTrigger(trigger.to_owned()))?;
    let bindings = Bindings {
        actor: actor.map_or(-1, |p| i64::from(p.0)),
        target: target.map_or(-1, |p| i64::from(p.0)),
        ..Bindings::default()
    };
    let failed = |script, message| TriggerTestError::Script {
        owner: ScriptEngine::owner(&defs.scripts, script).to_owned(),
        message,
    };

    let entity_of = |who: Option<PersonId>| {
        let id = who?;
        world
            .get_resource::<PersonIndex>()?
            .sorted
            .iter()
            .find(|(p, _)| *p == id)
            .map(|(_, e)| *e)
    };
    let actor_entity = entity_of(actor);
    let selected = (spec.factions.is_empty() && spec.persons.is_empty())
        || actor_entity.is_some_and(|e| {
            actor.is_some_and(|p| spec.persons.contains(&p.0))
                || world
                    .get::<crate::groups::Memberships>(e)
                    .is_some_and(|m| spec.factions.iter().any(|f| m.contains(*f)))
        });
    let in_area = spec.area.is_none_or(|area| {
        let (Some(e), Some(settings)) = (actor_entity, world.get_resource::<SocietySettings>())
        else {
            return false;
        };
        world.get::<Transform>(e).is_some_and(|t| {
            let p = settings.plane.project(t.translation);
            area.contains([p.x, p.y])
        })
    });

    let mut result = TriggerTest {
        selected,
        in_area,
        condition: None,
        would_fire: false,
        effects: Vec::new(),
    };
    if !selected || !in_area {
        return Ok(result);
    }
    if let Some(condition) = spec.condition {
        let holds = engine
            .eval_bool(&defs.scripts, condition, &snapshot, bindings)
            .map_err(|e| failed(condition, e))?;
        result.condition = Some(holds);
        if !holds {
            return Ok(result);
        }
    }
    let actions = engine
        .run(&defs.scripts, spec.then, &snapshot, bindings)
        .map_err(|e| failed(spec.then, e))?;
    result.would_fire = true;
    result.effects = actions.iter().map(|a| format!("{a:?}")).collect();
    Ok(result)
}

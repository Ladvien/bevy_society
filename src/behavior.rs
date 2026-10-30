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
use crate::rumor::RumorBook;

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

impl Doing {
    /// Whether the person is still walking to the activity's target, and so is being paid nothing yet.
    ///
    /// The activity's needs and duration start counting only once `arrived_day` is set, so a reader that
    /// wants "what is this person doing **now**" must check this first. [`Doing::label`] does it for you.
    pub fn is_travelling(&self) -> bool {
        self.arrived_day.is_none()
    }

    /// The honest answer to "what is this person doing?", for a HUD, a log or a script.
    ///
    /// `Doing::activity` names the activity that was **chosen**, from the moment it is chosen, so reading it
    /// raw reports a person as asleep while it is still walking to bed. Measured on the shipped village at
    /// 64 s/day, that over-reports sleep by more than three points of every person-tick. The convention here is
    /// `"travelling:<activity>"` while `arrived_day` is `None`, and the bare activity name once it is set.
    ///
    /// This is the convention the crate's own evidence harness had to invent privately before it existed,
    /// which is the argument for owning it here: a host that reads `activity` alone gets a number nobody can
    /// interpret, and the fix should not depend on every consumer rediscovering the same workaround.
    pub fn label(&self, defs: &crate::defs::SocietyDefs) -> String {
        let base = defs.activity_name(self.activity);
        if self.is_travelling() {
            format!("travelling:{base}")
        } else {
            base.to_owned()
        }
    }
}

/// Where a travelling person is walking to, on the society plane.
///
/// Inserted and updated while travelling and removed on arrival or when the activity ends. A host
/// that does not steer moves people with [`Tuning::walk_speed`](crate::Tuning) — see
/// `walk_toward_destination`. A host that adds a `SocialMover` per person steers them itself and this
/// system leaves them alone.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Default)]
#[reflect(Component)]
pub struct Destination(pub Vec2);

/// How close a travelling person has come to its target, and when it last got meaningfully closer
/// (`Tuning::stall_timeout_hours`, issue #26).
///
/// Written by `arrive_and_satisfy` while a person travels and only when a stall timeout is set; read by
/// `end_activities`. Removed on arrival, on every ending and whenever a new `Doing` is decided, so it
/// never outlives the walk it measures. It is not saved: a restored walker gets a fresh window.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct TravelProgress {
    /// The smallest distance to the target at the last improvement, in plane units.
    best: f32,
    /// Game day of the last improvement by more than `Tuning::stall_epsilon`.
    improved_day: f64,
}

/// Places a person recently failed to reach, and the game-day each is forgiven
/// (`Tuning::failed_place_memory_hours`).
///
/// Written by `end_activities` when a walk to a `Place` ends before arrival; read and pruned by
/// `decide_activities`. Kept sorted by insertion, so it never depends on hash order. Not saved.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub(crate) struct FailedPlaces(Vec<(Entity, f64)>);

/// People this person has just harmed, and the game-day each may be harmed again
/// (`Tuning::retaliation_refractory_hours`).
///
/// Written by `exchange` when an interaction whose deed has negative harm is attempted (accepted or
/// refused); read by `decide_activities`, which drops those people from the targets of a harmful
/// activity. Pruned on every write, in insertion order. Saved (`PersonSave::recent_harm`): a reload that
/// forgot it would let the next tick's insult through a refractory the uninterrupted run was still holding.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub(crate) struct RecentHarm(pub(crate) Vec<(Entity, f64)>);

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
///
/// Written when the person **decides**, which is the same instant its [`Doing`] is inserted and
/// `Doing::started_day` is stamped — not when it arrives. The pair therefore brackets the component:
/// one [`ActivityStarted`] when `Doing` appears, one [`ActivityEnded`] when it is removed, including
/// when a walk times out and the activity never runs. A host may read an end as the close of
/// something it was told about, whatever the reason it closed.
///
/// Arrival is a separate fact: it begins the activity's duration and its `satisfies` payout, and is
/// visible on [`Doing::arrived_day`].
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
            .init_resource::<EndRequests>()
            .add_message::<TravelFailed>()
            .add_message::<PersonDied>()
            .init_resource::<DeathQueue>()
            .add_observer(on_deceased_added)
            .add_message::<EndActivity>()
            .add_message::<ReportDeed>()
            .init_resource::<DeedReportQueue>()
            .add_systems(
                FixedUpdate,
                queue_reported_deeds.in_set(crate::SocietySystems::Clock),
            )
            .add_systems(
                FixedUpdate,
                // Ungated, in `Clock`: a host writes `TravelFailed` whenever it likes, and a
                // message written between two society ticks must survive to the tick that reads it.
                queue_end_requests.in_set(crate::SocietySystems::Clock),
            )
            .add_systems(
                FixedUpdate,
                (end_activities, decide_activities)
                    .chain()
                    .in_set(crate::SocietySystems::Decide),
            )
            .add_systems(
                FixedUpdate,
                // The walk runs every fixed step, not only on society ticks, so motion stays smooth
                // while the simulation ticks slowly — the same rule `steer_movers` follows.
                walk_toward_destination.after(crate::SocietySystems::Rules),
            )
            .add_systems(
                FixedUpdate,
                // **Before `apply_appraisals`**, so the grief a death produces counts in the tick it
                // happened. `apply_appraisals` is in `Clock`, which is earlier than `Interact`, so this
                // joins `Clock` and orders itself against that system explicitly.
                announce_deaths
                    .in_set(crate::SocietySystems::Clock)
                    .before(crate::mind::apply_appraisals),
            )
            .add_systems(
                FixedUpdate,
                (
                    arrive_and_satisfy,
                    resolve_interactions,
                    resolve_reported_deeds,
                )
                    .chain()
                    .in_set(crate::SocietySystems::Interact),
            )
            .add_systems(
                FixedUpdate,
                activity_barks.in_set(crate::SocietySystems::Interact),
            )
            // Contact gossip reads the contact list `expose_contacts` has just written, so it runs in
            // `Relate` and queues into the same `InteractQueue` the arrival path uses.
            .add_systems(
                FixedUpdate,
                contact_gossip.in_set(crate::SocietySystems::Relate),
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
        #[cfg(not(feature = "fields"))]
        {
            warn_unscored_fields(defs);
        }
        warn_short_travel_reach(&settings);
        warn_walk_outlasts_activity(defs, &settings);
        warn_unbalanced_needs(defs);
        warn_unmade_goods(defs);
        app.add_systems(Startup, warn_unreachable_places);
    }
}

/// Warns about a need whose best source cannot cover its decay at a plausible rate.
///
/// Every need decays per game-day and is supplied by activities that pay `satisfies · duration_hours`
/// per use. A need whose source pays less than its decay needs more than one use a day; one whose
/// source is also gated to a few hours, or sits behind a walk, will starve in practice. Measured on the
/// starter pack: `fun` decays 0.8/day, its only source `relax` pays 0.6 per use, and a person therefore
/// needs 1.33 of them a day — while a 24-person village manages 0.10, so `fun` sits at zero for a third
/// of every run.
///
/// The threshold is **four uses a day**, not one. A cheap `Person`-targeted source that pays little is
/// fine — `social` needs 2.78 chats a day and gets them, because the other person walks to the actor
/// and a chat is three ticks. The crate cannot tell from definitions alone which sources are expensive
/// to reach, so it warns only where the count is implausible for any activity shape.
fn warn_unbalanced_needs(defs: &SocietyDefs) {
    let mut starving: Vec<String> = Vec::new();
    for (index, need) in defs.needs.iter().enumerate() {
        let Some(best) = defs
            .activities
            .iter()
            .filter_map(|activity| {
                let rate = activity
                    .satisfies
                    .iter()
                    .find(|(slot, _)| usize::from(slot.0) == index)
                    .map(|(_, rate)| *rate)?;
                Some((activity.name.as_str(), rate * activity.duration_hours))
            })
            .fold(None::<(&str, f32)>, |best, candidate| match best {
                Some((_, amount)) if amount >= candidate.1 => best,
                _ => Some(candidate),
            })
        else {
            starving.push(format!("{} (no activity satisfies it)", need.name));
            continue;
        };
        if best.1 <= 0.0 {
            starving.push(format!("{} ({} supplies nothing)", need.name, best.0));
            continue;
        }
        let uses_per_day = need.decay_per_day / best.1;
        if uses_per_day > MAX_USES_PER_DAY {
            starving.push(format!(
                "{} (best source {} pays {:.2} per use against {:.2}/day decay, so {:.2} uses a day)",
                need.name, best.0, best.1, need.decay_per_day, uses_per_day
            ));
        }
    }
    if starving.is_empty() {
        return;
    }
    warn!(
        "bevy_society: {} needs cannot be met by a plausible number of activities: {}. A need whose \
         best source supplies less than a quarter of its daily decay will starve whatever the choice \
         rule does.",
        starving.len(),
        starving.join("; ")
    );
}

/// The most uses a day a need's best source may require before the crate says it is unbalanced.
///
/// `social` sits at 2.78 and is fine; a source behind a walk and an hour window at that count is not.
const MAX_USES_PER_DAY: f32 = 4.0;

/// Warns about a good some activity **consumes** and no activity in the pack **produces**.
///
/// A pack can name a good and use it without ever making it, and until `satisfies` started paying for
/// what a `consumes` actually found, that was invisible: every meal was paid in full from a shelf that
/// could never hold anything, so the village looked fed and the larder was a fiction. Now it means a
/// need nobody can ever meet — the worst kind of silent, because the decision rule keeps choosing the
/// activity and the host sees a store that is always empty.
///
/// Measured on this crate's own starter pack, which declares `bread` and has `eat` consume it with no
/// producer in the pack: a host built from `DefsFile::starter()` alone had a village that could never
/// hold a loaf, and nothing said so until a run was read (`docs/loop/runs/2026-10-26-12/`).
///
/// A warning rather than an error: a host may supply the producer in a second pack it merges in, and a
/// good with no producer is legitimate for a pack that only *reads* one — a market whose goods arrive
/// from elsewhere. The crate cannot tell those apart, and refusing to compile would break the second.
fn warn_unmade_goods(defs: &SocietyDefs) {
    let mut unmade: Vec<String> = Vec::new();
    for (index, good) in defs.goods.iter().enumerate() {
        let made = defs.activities.iter().any(|activity| {
            activity
                .produces
                .iter()
                .any(|&(slot, amount)| usize::from(slot.0) == index && amount > 0.0)
        });
        if made {
            continue;
        }
        let used = defs
            .activities
            .iter()
            .filter(|activity| {
                activity
                    .consumes
                    .iter()
                    .any(|&(slot, _)| usize::from(slot.0) == index)
            })
            .count();
        if used > 0 {
            unmade.push(format!(
                "{} ({used} activit{} consume it, none produces it)",
                good,
                if used == 1 { "y" } else { "ies" }
            ));
        }
    }
    if unmade.is_empty() {
        return;
    }
    warn!(
        "bevy_society: {} good{} no activity produces: {}. A `consumes` that nothing makes can never \
         be met, so the activities that pay `satisfies` for it will pay nothing and the store will stay \
         empty for the whole run. Add a `produces:` to some activity, or merge in the pack that makes \
         it.",
        unmade.len(),
        if unmade.len() == 1 { "" } else { "s" },
        unmade.join("; ")
    );
}

/// Warns that `travel_timeout_hours` and the day length disagree badly enough that walks will time out.
///
/// Two clocks meet at a walk: [`SocialMover::speed`](crate::SocialMover) is metres per **real** second
/// applied at the fixed step, while `travel_timeout_hours` is **game** hours. A host sets both, and
/// nothing told it they must agree. The coupling is `needs-decision/travel-time-in-two-clocks.md`,
/// still open — but the *silence* is not a decision the crate may defer, because it can compute how
/// much walking a timeout buys.
///
/// Measured on the project's own village at `seconds_per_day = 4`, where a 40-hour timeout is 426
/// ticks: an 8.2 m walk takes 308 ticks and fits, and the 13 m corner takes **527** and does not. The
/// far corner's people always time out, 78 of 108 `eat`s were interrupted, and `food` sat at 0.000
/// for the whole run.
///
/// The warning fires only when the timeout buys less than `MIN_WALK_METRES`, a distance no village
/// layout can be smaller than and still be a village. Above that the crate stays quiet: the reachable
/// distance depends on the host's layout, which `finish` cannot see.
fn warn_short_travel_reach(settings: &SocietySettings) {
    // **Walking is the host's under `Locomotion::Host`, and there may be no timeout at all.** Either
    // way this check has nothing to say: its whole subject is a crate walk bounded by a crate timeout.
    if settings.locomotion != crate::Locomotion::Builtin {
        return;
    }
    let Some(timeout_hours) = settings.tuning.travel_timeout_hours else {
        return;
    };
    let seconds_per_tick = f64::from(settings.tick_secs);
    let day_seconds = f64::from(settings.seconds_per_day);
    if seconds_per_tick <= 0.0 || day_seconds <= 0.0 {
        return;
    }
    let ticks_available = f64::from(timeout_hours) / 24.0 / (seconds_per_tick / day_seconds);
    // The default walking speed. Named here because the warning must speak for the default a host
    // gets; a host that sets its own `SocialMover::speed` is beyond what this check can see.
    let speed = 1.4_f64;
    let metres = speed * seconds_per_tick * ticks_available;
    if metres >= MIN_WALK_METRES {
        return;
    }
    warn!(
        "bevy_society: a travel_timeout_hours of {} at seconds_per_day {} allows {:.0} ticks, which at \
         {speed} m/s is only {:.1} metres of walking — less than {MIN_WALK_METRES:.0} m. People whose \
         activity is further away will always time out, and the needs those activities satisfy will \
         never be met. Raise travel_timeout_hours or shorten the walks.",
        timeout_hours, settings.seconds_per_day, ticks_available, metres
    );
}

/// Warns that `seconds_per_day` makes a typical walk longer than the activities it reaches.
///
/// The same two clocks meet, and this is the consequence the timeout warning cannot see: a walk costs
/// `seconds_per_day`, but every `duration_hours` is fixed in game time. So the **pacing knob rescales
/// every journey against every activity**, and the society changes when a host only meant to change how
/// fast it watches.
///
/// Measured on `village-evening`, seed 7, the same 16.4 game-days and the same 24 people, with only
/// `seconds_per_day` varied:
///
/// | `seconds_per_day` | travelling | need mean |
/// |---|---|---|
/// | 64 | **58.9%** | 0.545 |
/// | 144 | 50.4% | 0.603 |
/// | 480 | **43.9%** | 0.624 |
///
/// Fifteen points of a villager's life, from a setting that reads like a rendering speed. The crate's
/// own rule is that every rate is per game-day; walking is the exception, so the walk must be checked
/// against the activities rather than assumed consistent.
///
/// The check compares `MIN_WALK_METRES` of walking against the **mean** activity duration. The mean, not
/// the median, because a day's balance is carried by its long activities: the starter pack's median is
/// 0.5 h, which a 240-second day's 0.71 h walk already exceeds, yet that day is healthy (`food` 0.494,
/// needs 0.621). Over the mean the check separates the two cases that matter — the project's own
/// `village-evening` at 64 s/day, where a 10 m walk costs 2.68 game-hours against a 1.61 h mean and
/// villagers spend **58.9%** of their ticks travelling; and the shipped 240-second default, where the
/// walk is 0.71 h and the same measure is 46.8% with every need met.
fn warn_walk_outlasts_activity(defs: &SocietyDefs, settings: &SocietySettings) {
    // Walking is the host's under `Locomotion::Host`: how long a journey takes is its business, not a
    // consequence of `seconds_per_day`, so this warning would be about a walk the crate never takes.
    if settings.locomotion != crate::Locomotion::Builtin {
        return;
    }
    let day_seconds = f64::from(settings.seconds_per_day);
    if day_seconds <= 0.0 {
        return;
    }
    let durations: Vec<f64> = defs
        .activities
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != usize::from(crate::defs::IDLE_ACTIVITY.0))
        .map(|(_, activity)| f64::from(activity.duration_hours))
        .collect();
    if durations.is_empty() {
        return;
    }
    let mean = durations.iter().sum::<f64>() / durations.len() as f64;
    // The default walking speed, for the same reason `warn_short_travel_reach` names it: the warning
    // speaks for the default a host gets, and cannot see a host's own `SocialMover::speed`.
    let speed = 1.4_f64;
    let walk_hours = MIN_WALK_METRES / speed / day_seconds * 24.0;
    if walk_hours <= mean {
        return;
    }
    warn!(
        "bevy_society: at seconds_per_day {} a {MIN_WALK_METRES:.0} m walk costs {walk_hours:.2} game-hours, \
         longer than the mean activity ({mean:.2}h) — people will spend most of the day walking to \
         things that are over before they arrive, and the needs those activities satisfy will lag. This is \
         the coupling `needs-decision/travel-time-in-two-clocks.md` records: walking is in real time and \
         durations are in game time. Lengthen the activities, raise seconds_per_day, or move the places \
         closer together.",
        settings.seconds_per_day
    );
}

/// The shortest walk a village layout can reasonably ask for, in metres. Below this a timeout is
/// certainly too short; above it the crate cannot tell, because it does not know the layout.
const MIN_WALK_METRES: f64 = 10.0;

/// Warns when `travel_timeout_hours` cannot cover a walk between two places the world actually has.
///
/// [`warn_short_travel_reach`] has to stay quiet above `MIN_WALK_METRES` because `finish` cannot see the
/// layout. **A system can.** Every `Place` is an entity with a `Transform`, so at `Startup` the crate knows
/// exactly how far apart the places are and can say whether the timeout reaches them.
///
/// Measured on the project's own `village-evening`: a 4-game-hour timeout at 64 s/day buys **14.9 m**, the
/// farthest place-to-place walk in that village is **16.4 m**, and **150 of 400** interrupted activities
/// were `sleep` walks that ran out of time before arrival — every one of them `timed_out=true`. The old
/// warning stayed quiet throughout, because 14.9 m is above its 10 m threshold. Being above a generic floor
/// and being able to reach your own tavern are different things.
fn warn_unreachable_places(
    settings: Res<SocietySettings>,
    places: Query<(Entity, &Name, &Transform), With<crate::core::Place>>,
    homes: Query<&crate::Home>,
) {
    if settings.locomotion != crate::Locomotion::Builtin {
        return;
    }
    let Some(timeout_hours) = settings.tuning.travel_timeout_hours else {
        return;
    };
    let seconds_per_tick = f64::from(settings.tick_secs);
    let day_seconds = f64::from(settings.seconds_per_day);
    if seconds_per_tick <= 0.0 || day_seconds <= 0.0 {
        return;
    }
    let ticks_available = f64::from(timeout_hours) / 24.0 / (seconds_per_tick / day_seconds);
    // The default walking speed, for the same reason `warn_short_travel_reach` names it.
    let speed = 1.4_f64;
    let reachable = speed * seconds_per_tick * ticks_available;

    let plane = settings.plane;
    let mut worst = 0.0_f32;
    let mut worst_pair = (String::new(), String::new());
    for (_, name, transform) in &places {
        for (_, other_name, other) in &places {
            let here = plane.project(transform.translation);
            let there = plane.project(other.translation);
            let distance = here.distance(there);
            if distance > worst {
                worst = distance;
                worst_pair = (name.as_str().to_owned(), other_name.as_str().to_owned());
            }
        }
    }
    // **Homes holding more people than their arrival disc can seat.** Everyone goes to bed at the same hour,
    // so every resident of one place aims at the same coordinate inside `interaction_radius`. ORCA holds two
    // centres one body diameter apart, so the disc packs solid and the queue behind it times out. Measured on
    // the project's own village: **150 sleeps interrupted**, and at every one the goal already held a median of
    // **6** bodies — exactly that home's declared capacity.
    let mut residents: std::collections::BTreeMap<Entity, usize> =
        std::collections::BTreeMap::new();
    for home in &homes {
        *residents.entry(home.0).or_insert(0) += 1;
    }
    let body_diameter = 0.7_f32;
    for (entity, name, _) in &places {
        let Some(&count) = residents.get(&entity) else {
            continue;
        };
        // How many bodies the arrival disc can seat, and the radius that would seat them all. Hex
        // packing overestimates what ORCA manages (everyone aims at the centre rather than around it), so
        // this is a lower bound on the crowding, not a simulation of it.
        let seats = (settings.interaction_radius / body_diameter).powi(2);
        if (count as f32) <= seats || count < 3 {
            continue;
        }
        let needed = body_diameter * (count as f32).sqrt();
        warn!(
            "bevy_society: '{place}' is home to {count} people, but an arrival disc of radius {radius} seats \
             about {seats}. They all go to bed at the same hour, so the queue behind the disc times out and \
             their sleeps are interrupted. Give them more room: raise `interaction_radius` to about {needed:.1} \
             or spread the residents over more homes.",
            place = name.as_str(),
            radius = settings.interaction_radius,
            seats = seats as usize,
            needed = needed,
        );
    }
    if worst <= reachable as f32 || places.iter().len() < 2 {
        return;
    }
    warn!(
        "bevy_society: a travel_timeout_hours of {timeout_hours} at seconds_per_day {} buys {reachable:.1}          metres of walking, but the farthest pair of places is '{a}' -> '{b}' at {worst:.1} m. Every walk          between them times out before arrival, so the activities that need one end interrupted and the          needs they satisfy are never met. Raise travel_timeout_hours to at least {needed:.1}, or move the          places closer.",
        settings.seconds_per_day,
        a = worst_pair.0,
        b = worst_pair.1,
        needed = f64::from(worst) * 24.0 / (speed * day_seconds),
    );
}

/// Warns when a definition reads a social field that **cannot be populated**.
///
/// `Signal::Field` and `ActivityTarget::Gradient` were both dead: the signal returned a constant `0.0`
/// and the gradient produced no targets, so a definition using either compiled and could never be
/// chosen. Both are wired now — measured, an activity whose only consideration is `Field(Crowd)` is
/// chosen 6 times in 2 400 ticks against a 1-hour fallback.
///
/// What remains worth warning about is the **feature being off**. The grids are built by
/// `SocialFieldPlugin`, which only exists with the `fields` feature; without it any `Field` definition
/// is still dead, and the definitions file accepts it either way. That is the same silence as before,
/// one layer down.
#[cfg_attr(feature = "fields", allow(dead_code))]
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
        "bevy_society: {} definitions read social fields (`Field` considerations or a `Gradient` \
         target) but the `fields` feature is off, so no grid exists and those activities will never be \
         chosen. Enable `fields`, or remove the definitions. See `docs/loop/needs-decision/` — {}",
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
    stocks: Query<&crate::Stock>,
    mut arrivals: Option<ResMut<ArrivalQueue>>,
    mut progress: Query<&mut TravelProgress>,
) {
    let now = clock.now_days;
    let dt_hours = f64::from(clock.dt_days) * 24.0;
    let radius = settings.interaction_radius;
    let plane = settings.plane;

    // Who has already arrived at each place, as a sorted list so a count is two binary searches. A
    // snapshot taken before anyone moves on this tick, so the answer does not depend on iteration order.
    let mut arrived_at: Vec<Entity> = people
        .iter()
        .filter_map(|(doing, _)| match (doing.target, doing.arrived_day) {
            (DoingTarget::Place(place), Some(_)) => Some(place),
            _ => None,
        })
        .collect();
    arrived_at.sort_unstable();

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
            // **A `Person` target moves, so its `Destination` is rewritten every tick**, whichever
            // locomotion the host chose. Otherwise a chase would follow where the target *was* when the
            // activity started, which under `Locomotion::Host` means the host is asked to walk to a
            // stale point and reports failure for a walk that was never impossible.
            //
            // This is before the arrival gate below on purpose: a person still far away is exactly the
            // one that needs its Destination updated, and the gate `continue`s.
            if let DoingTarget::Person(_) = doing.target {
                commands.entity(person).insert(Destination(target_pos));
            }
            if settings.tuning.stall_timeout_hours.is_some() {
                let distance = here.distance(target_pos);
                match progress.get_mut(person) {
                    Ok(mut seen) => {
                        if distance < seen.best - settings.tuning.stall_epsilon {
                            seen.best = distance;
                            seen.improved_day = now;
                        }
                    }
                    Err(_) => {
                        commands.entity(person).insert(TravelProgress {
                            best: distance,
                            improved_day: now,
                        });
                    }
                }
            }
            // **A crowded place is still an arrival.** Everyone sleeps at the same hour, so every resident
            // aims at one coordinate inside `interaction_radius`; ORCA holds centres one body diameter apart,
            // so the disc packs solid and the queue behind it times out. Measured: the goal already holds a
            // median of 6 bodies when a sleep fails, and a 1.5 m disc seats about 4.6.
            //
            // **The crowd is whoever is at the place**: the larger of the bodies inside `interaction_radius`
            // and the people the crate already knows arrived there. Counting only the disc sealed a crowded
            // place (issue #25): the people admitted at `allowed > radius` stand *outside* the disc, so the
            // count stopped growing and the next walker was refused on the ring the last one stood on.
            // Walkers that have not arrived never count, so nobody widens the gate for themselves.
            let crowd = match doing.target {
                DoingTarget::Place(place) => {
                    let arrived = arrived_at.partition_point(|&e| e <= place)
                        - arrived_at.partition_point(|&e| e < place);
                    let in_disc = perception
                        .pos
                        .iter()
                        .filter(|p| p.distance(target_pos) <= radius)
                        .count();
                    arrived.max(in_disc)
                }
                _ => 0,
            };
            let allowed = radius + settings.tuning.crowded_arrival_growth * (crowd as f32).sqrt();
            if here.distance(target_pos) > allowed {
                continue;
            }
            doing.arrived_day = Some(now);
            commands
                .entity(person)
                .remove::<(Destination, TravelProgress)>();
            // No `ActivityStarted` here: the decision already wrote it, paired with the
            // `ActivityEnded` that closes it. Arrival begins the *duration* and the `satisfies`
            // payout, which is what `arrived_day` is for.
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
            // **A meal feeds only as much as the shelf had.** `satisfies` is the crate's *satiation*
            // term and satiation follows the amount eaten, not the time spent at the table: Zandstra 2000
            // (DOI `10.18174/197257`) separates satiation, which "influences the size of meals", from
            // satiety, which "influences the frequency of meals". This used to pay in full from an empty
            // store, so the economy said "no bread changed hands" and the needs model said "fed". Measured
            // on the price pass's village: 4.80 loaves baked a game-day against 21.06 completed meals, so
            // ~77% of meals were conjured, and the village held zero bread for 24.0% of its ticks.
            //
            // **The factor is the authored fraction, not a penalty rate.** `taken / asked` is already in
            // the definitions file; a `Tuning` coefficient would invent a curve where the data gives one.
            // It is stable across a meal's ticks because `Stock` is debited at *completion* (the same rule
            // an interrupted task follows), so every tick of one meal reads the same store.
            //
            // **An activity that consumes nothing is untouched**: `factor` is `1.0` and `Stock` is never
            // read, which is every activity in the starter pack except `eat`.
            let factor = match stocks.get(person).ok() {
                Some(stock) if !spec.consumes.is_empty() => spec
                    .consumes
                    .iter()
                    .map(|&(good, asked)| {
                        if asked <= 0.0 {
                            1.0
                        } else {
                            (stock.get(good) / asked).min(1.0)
                        }
                    })
                    .fold(f32::INFINITY, f32::min),
                _ => 1.0,
            };
            // `satisfies` is a per-game-hour delta, so a tick is `dt_days · 24` hours of it.
            for &(need, delta) in &spec.satisfies {
                if let Some(level) = needs.0.get_mut(usize::from(need.0)) {
                    let gain = f64::from(delta) as f32 * dt_hours as f32 * factor;
                    *level = (*level + gain).clamp(0.0, 1.0);
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

/// Moves every person with a [`Destination`] toward it, unless something else is steering them.
///
/// This is the promise `Destination`'s own doc comment makes — *"games without the steering plugin
/// move persons themselves toward this point"* — kept by the plugin that creates the destination.
/// Without it a society has no way to arrive: measured, a person given `Destination(6.5, -5.0)` sat at
/// `(0, 0)` for 600 ticks, every `Place`- and `Home`-targeted activity timed out, and the needs those
/// activities satisfy never rose.
///
/// A person carrying a `SocialMover` is skipped: that is the steering plugin's, and two systems
/// writing one `Transform` would fight. The speed is [`Tuning::walk_speed`], metres per real second
/// at the fixed step, which is the same unit `SocialMover::speed` uses.
#[cfg(feature = "steering")]
type NotSteered = Without<crate::SocialMover>;
/// Without the `steering` feature there is no `SocialMover` component to exclude, so every traveller
/// is walked by this system. The alias keeps one function body for both builds.
#[cfg(not(feature = "steering"))]
type NotSteered = ();

fn walk_toward_destination(
    settings: Res<SocietySettings>,
    time: Res<Time<Fixed>>,
    mut walkers: Query<(&Destination, &mut Transform), NotSteered>,
) {
    // **No crate system writes a person's `Transform` under `Locomotion::Host`.** That is the whole
    // contract of the setting: the host walks them, on its own ground, around its own houses.
    if settings.locomotion != crate::Locomotion::Builtin {
        return;
    }
    let step = (settings.tuning.walk_speed * time.delta_secs()) as f64;
    if step <= 0.0 {
        return;
    }
    let plane = settings.plane;
    for (destination, mut transform) in &mut walkers {
        let mut here = plane.project(transform.translation);
        let toward = destination.0 - here;
        let distance = toward.length();
        if distance <= f32::EPSILON {
            continue;
        }
        let advanced = if f64::from(distance) <= step {
            distance
        } else {
            step as f32
        };
        here += toward / distance * advanced;
        // Keep the body's own height: `SocietyPlane::lift` puts the other axis at zero, which is right
        // for a ground marker and wrong for a capsule.
        transform.translation = match plane {
            crate::SocietyPlane::XZ => Vec3::new(here.x, transform.translation.y, here.y),
            crate::SocietyPlane::XY => Vec3::new(here.x, here.y, transform.translation.z),
        };
    }
}

/// The host reports a deed the society could not see, naming the people who saw it.
///
/// Sanctuary resolves events the society has no eyes for: a townsperson found afflicted at dawn, a
/// person accused by a crowd at the well. They must enter reputation and rumor as seen by **the people
/// the game names** — the family, the work crew, the crowd at the well — and then spread person to
/// person. A deed that enters through an accepted `Interact` gets its witnesses from perception; this is
/// the same path with the witness list supplied instead.
///
/// ```ignore
/// commands.write_message(ReportDeed {
///     actor,
///     patient: Some(found_afflicted),
///     deed: "affliction".to_owned(),
///     witnesses: vec![kin, neighbour, priest],
/// });
/// ```
///
/// An unknown deed is a `warn!` and nothing happens. A witness that is not a person is dropped with one
/// warning per report. The actor and the patient are never witnesses of their own deed. With no
/// witnesses left, there is no rumor and no reaction — a deed nobody saw is not a deed.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct ReportDeed {
    /// Who did it.
    pub actor: Entity,
    /// Who it was done to, when there was someone.
    pub patient: Option<Entity>,
    /// The deed's id, as the defs declare it.
    pub deed: String,
    /// The people who saw it, by name from the game. Order does not matter.
    pub witnesses: Vec<Entity>,
}

/// Host-reported deeds waiting for the next society tick.
///
/// **Drained by an ungated system in `Clock`**, the pattern `queue_interacts` and `EndRequests` use: a
/// society tick is many fixed steps apart and Bevy keeps a message for two buffer swaps, so a
/// `ReportDeed` written in `Update` would be gone before the tick that resolves it.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct DeedReportQueue(pub(crate) Vec<ReportDeed>);

/// Drains every host-written [`ReportDeed`] into [`DeedReportQueue`] on every fixed step.
fn queue_reported_deeds(
    mut reports: MessageReader<ReportDeed>,
    mut queue: ResMut<DeedReportQueue>,
) {
    for report in reports.read() {
        queue.0.push(report.clone());
    }
}

/// The host asks for a person's current activity to end, without dormancy.
///
/// The use a host has: a place that has closed for the night, a shift that ended, an activity the game
/// has made impossible. The activity ends with `ActivityEnded { completed: false }` in the same tick
/// and the person re-decides. A request about a person with no activity is ignored.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndActivity {
    /// The person whose activity should end.
    pub person: Entity,
}

/// A person died: written once, when [`Deceased`](crate::Deceased) is inserted.
///
/// The crate writes it itself, from an `On<Add<Deceased>>` observer, so a host gets it by inserting the
/// marker and nothing else. Use it to keep a roster, to spawn an undead under the dead person's name, or
/// to mark the town in mourning.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonDied {
    /// The person who died.
    pub person: Entity,
}

/// Deaths waiting for the next society tick.
///
/// **A resource rather than the message alone**, for the same reason `EndRequests` is one: an observer
/// runs whenever the world is mutated — possibly in `Update`, between two fixed steps — and the
/// bereavement pass must run inside the tick. The queue is drained once by [`announce_deaths`].
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct DeathQueue(pub(crate) Vec<Entity>);

/// Records a death the moment `Deceased` lands.
fn on_deceased_added(add: On<Add<crate::Deceased>>, mut queue: ResMut<DeathQueue>) {
    queue.0.push(add.entity);
}

/// Announces every death recorded since the last tick: the message, its trigger, and the bereavement.
///
/// **Runs before `apply_appraisals`**, so the grief a death produces counts in the tick it happened rather
/// than a tick later.
///
/// Bereavement is proportional to closeness, because that is what the literature says it is: Prigerson et
/// al. 2009 (DOI `10.1371/journal.pmed.1000121`) key prolonged grief disorder to the loss of a **close
/// relationship** rather than to any death, so a stranger's death is a fact and a friend's is a wound.
/// Intensity is `(0.5 · (1 + affinity)).clamp(0, 1)`: an enemy's death is not a grief, a tie at affinity 0
/// is half a grief, and a friend at 1.0 is a full one.
#[allow(clippy::too_many_arguments)]
fn announce_deaths(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<crate::Deceased>>,
    mut queue: ResMut<DeathQueue>,
    mut died: MessageWriter<PersonDied>,
    mut triggers: Option<ResMut<crate::rules::TriggerQueue>>,
    mut appraisals: MessageWriter<crate::Appraise>,
    mut memories: Query<&mut crate::Memories>,
    ties: Query<&Ties>,
) {
    let deaths = std::mem::take(&mut queue.0);
    if deaths.is_empty() {
        return;
    }
    let now = clock.now_days;
    // Sorted and deduped by `PersonId`, so two deaths in one tick give the same society however the
    // observer happened to fire, and a death recorded twice is announced once.
    let mut ordered: Vec<(u32, Entity)> = deaths
        .into_iter()
        .filter_map(|entity| {
            index
                .sorted
                .iter()
                .find(|(_, e)| *e == entity)
                .map(|(id, _)| (id.0, entity))
        })
        .collect();
    ordered.sort_unstable();
    ordered.dedup();

    for (_, person) in ordered {
        died.write(PersonDied { person });
        if let Some(triggers) = triggers.as_mut() {
            let actor = index
                .sorted
                .iter()
                .find(|(_, e)| *e == person)
                .map_or(-1, |(id, _)| i64::from(id.0));
            triggers
                .0
                .push(crate::rules::TriggerEvent::Deceased { actor });
        }

        // Everyone living who was close to the dead person grieves.
        for &(_, other) in &index.sorted {
            if other == person || deceased.get(other).is_ok() {
                continue;
            }
            let Ok(their_ties) = ties.get(other) else {
                continue;
            };
            let Some(tie) = their_ties.get(person) else {
                continue;
            };
            let Some(intensity) = crate::absence::closeness(tie, &settings.tuning) else {
                continue;
            };
            appraisals.write(crate::Appraise {
                person: other,
                kind: crate::EmotionKind::Distress,
                intensity,
                toward: Some(person),
            });
            if let Ok(mut theirs) = memories.get_mut(other) {
                theirs.record(
                    crate::Memory {
                        day: now as f32,
                        kind: crate::MemoryKind::Custom("bereaved".to_owned()),
                        other: Some(person),
                        valence: -intensity,
                        importance: 1.0,
                    },
                    now,
                    settings.memory_capacity,
                );
            }
        }
    }
}

/// The host reports that it cannot walk a person to its `Destination`.
///
/// Under [`Locomotion::Host`](crate::Locomotion::Host) the crate cannot see the host's paths, so it
/// cannot tell a slow walk from an impossible one. This is how the host says which. The activity ends
/// with `ActivityEnded { completed: false }` in the same tick, and the person re-decides.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TravelFailed {
    /// The person whose walk failed.
    pub person: Entity,
}

/// Requests to end an activity, drained from host-written messages so a request written between two
/// society ticks is never lost to a buffer swap.
///
/// **Ungated, in `Clock`**, because Bevy keeps a message for two buffer swaps and a society tick is
/// many fixed steps apart: a `TravelFailed` written in `Update` would be gone before the tick that
/// consumes it. The same pattern `queue_interacts` uses.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct EndRequests(pub(crate) Vec<Entity>);

/// Drains every host-written end-request message into [`EndRequests`] on every fixed step.
fn queue_end_requests(
    mut travel_failed: MessageReader<TravelFailed>,
    mut end_activity: MessageReader<EndActivity>,
    mut requests: ResMut<EndRequests>,
) {
    for failed in travel_failed.read() {
        requests.0.push(failed.person);
    }
    for asked in end_activity.read() {
        requests.0.push(asked.person);
    }
}

/// Ends every activity that is over (plan D7): its duration passed, its `Person` target left
/// perception, or it never arrived within `travel_timeout_hours`.
///
/// Gathers first and writes after, so the messages and the removals happen outside any borrow of the
/// people it read.
#[allow(clippy::too_many_arguments)]
/// The two per-person queries `end_activities` reads.
///
/// Bundled because Bevy's system-parameter limit is a hard cap and `end_activities` is close to it
/// once the dormancy query joins the `Doing` one; this crate already bundles this way in
/// `TriggerBookkeeping`.
#[derive(bevy::ecs::system::SystemParam)]
struct ActivityQueries<'w, 's> {
    doing: Query<'w, 's, &'static Doing>,
    progress: Query<'w, 's, &'static TravelProgress>,
    failed: Query<'w, 's, &'static FailedPlaces>,
    dormant: Query<'w, 's, (), With<crate::Dormant>>,
}

#[allow(clippy::too_many_arguments)]
fn end_activities(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    defs: Res<SocietyDefs>,
    index: Res<PersonIndex>,
    perception: Res<Perception>,
    mut commands: Commands,
    queries: ActivityQueries,
    mut ended: MessageWriter<ActivityEnded>,
    mut requests: ResMut<EndRequests>,
) {
    let ActivityQueries {
        doing: people,
        progress,
        dormant,
        failed,
    } = queries;
    let now = clock.now_days;
    // `None` means a walk never times out: only the host may declare it impossible, through
    // `TravelFailed`, because only the host can see whether a path exists.
    let timeout_hours = settings.tuning.travel_timeout_hours.map(f64::from);
    let stall_hours = settings.tuning.stall_timeout_hours.map(f64::from);
    let mut endings: Vec<(Entity, String, bool)> = Vec::new();

    // **Host end-requests come first**, so a `TravelFailed` written this tick ends its activity in
    // this tick rather than a tick later, and so a person the host cancelled is not also counted as a
    // timeout below. `BTreeSet` order by `PersonId`, so several requests in one tick give the same
    // society however the host's vec happened to be ordered.
    let cancelled: std::collections::BTreeSet<u32> = std::mem::take(&mut requests.0)
        .into_iter()
        .filter_map(|entity| {
            index
                .sorted
                .iter()
                .find(|(_, e)| *e == entity)
                .map(|(id, _)| id.0)
        })
        .collect();
    for id in &cancelled {
        let Some(&(person_id, person)) = index.sorted.iter().find(|(p, _)| p.0 == *id) else {
            continue;
        };
        let _ = person_id;
        let Ok(doing) = people.get(person) else {
            // A request about a person with no activity, or one that no longer exists, is ignored:
            // the host reports what it observes, and a despawn mid-walk is not an error.
            continue;
        };
        if let Some(spec) = defs.activities.get(usize::from(doing.activity.0)) {
            endings.push((person, spec.name.clone(), false));
        }
    }

    for &(_, person) in &index.sorted {
        let Ok(doing) = people.get(person) else {
            continue;
        };
        let Some(spec) = defs.activities.get(usize::from(doing.activity.0)) else {
            continue;
        };
        if endings.iter().any(|(ended, _, _)| *ended == person) {
            continue;
        }
        // **A dormant person is out of its daily life, and an activity that targets a dormant person
        // can no longer happen.** Both end now, `completed: false`: dormancy is a suspension, not a
        // completion, and whoever was walking toward them has to choose again.
        let suspended = dormant.get(person).is_ok()
            || match doing.target {
                DoingTarget::Person(target) => dormant.get(target).is_ok(),
                _ => false,
            };
        if suspended {
            endings.push((person, spec.name.clone(), false));
            continue;
        }
        let (finished, completed) = match doing.arrived_day {
            None => {
                let timed_out = timeout_hours
                    .is_some_and(|hours| (now - doing.started_day) * 24.0 > hours)
                    || stall_hours.is_some_and(|hours| {
                        progress
                            .get(person)
                            .is_ok_and(|seen| (now - seen.improved_day) * 24.0 > hours)
                    });
                // A `Person` target that no longer exists can never be reached, so the activity ends
                // now rather than at a timeout that may not exist at all.
                // **`contains`, not `binary_search`.** `Perception::persons` is filled in
                // `PersonIndex.sorted` order — by `PersonId` — while an `Entity`'s ordering is by index
                // and generation. `binary_search` therefore reports "gone" for people who are present,
                // and every activity targeting them is interrupted on the very tick it starts. Measured
                // on `village-evening`: **1 045 of 1 045** `compliment` interruptions took this branch
                // with the target standing in a list of 24; `binary_search` said gone and `contains` said
                // present on the same call.
                let gone = match doing.target {
                    DoingTarget::Person(target) => !perception.persons.contains(&target),
                    _ => false,
                };
                (timed_out || gone, false)
            }
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

    let memory_days = settings
        .tuning
        .failed_place_memory_hours
        .map(|hours| f64::from(hours) / 24.0);
    for (person, activity, completed) in endings {
        // **A walk to a place that ends before arrival is remembered** so the next decision does not
        // pick the same place again (`Tuning::failed_place_memory_hours`). A dormant person was
        // suspended, not defeated, and is not.
        if let (Some(days), false, Ok(doing)) = (memory_days, completed, people.get(person))
            && doing.arrived_day.is_none()
            && dormant.get(person).is_err()
            && let DoingTarget::Place(place) = doing.target
        {
            let mut memory = failed.get(person).cloned().unwrap_or_default();
            memory
                .0
                .retain(|(avoided, until)| *avoided != place && *until > now);
            memory.0.push((place, now + days));
            commands.entity(person).insert(memory);
        }
        ended.write(ActivityEnded {
            person,
            activity,
            completed,
        });
        commands
            .entity(person)
            .remove::<(Doing, Destination, TravelProgress)>();
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

/// One place as a decision sees it (plan D7): its tag set, how full it is, and **where it is**.
///
/// The position is what makes [`Signal::Distance`] meaningful for a `Place`. Without it the signal reported
/// `1.0` — the maximum, "as far as it can see" — for every place, so a definition that asked for `Distance`
/// on a `Place` target scored `0` and the activity could never be chosen. Measured: adding
/// `Distance Linear(-1, 1)` to the starter pack's `eat` took `food` from a mean of 0.402 to **0.034**, at
/// zero 93% of the time. The activity was not made travel-aware; it was disabled.
struct PlaceView {
    entity: Entity,
    tags: Vec<String>,
    capacity: u32,
    /// How many people are already aiming at this place, from the decisions already running.
    used: u32,
    position: Vec2,
}

/// Everything scoring one candidate needs, copied out of the world once per decision (plan D7).
///
/// Copied rather than borrowed because a decision ends by writing `Doing` on the very person it
/// scored, and a `&Ties` from the world cannot coexist with `&mut World`.
struct Decision {
    position: Vec2,
    needs: Vec<f32>,
    /// The host-written signal values at the tick start, indexed by `SignalId`.
    signals: Vec<f32>,
    personality: Personality,
    mood: [f32; 3],
    emotions: Emotions,
    ties: Vec<crate::relationships::Tie>,
    /// Neighbours with their plane positions, for `Distance`.
    near: Vec<(Entity, Vec2)>,
    /// Places with their plane positions, so `Distance` works for a `Place` target too. Without these the
    /// signal reported the maximum for every place, which silently disabled any activity targeting one.
    place_positions: Vec<(Entity, Vec2)>,
    /// The person's factions at the tick start, for the `Field` inputs.
    own_factions: Vec<crate::FactionId>,
    /// The person's close ties at the tick start, for `SocialProof`.
    close_ties: Vec<Entity>,
    /// What each **close tie** is doing, so `SocialProof` can count who else chose this activity.
    ///
    /// Only the close ties, not the whole population. This used to be `all_doing.clone()` inside the
    /// per-person loop: N copies of an N-length vector per tick, which is exactly N² and was most of the
    /// scale curve. Measured on the benchmark: 14.55 ms a tick at 1 000 people and **254.5 ms** at 5 000,
    /// with the per-pair cost flat, i.e. the only variable was the population squared. `SocialProof` only
    /// ever looks at `close_ties`, so filtering to them is the same answer at a `close_ties`-sized cost.
    close_doing: Vec<(Entity, u16)>,
    hour: f32,
    perception_radius: f32,
    /// The actor's own goods, so `Surplus` can ask what a unit is worth to it.
    stock: crate::Stock,
    /// Every neighbour's goods, so `Surplus` can ask the same of the other side of a trade. Keyed by entity
    /// and only for the people in `near`, which is the same set a `Person`-targeted activity can choose.
    near_stock: Vec<(Entity, crate::Stock)>,
    /// [`Tuning::trade_reserve`](crate::Tuning), copied so `signal_value` needs no settings handle.
    trade_reserve: f32,
    /// [`Tuning::trade_margin`](crate::Tuning), copied for the same reason.
    trade_margin: f32,
}

/// The components a decision reads for one person.
#[derive(QueryData)]
#[query_data(mutable)]
struct Decider<'w> {
    ties: &'w Ties,
    memberships: &'w crate::Memberships,
    needs: &'w Needs,
    signals: &'w crate::mind::Signals,
    personality: &'w Personality,
    mood: &'w Mood,
    emotions: &'w Emotions,
    home: Option<&'w crate::Home>,
    bound: Option<&'w crate::BoundPlaces>,
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
#[derive(Debug, Clone)]
struct Candidate {
    activity: u16,
    /// The targets that tied for the activity's best score. The draw happens over *activities*
    /// (Dill's "option" is an action), then one of these is picked uniformly — so an activity with
    /// many eligible targets is not made more likely by having them.
    targets: Vec<DoingTarget>,
    weight: f32,
    score: f32,
}

/// The four queries a decision reads. Bundled because Bevy's system-parameter limit is a hard cap and
/// `decide_activities` was already at it before the bound-place check joined.
#[derive(bevy::ecs::system::SystemParam)]
struct DecisionQueries<'w, 's> {
    people: Query<'w, 's, Decider<'static>>,
    places: Query<
        'w,
        's,
        (
            Entity,
            &'static crate::Place,
            &'static crate::PlaceTags,
            &'static Transform,
        ),
    >,
    doing: Query<'w, 's, &'static Doing>,
    failed: Query<'w, 's, &'static FailedPlaces>,
    harmed: Query<'w, 's, &'static RecentHarm>,
}

/// Whether an activity's interaction carries a deed that hurts its target (`harm < 0`).
fn activity_is_harmful(defs: &SocietyDefs, spec: &crate::defs::ActivitySpec) -> bool {
    spec.interaction
        .and_then(|id| defs.interactions.get(usize::from(id.0)))
        .and_then(|interaction| interaction.deed)
        .and_then(|deed| defs.deeds.get(usize::from(deed.0)))
        .is_some_and(|deed| deed.harm < 0.0)
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
    world: DecisionQueries,
    // Two markers in one query: `Has` reads both without a second one, which is what keeps this
    // system inside Bevy's system-parameter cap.
    marks: Query<(Has<crate::Autonomous>, Has<crate::Dormant>)>,
    mut started: MessageWriter<ActivityStarted>,
    engine: Option<Res<crate::script::ScriptEngine>>,
    cell: Res<crate::snapshot::SnapshotCell>,
    fields: Option<Res<FieldGrids>>,
    mut trace: Option<ResMut<crate::trace::DecisionTrace>>,
    // Bindings already warned about, by `(PersonId, name)`. A `Local` rather than a resource because
    // it is this system's own bookkeeping, and a `BTreeSet` so its iteration can never matter.
    mut warned_bindings: Local<std::collections::BTreeSet<(u32, String)>>,
    stocks: Query<&crate::Stock>,
) {
    let DecisionQueries {
        people,
        places,
        doing,
        failed,
        harmed,
    } = world;
    use bevy_emerge_rng::DetRng;

    // (person, scored options, the host signals the options were scored with) for the trace; only
    // filled when a host added the resource.
    type Scored = (String, f32, bool);
    let mut recorded: Vec<(Entity, Vec<Scored>, Vec<f32>)> = Vec::new();

    let now = clock.now_days;
    let tuning = &settings.tuning;
    // **Neighbours' stores are copied only when some activity reads them.** `near_stock` is a `Vec` of
    // `Stock`s per person per tick, which is cost a host with no `Surplus` consideration would pay for
    // nothing: the benchmark has none and runs at 5 000 people.
    let reads_stores = defs.activities.iter().any(|activity| {
        activity
            .considerations
            .iter()
            .any(|term| matches!(term.signal, Signal::Surplus(_)))
    });
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
        .map(|(entity, place, tags, transform)| PlaceView {
            entity,
            tags: tags.0.clone(),
            capacity: place.capacity,
            used: used
                .iter()
                .find(|(p, _)| *p == entity)
                .map_or(0, |(_, n)| *n),
            position: settings.plane.project(transform.translation),
        })
        .collect();
    place_views.sort_by_key(|p| p.entity.to_bits());

    // What everyone is doing, for `SocialProof`, as a lookup by `Entity::to_bits` so one person's
    // close ties can be read without scanning the population. `all_doing` used to be cloned whole into
    // every `Decision`; the filtered list is now built from this map at a `close_ties`-sized cost.
    let all_doing: Vec<(Entity, u16)> = index
        .sorted
        .iter()
        .filter_map(|&(_, entity)| doing.get(entity).ok().map(|d| (entity, d.activity.0)))
        .collect();

    // The engine and the tick's snapshot are the same for every consideration in this pass, so they
    // are read once and shared by reference.
    let snapshot = cell.0.clone();
    // Plane positions of everything a destination can name, gathered once: `Commands` cannot read
    // the world while a plan is being written. The places query carries their `Transform`, and a
    // person's position comes from `Perception`, so no separate transform query is needed — which is
    // also what keeps this system inside Bevy's sixteen-parameter limit.
    let mut positions: Vec<(Entity, Vec2)> = place_views
        .iter()
        .map(|view| (view.entity, view.position))
        .chain(
            index
                .sorted
                .iter()
                .enumerate()
                .filter_map(|(slot, (_, entity))| {
                    perception.pos.get(slot).map(|at| (*entity, *at))
                }),
        )
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
        // **A dormant person is out of its daily life**: it keeps its ties, memories and knowledge,
        // but the crate stops choosing for it until the marker is removed.
        let (autonomous, dormant) = marks.get(person).unwrap_or((false, false));
        if !autonomous || dormant || doing.get(person).is_ok() {
            continue;
        }
        let Ok(view) = people.get(person) else {
            continue;
        };
        let (ties, memberships, needs, signals, personality, mood, emotions, home, bound) = (
            view.ties,
            view.memberships,
            view.needs,
            view.signals,
            view.personality,
            view.mood,
            view.emotions,
            view.home,
            view.bound,
        );
        let position = perception.pos.get(slot).copied().unwrap_or(Vec2::ZERO);
        // `near` and `near_ids` are built from one pass over `perception.near`, so index `k` of
        // each describes the same neighbour. The id comes from `PersonIndex.sorted`, whose order is
        // the same index order `perception.near` uses.
        let mut near: Vec<(Entity, Vec2)> = Vec::new();
        let mut near_ids: Vec<u32> = Vec::new();
        if let Some(list) = perception.near.get(slot) {
            for (j, _) in list {
                let j = *j as usize;
                let (Some(other), Some(pos)) = (
                    perception.persons.get(j).copied(),
                    perception.pos.get(j).copied(),
                ) else {
                    continue;
                };
                if other == person {
                    continue;
                }
                near.push((other, pos));
                near_ids.push(index.sorted.get(j).map_or(u32::MAX, |(id, _)| id.0));
            }
        }

        let place_positions: Vec<(Entity, Vec2)> = place_views
            .iter()
            .map(|view| (view.entity, view.position))
            .collect();
        let close = ties.close_targets(now as f32, tuning, settings.close_ties);
        let own_factions = memberships.0.clone();
        let decision = Decision {
            own_factions: own_factions.clone(),
            place_positions: place_positions.clone(),
            position,
            needs: needs.0.clone(),
            signals: signals.0.clone(),
            personality: *personality,
            mood: mood.as_array(),
            emotions: emotions.clone(),
            ties: ties.0.clone(),
            near: near.clone(),
            close_ties: close.clone(),
            close_doing: all_doing
                .iter()
                .filter(|(entity, _)| close.contains(entity))
                .copied()
                .collect(),
            hour,
            perception_radius: settings.perception_radius,
            trade_reserve: tuning.trade_reserve,
            trade_margin: tuning.trade_margin,
            // Goods, for `Surplus`. Only the neighbours in `near`, which is the same set a
            // `Person`-targeted activity can actually choose from.
            stock: stocks.get(person).cloned().unwrap_or_default(),
            near_stock: if reads_stores {
                near.iter()
                    .filter_map(|(entity, _)| {
                        stocks.get(*entity).ok().cloned().map(|s| (*entity, s))
                    })
                    .collect()
            } else {
                Vec::new()
            },
        };
        // The actor is fixed for this person; the target changes with every option, so it is bound
        // per candidate in `score_option` rather than here.
        // `slot` indexes `PersonIndex.sorted`, so the id is read, not searched for (the search was
        // O(people) per person).
        let actor_id = index.sorted.get(slot).map_or(-1, |(id, _)| i64::from(id.0));
        // A neighbour's id by entity, as a sorted lookup: `near.iter().position` per target was
        // O(neighbours) per target, so O(neighbours squared) per activity per person.
        let mut near_lookup: Vec<(u64, u32)> = near
            .iter()
            .zip(&near_ids)
            .map(|((entity, _), id)| (entity.to_bits(), *id))
            .collect();
        near_lookup.sort_by_key(|(bits, _)| *bits);
        // The scripts read the tick-start snapshot and the compiled definitions; both are the same
        // for every option, so they are borrowed once and only the target is rebound per candidate.
        let script_ready = match (&engine, &snapshot) {
            (Some(engine), Some(snapshot)) => Some((snapshot, engine)),
            _ => None,
        };
        // **A binding to something that is not a place is a wiring mistake, and it is warned about
        // once per person and name.** An unbound person is not warned about at all: a host that binds
        // some people and not others (a night watch has no work place) would otherwise be told once per
        // person for nothing.
        if let Some(bindings) = bound {
            let actor_id = index
                .sorted
                .iter()
                .find(|(_, e)| *e == person)
                .map(|(id, _)| id.0)
                .unwrap_or(u32::MAX);
            for (name, entity) in &bindings.0 {
                if place_views.iter().any(|place| place.entity == *entity) {
                    continue;
                }
                if !warned_bindings.insert((actor_id, name.clone())) {
                    continue;
                }
                warn!(
                    "bevy_society: person {actor_id} has \"{name}\" bound to {entity:?}, which is not a \
                     place; the activity has no target"
                );
            }
        }

        let mut best: Option<(u8, Vec<Candidate>)> = None;
        // Every option that scored, for the trace only. `best` keeps the winning rank tier, so
        // without this a host would never see the lower-rank activities a choice ruled out.
        let mut all_scored: Vec<(String, f32)> = Vec::new();
        for (i, spec) in defs.activities.iter().enumerate() {
            // Index 0 is the reserved idle activity: it is the fallback, never a candidate.
            if i == 0 {
                continue;
            }
            let mut targets = targets_for(
                &spec.target,
                position,
                &near,
                &place_views,
                home.map(|h| h.0),
                bound,
                &own_factions,
                fields.as_deref(),
                &mut rng.0,
            );
            // **A place this person just failed to reach is not a candidate** until it is forgiven
            // (`Tuning::failed_place_memory_hours`). Filtered after `targets_for` so the RNG draws it
            // makes are the same whether or not anything is remembered.
            if let Ok(memory) = failed.get(person) {
                targets.retain(|target| match target {
                    DoingTarget::Place(place) => !memory
                        .0
                        .iter()
                        .any(|(avoided, until)| avoided == place && *until > now),
                    _ => true,
                });
            }
            // **A person this one has just harmed is not a target of another harm** until the refractory
            // period ends (`Tuning::retaliation_refractory_hours`): the answer to a grievance is one blow,
            // not a standing exchange of them. Filtered after `targets_for`, like the failed places.
            if tuning.retaliation_refractory_hours.is_some()
                && activity_is_harmful(&defs, spec)
                && let Ok(memory) = harmed.get(person)
            {
                targets.retain(|target| match target {
                    DoingTarget::Person(other) => !memory
                        .0
                        .iter()
                        .any(|(spared, until)| spared == other && *until > now),
                    _ => true,
                });
            }
            // **One candidate per activity, not per target.** Dill's weight-based draw is over
            // *options*, and an option is an action; an action with eight eligible targets is not
            // eight times as desirable as one with a single target. Scoring every target and
            // keeping the best (with its ties) gives the activity one score, and the target is
            // picked from the ties afterwards. Before this, `gossip` with eight neighbours held
            // 8/9 of the selection mass and `relax` starved: measured, a Person activity went from
            // 0.00 of all starts at one neighbour to 0.97 at twelve.
            let mut best_for_activity: Option<(f32, Vec<DoingTarget>)> = None;
            for target in targets {
                // `near_lookup` answers by binary search what a scan of `near` answered before.
                let target_id = match target {
                    DoingTarget::Person(other) => near_lookup
                        .binary_search_by_key(&other.to_bits(), |(bits, _)| *bits)
                        .map_or(-1, |at| i64::from(near_lookup[at].1)),
                    _ => -1,
                };
                let script_ref = script_ready.map(|(snapshot, engine)| ScriptContext {
                    defs: &defs,
                    snapshot,
                    engine,
                    actor: actor_id,
                    target: target_id,
                });
                let mut score = score_option(
                    &spec.considerations,
                    &decision,
                    target,
                    i as u16,
                    script_ref.as_ref(),
                    fields.as_deref(),
                    &defs.skills,
                );
                // **What a place with no room costs.** The price belongs to the target, not to a consideration, so
                // every activity pays it — including ones a host wrote — without a definitions-file change.
                score *= crowding_price(target, &place_views, tuning.crowding_penalty);
                if score <= 0.0 {
                    continue;
                }
                match &mut best_for_activity {
                    Some((best, ties)) if score > *best + f32::EPSILON => {
                        *best = score;
                        *ties = vec![target];
                    }
                    Some((best, ties)) if (score - *best).abs() <= f32::EPSILON => {
                        ties.push(target);
                    }
                    Some(_) => {}
                    None => best_for_activity = Some((score, vec![target])),
                }
            }
            let Some((score, targets)) = best_for_activity else {
                continue;
            };
            let candidate = Candidate {
                activity: i as u16,
                targets,
                weight: spec.weight,
                score,
            };
            if trace.is_some() {
                all_scored.push((
                    defs.activity_name(ActivityId(candidate.activity))
                        .to_owned(),
                    candidate.score,
                ));
            }
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

        if trace.is_some() {
            let mut scored: Vec<(String, f32, bool)> = all_scored
                .into_iter()
                .map(|(name, score)| (name, score, false))
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            recorded.push((person, scored, decision.signals.clone()));
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
            let mut chosen = (*viable.last()?).clone();
            for candidate in &viable {
                roll -= candidate.weight * candidate.score;
                if roll <= 0.0 {
                    chosen = (*candidate).clone();
                    break;
                }
            }
            // The activity is chosen; now which of its tied targets.
            //
            // **A `Place` is a fixed destination, so the nearest one wins.** Every place carrying the tag
            // scores the same — a `Place` target has no `Distance` term — so they tie, and the tie used to
            // be a uniform pull: measured, a person with a tavern 1 m away and another 9 m away chose the
            // far one **2 979 times in 6 000 decisions**. That coin flip is most of why `village-evening`
            // spends **62.5% of all person-ticks travelling**.
            //
            // A `Person` keeps the uniform pull: chatting with whichever neighbour comes up is variety,
            // not a defect. Ties on distance fall back to the seeded draw, so the result stays
            // deterministic.
            let target = match chosen.targets.first() {
                Some(DoingTarget::Place(_) | DoingTarget::Point(_)) => chosen
                    .targets
                    .iter()
                    .min_by(|a, b| {
                        let da = place_distance(**a, position, &place_positions);
                        let db = place_distance(**b, position, &place_positions);
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
                    .unwrap_or(chosen.targets[0]),
                _ => {
                    let index = rng.0.below(chosen.targets.len().max(1));
                    *chosen.targets.get(index)?
                }
            };
            Some((chosen.activity, target))
        });

        if trace.is_some()
            && let Some((_, scored, _)) = recorded.iter_mut().find(|(p, _, _)| *p == person)
        {
            let chosen = picked.map(|(activity, _)| activity);
            if scored.is_empty() {
                scored.push((
                    defs.activity_name(crate::defs::IDLE_ACTIVITY).to_owned(),
                    1.0,
                    true,
                ));
            } else {
                for (name, _, is_chosen) in scored.iter_mut() {
                    *is_chosen = defs.activity_id(name).map(|id| id.0) == chosen;
                }
            }
        }

        match picked {
            Some((activity, target)) => {
                let activity = ActivityId(activity);
                let name = defs.activity_name(activity).to_owned();
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

    if let Some(trace) = trace.as_mut() {
        for (person, scored, signals) in recorded {
            trace.push(crate::trace::Decision {
                tick: clock.tick,
                person,
                // Resolved by `Decision::describe` when read: `decide_activities` is already at
                // Bevy's 16-parameter limit, and a `Name` query would be the seventeenth.
                name: None,
                signals,
                options: scored
                    .into_iter()
                    .map(|(activity, score, chosen)| crate::trace::ScoredOption {
                        activity,
                        score,
                        chosen,
                    })
                    .collect(),
            });
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
        entity.remove::<TravelProgress>();
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
        // Written at decision, not on arrival: `Doing::started_day` is the day the activity
        // started, and it is stamped just above. Emitting only on arrival meant a person who
        // never arrived — its target out of reach, `travel_timeout_hours` elapsed — produced an
        // `ActivityEnded` for a start no host was ever told about. Measured: 0 starts against
        // 149 ends for one person whose place was 50 m away with no steering plugin.
        started.write(ActivityStarted {
            person,
            activity: name,
            target: person_target,
        });
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

/// What a place-targeted option keeps after paying for a place that has no room left.
///
/// `1 - penalty` when the place is **full**, `1.0` when it has a seat to spare, and `1.0` for any other target.
/// A place that seats nobody is never offered, so it is not priced here. `penalty == 0` makes this exactly `1.0`
/// everywhere, which is how the un-priced ranking is restored.
///
/// A **step**, not a ramp: the queue exists only when there is no seat, so that is where the cost is charged. A
/// proportional price was measured and rejected because it diverts people from a place that is merely nearly full,
/// where nobody queues, and took `work` from 10.3% to 6.6% at 48 people. See
/// [`Tuning::crowding_penalty`](crate::Tuning).
fn crowding_price(target: DoingTarget, places: &[PlaceView], penalty: f32) -> f32 {
    if penalty <= 0.0 {
        return 1.0;
    }
    let DoingTarget::Place(entity) = target else {
        return 1.0;
    };
    let Some(place) = places.iter().find(|place| place.entity == entity) else {
        return 1.0;
    };
    if place.used < place.capacity {
        return 1.0;
    }
    (1.0 - penalty).clamp(0.0, 1.0)
}

/// The targets an activity offers this decision (plan D7), with closed places excluded.
///
/// **Closed, not full.** A place with no free seat is still offered: `capacity` says how many people it is
/// *sized* for, and a host that wants a crowd to be unattractive expresses that as a consideration. Deleting the
/// target was measured to cost far more than it bought — see the note on `Place(tag)` below.
///
/// `Wander` draws its point here, from the two `unit()` draws the SPEC names: radius · √u₁ at angle
/// 2πu₂, which is uniform over the disc rather than bunched at its centre.
#[allow(clippy::too_many_arguments)]
fn targets_for(
    target: &ActivityTarget,
    position: Vec2,
    near: &[(Entity, Vec2)],
    places: &[PlaceView],
    home: Option<Entity>,
    bound: Option<&crate::BoundPlaces>,
    own_factions: &[crate::FactionId],
    fields: Option<&FieldGrids>,
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
        // **A full place is still a place.** Filtering on `used < capacity` deleted the option entirely, so a
        // full workshop made `work` something that did not exist — and the consequence was not a lost activity
        // but a different village: measured, a 30-seat village given 48 people took `work` from 10.3% to 3.6% of
        // a life and rival density from 7.8% to 20.1% of directed pairs, because the same hostile events landed
        // on a contact graph made dense by unemployment. Pentheny, *Game AI Pro 2* ch. 17 (corpus
        // `gameaipro2-ch17-advanced-techniques-robust-efficient-crowds`) names the failure — "agents fight the
        // rest of the crowd to follow their ideal path as closely as possible, seemingly ignoring all other
        // possible routes" — and congestion maps fix it by making a crowd *cost*; with `Place(tag)` already
        // picking the nearest place a host can express that cost itself, and the crate's job is not to delete
        // the option. Only a place that seats **nobody** is impossible: `capacity: 0` is a closed door, not a
        // queue.
        ActivityTarget::Place(tag) => places
            .iter()
            .filter(|place| place.capacity > 0 && place.tags.iter().any(|t| t == tag))
            .map(|place| DoingTarget::Place(place.entity))
            .collect(),
        // The place this person bound under the name, when they have one. No binding means no candidate at
        // all: the activity simply does not apply to them. Room is not required — see `Place(tag)` — but a
        // place that seats nobody is closed.
        ActivityTarget::Bound(name) => bound
            .and_then(|bindings| bindings.get(name))
            .filter(|entity| {
                places
                    .iter()
                    .any(|place| place.entity == *entity && place.capacity > 0)
            })
            .map_or_else(Vec::new, |entity| vec![DoingTarget::Place(entity)]),
        ActivityTarget::Person => near.iter().map(|(e, _)| DoingTarget::Person(*e)).collect(),
        // A gradient target walks toward the slope of the named field. Without the `fields` feature, or
        // before the grid exists on the first tick, there is no slope and the option has no target —
        // which is the honest answer, and what the startup warning is about when the feature is off.
        ActivityTarget::Gradient(input, toward) => fields
            .and_then(|grids| grids.gradient_target(*input, own_factions, position, *toward))
            .map_or_else(Vec::new, |point| vec![DoingTarget::Point(point)]),
    }
}

/// The social-field grids, when the `fields` feature is on.
///
/// With the feature off this is an empty resource, so the decision keeps one shape and `Signal::Field`
/// still has an arm to read — a zero, because no grid exists. That is what the startup warning tells
/// the host about, rather than a compile error for a definition file the crate already accepts.
#[cfg(feature = "fields")]
type FieldGrids = crate::SocialFields;
/// An empty stand-in for [`crate::SocialFields`] when the `fields` feature is off.
///
/// It answers the two calls the decision makes, with the only honest values: no cell, no slope. A
/// definition that reads a field therefore scores `0` and is never chosen, which is exactly what the
/// startup warning says will happen — rather than a compile error for a definition file the crate
/// accepts in either configuration.
#[cfg(not(feature = "fields"))]
#[derive(bevy::prelude::Resource, Default)]
pub struct NoFields;

#[cfg(not(feature = "fields"))]
impl NoFields {
    fn world_to_cell(&self, _at: Vec2) -> bevy::math::IVec2 {
        bevy::math::IVec2::ZERO
    }

    fn field_value(
        &self,
        _input: crate::FieldInput,
        _own: &[crate::FactionId],
        _cell: bevy::math::IVec2,
    ) -> f32 {
        0.0
    }

    fn gradient_target(
        &self,
        _input: crate::FieldInput,
        _own: &[crate::FactionId],
        _pos: Vec2,
        _toward: bool,
    ) -> Option<Vec2> {
        None
    }
}
#[cfg(not(feature = "fields"))]
type FieldGrids = NoFields;

/// How far a target is from `from`, on the society plane. `Person` and `Itself` report zero, because a
/// person target's position is not what a `Place` tie is choosing between.
fn place_distance(target: DoingTarget, from: Vec2, places: &[(Entity, Vec2)]) -> f32 {
    let of = |entity: Entity| {
        places
            .iter()
            .find(|(place, _)| *place == entity)
            .map(|(_, at)| *at)
    };
    match target {
        DoingTarget::Place(place) => of(place).map_or(f32::MAX, |at| from.distance(at)),
        DoingTarget::Point(point) => from.distance(point),
        DoingTarget::Person(_) | DoingTarget::Itself => 0.0,
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
    fields: Option<&FieldGrids>,
    skills: &[crate::defs::SkillGroup],
) -> f32 {
    let n = considerations.len();
    if n == 0 {
        return 1.0;
    }
    // **Read once per option, not once per consideration.** The target's tie and its position are
    // linear searches over the tie list and the neighbour list; every term of an option shares them.
    let tie = match target {
        DoingTarget::Person(other) => decision.ties.iter().find(|t| t.target == other).copied(),
        _ => None,
    };
    let there = if considerations
        .iter()
        .any(|term| matches!(term.signal, Signal::Distance))
    {
        match target {
            DoingTarget::Person(other) => decision
                .near
                .iter()
                .find(|(e, _)| *e == other)
                .map(|(_, p)| *p),
            DoingTarget::Place(place) => decision
                .place_positions
                .iter()
                .find(|(e, _)| *e == place)
                .map(|(_, p)| *p),
            // A `Point` is chosen by `Wander` and has no entry in the place list; `Itself` is the
            // actor's own position, which is not what a distance consideration is asking about.
            DoingTarget::Point(_) | DoingTarget::Itself => None,
        }
    } else {
        None
    };
    let mut product = 1.0_f32;
    for term in considerations {
        let s = term.curve.eval(signal_value(
            term.signal,
            decision,
            target,
            tie,
            there,
            activity,
            scripts,
            fields,
            skills,
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
#[allow(clippy::too_many_arguments)]
fn signal_value(
    signal: Signal,
    decision: &Decision,
    target: DoingTarget,
    tie: Option<crate::relationships::Tie>,
    there: Option<Vec2>,
    activity: u16,
    scripts: Option<&ScriptContext<'_>>,
    fields: Option<&FieldGrids>,
    skills: &[crate::defs::SkillGroup],
) -> f32 {
    let value = match signal {
        Signal::Need(id) => decision
            .needs
            .get(usize::from(id.0))
            .copied()
            .unwrap_or(0.0),
        Signal::Host(id) => decision
            .signals
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
            DoingTarget::Person(_) => tie.map_or(0.0, |t| t.familiarity_peak),
            _ => 0.0,
        },
        Signal::Distance => there.map_or(1.0, |p| {
            (decision.position.distance(p) / decision.perception_radius).min(1.0)
        }),
        // **The joint surplus of a trade**, from the two stores. Not a rate: `value_of` is `1 / (1 + held)`,
        // so the gap is near zero when two people hold the same amount and largest when one has nothing.
        // Hall & Milgrom's precondition for a bargain to have anything in it. It is zero unless the actor
        // holds **more than** it keeps back (`Tuning::trade_reserve`, so a giver is never left with less
        // than the reserve when it gives a unit) and the gap is worth the walk (`Tuning::trade_margin`); a target who is not in the neighbour list, or who holds no goods at all,
        // is not a counterparty.
        Signal::Surplus(id) => {
            let actor_holds = decision.stock.get(id);
            let target_holds = match target {
                DoingTarget::Person(other) => decision
                    .near_stock
                    .iter()
                    .find(|(e, _)| *e == other)
                    .map(|(_, stock)| stock.get(id)),
                _ => None,
            };
            target_holds.map_or(0.0, |theirs| {
                let gap = crate::economy::value_of(theirs) - crate::economy::value_of(actor_holds);
                if actor_holds <= decision.trade_reserve || gap < decision.trade_margin {
                    0.0
                } else {
                    gap.min(1.0)
                }
            })
        }
        // **A skill is read, not stored.** The level is the trait its group names, so the same personality
        // is the same competence everywhere and nothing has to be saved. An undefined skill cannot reach
        // here: `compile_term` resolves the name against the defs and refuses a package that names one.
        Signal::Skill(id) => skills.get(usize::from(id.0)).map_or(0.0, |group| {
            crate::skill_level(&decision.personality, *group)
        }),
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
        // A field input reads the stigmergy grid at the actor's own cell. Without the `fields` feature
        // there is no grid and the honest answer is 0 — but with it the value is real, and returning a
        // constant here made every `Field` definition dead while the definitions file accepted it.
        Signal::Field(input) => match fields {
            Some(grids) => {
                let cell = grids.world_to_cell(decision.position);
                grids.field_value(input, &decision.own_factions, cell)
            }
            None => 0.0,
        },
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
    // **A dormant person takes part in nothing**, and that includes an exchange the host asked for
    // directly with `Interact`: dormancy is the host saying this person is not living their daily life
    // right now, and honouring it only for crate-chosen exchanges would make the marker a suggestion.
    if world.get::<crate::Dormant>(actor).is_some() || world.get::<crate::Dormant>(target).is_some()
    {
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

    // A harmful attempt starts the actor's refractory period against this target, whatever the answer:
    // a refused insult is still an attempt, and it must not feed the next (`retaliation_refractory_hours`).
    if let Some(hours) = settings.tuning.retaliation_refractory_hours
        && spec
            .deed
            .and_then(|d| defs.deeds.get(usize::from(d.0)))
            .is_some_and(|deed| deed.harm < 0.0)
    {
        let mut harmed = world.get::<RecentHarm>(actor).cloned().unwrap_or_default();
        harmed
            .0
            .retain(|(other, until)| *other != target && *until > now);
        harmed.0.push((target, now + f64::from(hours) / 24.0));
        world.entity_mut(actor).insert(harmed);
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
            // The witnesses of an observed exchange are whoever was within perception of the actor,
            // excluding the two parties. That is `Perception`'s job; a host-reported deed brings its own
            // list, which is why `witness_deed` takes one rather than computing it.
            let seen_by: Vec<Entity> = index
                .iter()
                .position(|(_, e)| *e == actor)
                .and_then(|slot| perception.near.get(slot))
                .map(|list| {
                    list.iter()
                        .filter_map(|(j, _)| perception.persons.get(*j as usize).copied())
                        .filter(|w| *w != actor && *w != target)
                        .collect()
                })
                .unwrap_or_default();
            witness_deed(world, deed, actor, target, defs, settings, now, &seen_by);
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
            tie.add_affinity(
                -settings.tuning.rejection_affinity_penalty,
                &settings.tuning,
            );
        }
        // **The definition's own `on_reject` emotion wins.** A pack that says what a refusal feels
        // like is not second-guessed; the crate's number is the fallback for a pack that says nothing,
        // which the starter pack did. Emitting both stacked `0.3` on top of the definition's `0.1` and
        // measured *worse* mood than either alone — two paths for one event.
        if effects.actor_emotion.is_none() && settings.tuning.rejection_distress > 0.0 {
            world.write_message(crate::mind::Appraise {
                person: actor,
                kind: crate::mind::EmotionKind::Distress,
                intensity: settings.tuning.rejection_distress,
                toward: Some(target),
            });
        }
    }

    // (6) Memories, both sides.
    //
    // The magnitude of a **deed's** memory is `0.3 + its harm`, but the deed fires only on accept — see
    // step (4). Sizing every outcome by `harm` therefore records an injury that never happened: a
    // *refused* insult wrote `-0.7` into the snubber, about the person who refused.
    //
    // That closed a loop. `snub`'s only consideration is `ctx.grudge(actor, target)`, so a refused snub
    // deepened the very grudge that motivated the next one, and because `snub` satisfies no need nothing
    // pushed back. Measured on `rumor-crosses-a-cluster` (12 people, seed 11): `snub` was **170 of 338**
    // activity starts, ten of them in a row, and the narrative read `snub, compliment, snub, compliment`
    // through the evening. A single refusal scored the next snub at 0.7 through `Linear(1.0, 0.0)`; two
    // clamped the grudge to 1.0.
    //
    // A refusal is not an injury. It costs the interaction's own weight — the `0.3` every resolved
    // exchange carries — and the deed's `harm` sizes a memory only when the deed actually fired.
    let harm = if accepted {
        spec.deed
            .and_then(|d| defs.deeds.get(usize::from(d.0)))
            .map_or(0.0, |spec| spec.harm)
    } else {
        0.0
    };
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
    // **Goods change hands, and only what is there.** A giver who has less than the amount gives what it
    // has; the transfer is a barter, so both sides move in the same call and a shortfall on one side is not
    // a debt. `Stock::take` reports what actually moved and `Stock::add` what actually landed, so the pair
    // cannot create goods.
    if !effects.actor_gives.is_empty() || !effects.target_gives.is_empty() {
        let ceiling = world
            .get_resource::<crate::SocietySettings>()
            .map_or_else(Default::default, |settings| settings.tuning.stock_ceiling);
        for (from, to, list) in [
            (actor, target, &effects.actor_gives),
            (target, actor, &effects.target_gives),
        ] {
            for &(good, amount) in list {
                let taken = world
                    .get_mut::<crate::Stock>(from)
                    .map_or(0.0, |mut stock| stock.take(good, amount));
                if taken > 0.0
                    && let Some(mut theirs) = world.get_mut::<crate::Stock>(to)
                {
                    theirs.add(good, taken, ceiling);
                }
            }
        }
    }
    // Read once: the affinity clamp is `Tuning::affinity_ceiling`, so every writer needs it.
    let tuning = world
        .get_resource::<SocietySettings>()
        .map_or_else(Default::default, |settings| settings.tuning.clone());
    // Tie deltas, clamped exactly as `fx.adjust` clamps them.
    for (owner, other, delta) in [
        (actor, target, effects.actor_to_target),
        (target, actor, effects.target_to_actor),
    ] {
        let Some(mut ties) = world.get_mut::<Ties>(owner) else {
            continue;
        };
        let tie = ties.entry(other);
        tie.add_affinity(delta.affinity, &tuning);
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
            // **This is the gossip path, so the teller is known.** What a listener makes of a rumor
            // depends on who told them, and a rumor learned by seeing the deed is not someone's word.
            from: Some(teller),
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
fn witness_deed(
    world: &mut World,
    deed: crate::defs::DeedId,
    actor: Entity,
    patient: Entity,
    defs: &SocietyDefs,
    settings: &SocietySettings,
    now: f64,
    witnesses: &[Entity],
) {
    let Some(deed_spec) = defs.deeds.get(usize::from(deed.0)).cloned() else {
        return;
    };

    // **A deed deposits its harm into the commotion field, at the actor's cell.** The channel is what
    // `FieldInput::Commotion` reads and what a `Gradient(Commotion, true)` activity climbs, and until
    // this line nothing wrote it: the field was always zero, so a `gawk` activity had no direction and
    // every `Commotion` consideration scored 0. `at` is a plane position, which is what the message
    // takes, so a host writing one needs no grid geometry and neither does this.
    #[cfg(feature = "fields")]
    if deed_spec.harm != 0.0
        && let Some(transform) = world.get::<GlobalTransform>(actor)
    {
        world.write_message(crate::fields::Commotion {
            at: settings.plane.project(transform.translation()),
            amount: deed_spec.harm.abs(),
        });
    }

    let mut approvals: Vec<(Entity, f32)> = Vec::new();
    // Everyone who saw it, for `Tuning::witness_dilution`; the crowd is the same for every witness.
    let watcher_count = witnesses.len();
    for &watcher in witnesses {
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
            watcher_count,
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
        crate::reputation::patient_reaction(
            &mut ties,
            &mut emotions,
            deed_spec.harm,
            gain,
            actor,
            &settings.tuning,
        );
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
            world
                .get_resource::<crate::core::SocietySettings>()
                .map_or(64, |s| s.knowledge_capacity),
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
                    // A witnessed deed: nobody told them, they saw it.
                    from: None,
                });
            }
        }
        world.insert_resource(book);
    }
}

/// Resolves every host-reported deed (issue #8), after the exchanges have run.
///
/// Exclusive because [`witness_deed`] writes to whoever it likes, the same as the exchange path.
///
/// The named witnesses are the point: perception is not consulted at all, so a host can report what its
/// own rules saw. Ordering is fixed for determinism — the witnesses are deduped and ordered by
/// [`PersonId`](crate::PersonId) before the deed is reported, so two runs with the same witness set give
/// the same society however the host's `Vec` happened to be ordered.
fn resolve_reported_deeds(world: &mut World) {
    let (now, defs, reports, index) = {
        let now = world.resource::<SocietyClock>().now_days;
        let defs = world.resource::<SocietyDefs>().clone();
        let index = world.resource::<PersonIndex>().sorted.clone();
        let mut queue = world
            .remove_resource::<DeedReportQueue>()
            .unwrap_or_default();
        let reports = std::mem::take(&mut queue.0);
        world.insert_resource(queue);
        (now, defs, reports, index)
    };
    if reports.is_empty() {
        return;
    }
    let settings = world.resource::<SocietySettings>().clone();

    for report in reports {
        let Some(deed) = defs.deed_id(&report.deed) else {
            warn!(
                "bevy_society: ReportDeed names unknown deed '{}'; ignored",
                report.deed
            );
            continue;
        };
        // **The named witnesses, deduped and in `PersonId` order.** The actor and the patient are never
        // witnesses of their own deed, and a dormant person still counts if the host named them: the
        // host saw something the crate cannot, and `Dormant` is about daily life, not about eyes.
        let mut witnesses: Vec<Entity> = Vec::new();
        let mut not_people = 0usize;
        for &entity in &report.witnesses {
            if entity == report.actor || Some(entity) == report.patient {
                continue;
            }
            if !index.iter().any(|(_, e)| *e == entity) {
                not_people += 1;
                continue;
            }
            if !witnesses.contains(&entity) {
                witnesses.push(entity);
            }
        }
        if not_people > 0 {
            warn!(
                "bevy_society: ReportDeed for '{}' names {not_people} witness(es) that are not persons; \
                 dropped",
                report.deed
            );
        }
        if witnesses.is_empty() {
            // A deed nobody saw is not a deed: no reactions and no rumor.
            continue;
        }
        witnesses.sort_unstable_by_key(|entity| {
            index
                .iter()
                .find(|(_, e)| e == entity)
                .map_or(u32::MAX, |(id, _)| id.0)
        });
        let patient = report.patient.unwrap_or(report.actor);
        witness_deed(
            world,
            deed,
            report.actor,
            patient,
            &defs,
            &settings,
            now,
            &witnesses,
        );
        // The victim keeps what the exchange path gives them: a memory sized `0.3 + |harm|` about
        // the actor. Without it `Memories::grudge` stays 0 and `snub` is unreachable for them.
        if let Some(patient) = report.patient
            && let Some(spec) = defs.deeds.get(usize::from(deed.0))
            && spec.harm < 0.0
            && let Some(mut memories) = world.get_mut::<crate::mind::Memories>(patient)
        {
            let magnitude = 0.3 + spec.harm.abs();
            memories.record(
                crate::mind::Memory {
                    day: now as f32,
                    kind: crate::mind::MemoryKind::Suffered {
                        deed,
                        actor: report.actor,
                    },
                    other: Some(report.actor),
                    valence: -magnitude.min(1.0),
                    // Design default: as heavy as the memory is bad.
                    importance: magnitude.min(1.0),
                },
                now,
                settings.memory_capacity,
            );
        }
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

/// The reads [`contact_gossip`] needs, grouped so the system stays inside clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
struct ContactGossipWorld<'w> {
    clock: Res<'w, SocietyClock>,
    settings: Res<'w, SocietySettings>,
    perception: Res<'w, Perception>,
    index: Res<'w, PersonIndex>,
    book: Res<'w, RumorBook>,
    defs: Res<'w, SocietyDefs>,
    queue: Option<ResMut<'w, InteractQueue>>,
}

/// **People speak when they have news** (plan D7b). Two people who share a place and are not otherwise
/// exchanging still talk *if the speaker has a rumor the listener does not know* — the gate the
/// `how-can-two-groups-ever-meet` decision never tried.
///
/// The earlier attempt queued a `chat` for every pair in contact, which nearly tripled all social contact
/// (13 742 requests over 17 game-days against a 7 500 baseline) and turned the village into a clique: 267
/// friend pairs out of 276 possible. The fault was not the rate but the *gate*: a `chat` is offered
/// whether or not either person has anything to say.
///
/// [`Knowledge::best_telling`] is self-limiting in exactly the way contact-chat was not. It requires a rumor
/// that is `alive`, whose subject is not the listener, that the teller still `spreader`s, and it ranks by
/// `|valence| · belief · 0.995^(hours since heard)` (Park et al. 2023, DOI 10.48550/arXiv.2304.03442). Once
/// the listener knows the rumor the telling stifles (Daley & Kendall 1964, DOI 10.1038/2041118a0), so the
/// pair stops speaking. Contact carries *news*, not company: a village with nothing to say stays quiet.
///
/// Proximity is the channel because that is what proximity is for — Eckland 1968, quoted in Buss 1993
/// (DOI 10.1037/0033-295x.100.2.204): "the chances are about 50-50 that the 'one and only' lives within
/// walking distance."
///
/// Determinism: iterates [`PersonIndex::sorted`] and each person's `contact` list in index order; draws
/// nothing from `SocietyRng`.
fn contact_gossip(
    world: ContactGossipWorld,
    knowledge: Query<&crate::Knowledge>,
    mut ties: Query<&mut Ties>,
) {
    let ContactGossipWorld {
        clock,
        settings,
        perception,
        index,
        book,
        defs,
        queue,
    } = world;
    let Some(mut queue) = queue else {
        return;
    };
    let Some(gossip_id) = defs.interaction_id("gossip") else {
        return;
    };
    let tuning = &settings.tuning;
    let now = clock.now_days as f32;
    for (i, &(_, entity)) in index.sorted.iter().enumerate() {
        let Ok(mine) = knowledge.get(entity) else {
            continue;
        };
        let Some(contacts) = perception.contact.get(i) else {
            continue;
        };
        for &j in contacts {
            let Some(&other) = perception.persons.get(j as usize) else {
                continue;
            };
            // **Never tell someone who already knows.** The Daley–Kendall rule *stifles* a teller that
            // repeats itself, and that is measured harm here: without this check every contact-tell to a
            // neighbour who already heard burns a spreader, and village rumor reach fell **0.58 -> 0.375**.
            // The check protects the spreader pool; the pair goes quiet because there is no news between
            // them, not because one of them has been silenced.
            let Ok(theirs) = knowledge.get(other) else {
                continue;
            };
            let telling = mine.best_telling(now, |rumor| {
                let entry = book.get(rumor)?;
                if !entry.alive || entry.subject == other || theirs.knows(rumor) {
                    return None;
                }
                Some(entry.valence)
            });
            if telling.is_none() {
                continue;
            }
            // A cooldown per directed tie: sharing a bench for a whole activity is one conversation,
            // not one per fixed step.
            let Ok(mut owned) = ties.get_mut(entity) else {
                continue;
            };
            let Some(tie) = owned.get_mut(other) else {
                continue;
            };
            if now - tie.last_contact_day < tuning.contact_gossip_cooldown_days {
                continue;
            }
            tie.last_contact_day = now;
            queue.0.push(crate::interact::Interact {
                actor: entity,
                target: other,
                interaction: "gossip".to_owned(),
            });
            let _ = gossip_id;
        }
    }
}

#[cfg(test)]
mod crowding_tests {
    use super::*;

    /// A place view at a given fill. Only the fields the price reads are set.
    fn place(capacity: u32, used: u32) -> PlaceView {
        PlaceView {
            entity: Entity::from_bits(7),
            tags: Vec::new(),
            capacity,
            used,
            position: Vec2::ZERO,
        }
    }

    /// **The price is charged only when there is no room.** A place with a seat to spare costs nothing, which is
    /// what keeps every uncrowded village bit-identical to the build before this rule; a full one costs `penalty`.
    #[test]
    fn a_place_with_room_is_free_and_a_full_one_is_priced() {
        let views = [place(4, 3)];
        let target = DoingTarget::Place(Entity::from_bits(7));
        assert_eq!(
            crowding_price(target, &views, 0.6),
            1.0,
            "three of four seats is a place with room: no price"
        );
        assert!(
            (crowding_price(
                DoingTarget::Place(Entity::from_bits(7)),
                &[place(4, 4)],
                0.6
            ) - 0.4)
                .abs()
                < 1.0e-6,
            "a full place must cost the penalty"
        );
    }

    /// **The price is a step, not a ramp, and it never becomes a wall.** Over-full still costs exactly `penalty`;
    /// a penalty of 1.0 is the only value that reaches zero.
    #[test]
    fn the_price_is_a_step_and_never_a_wall() {
        let target = DoingTarget::Place(Entity::from_bits(7));
        let full = crowding_price(target, &[place(2, 12)], 0.6);
        assert!(
            (full - 0.4).abs() < 1.0e-6,
            "twelve people at two seats costs the same as two: {full}"
        );
        assert!(
            crowding_price(target, &[place(2, 12)], 0.99) > 0.0,
            "a price below 1.0 must leave the option reachable, however full the place"
        );
    }

    /// **A penalty of zero is the identity**, for every target and every place, which is how a host (and this
    /// crate's own regression tests) can switch the mechanism off.
    #[test]
    fn a_zero_penalty_changes_nothing() {
        for target in [
            DoingTarget::Place(Entity::from_bits(7)),
            DoingTarget::Place(Entity::from_bits(9)),
            DoingTarget::Itself,
            DoingTarget::Point(Vec2::new(1.0, 2.0)),
        ] {
            assert_eq!(crowding_price(target, &[place(1, 99)], 0.0), 1.0);
        }
    }

    /// **Anything that is not a place pays nothing** — a person, a wander point, the person itself.
    #[test]
    fn a_non_place_target_is_free() {
        let views = [place(1, 1)];
        for target in [DoingTarget::Itself, DoingTarget::Point(Vec2::new(3.0, 4.0))] {
            assert_eq!(crowding_price(target, &views, 0.9), 1.0);
        }
    }
}

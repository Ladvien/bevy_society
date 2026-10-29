//! Absence has a social weight (issue #35): the people close to someone who is [`Away`] miss them, and a
//! return is an event.
//!
//! Everything here is mood. The *missing* is a [`Distress`](EmotionKind::Distress) toward the away
//! person, held at a level that grows with the days away and with how close the tie is, and capped; the
//! crate never lets it cause an activity, so a host that wants to say "Odo has not eaten since Hawise
//! left" reads the fact and the person from [`Emotions`] and [`AwaySince`] and writes the line itself.
//!
//! The model is separation distress as the mark of an attachment bond (Hazan & Shaver 1987, DOI
//! `10.1037/0022-3514.52.3.511`). The numbers are the crate's own and are labelled design defaults on
//! [`Tuning`](crate::Tuning); see [`Tuning::missing_cap`](crate::Tuning::missing_cap).

use bevy::prelude::*;

use crate::core::{Away, AwaySince, Deceased, PersonIndex, SocietyClock, SocietySettings, Tuning};
use crate::mind::{Appraise, EmotionKind, Emotions};
use crate::relationships::{Tie, Ties};

/// A person came back: written once, when [`Away`] is removed.
///
/// A person who died while away has no return: `Deceased` replaces `Away` and the crate writes
/// [`PersonDied`](crate::PersonDied) alone. The crate also gives every holder who missed the person a
/// reunion lift (`Joy`, a fraction of what they were missing), so a host reads the return here and the
/// feeling in [`Emotions`].
///
/// Removing `Away` leaves [`Dormant`](crate::Dormant) in place, as removing any required component does:
/// a host that is done with the absence removes both, or only `Away` when something else still keeps the
/// person dormant.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct PersonReturned {
    /// The person who came back.
    pub person: Entity,
    /// How long they were gone, in game days, from `Away` inserted to `Away` removed.
    pub days_away: f64,
}

/// Returns waiting for the next fixed step: `(person, days away)`.
///
/// **A resource rather than the message alone**, for the reason `DeathQueue` is one: an observer runs
/// whenever the world is mutated, possibly in `Update` between two fixed steps, and the reunion must run
/// inside the fixed step.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct ReturnQueue(pub(crate) Vec<(Entity, f64)>);

/// How close a holder is to `tie`'s target, in `[0, 1]`, or `None` when the tie is not a close one.
///
/// The line grief uses: a `Friend` label or a `familiarity_peak` at
/// [`Tuning::bereavement_familiarity`], and a closeness of `0.5 · (1 + affinity)` so an enemy's tie is
/// none at all (Prigerson et al. 2009, DOI `10.1371/journal.pmed.1000121`).
pub(crate) fn closeness(tie: &Tie, tuning: &Tuning) -> Option<f32> {
    let close = tie.label == crate::TieLabel::Friend
        || tie.familiarity_peak >= tuning.bereavement_familiarity;
    close.then(|| (0.5 * (1.0 + tie.affinity)).clamp(0.0, 1.0))
}

/// The distress a holder at `closeness` feels at `days` of absence:
/// `cap · closeness · (1 − e^(−days / missing_days))`. Saturating, so it never passes the cap.
fn missing_level(tuning: &Tuning, closeness: f32, days: f64) -> f32 {
    let grown = 1.0 - (-(days.max(0.0) as f32) / tuning.missing_days.max(f32::EPSILON)).exp();
    tuning.missing_cap * closeness * grown
}

/// Records when an absence began, unless a restore (or the host) already did.
fn on_away_added(
    add: On<Add<Away>>,
    clock: Res<SocietyClock>,
    since: Query<Has<AwaySince>>,
    mut commands: Commands,
) {
    if matches!(since.get(add.entity), Ok(false)) {
        commands
            .entity(add.entity)
            .insert(AwaySince(clock.now_days));
    }
}

/// Queues the return the moment `Away` goes.
fn on_away_removed(
    remove: On<Remove<Away>>,
    clock: Res<SocietyClock>,
    since: Query<&AwaySince>,
    mut queue: ResMut<ReturnQueue>,
    mut commands: Commands,
) {
    let days = since
        .get(remove.entity)
        .map_or(0.0, |since| (clock.now_days - since.0).max(0.0));
    queue.0.push((remove.entity, days));
    commands.entity(remove.entity).try_remove::<AwaySince>();
}

/// A death ends an absence without a return.
fn on_deceased_ends_absence(add: On<Add<Deceased>>, mut commands: Commands) {
    commands.entity(add.entity).try_remove::<Away>();
}

/// Keeps every holder's missing at the level the absence has earned: each society tick, for every living
/// holder who is not themselves away, a `Distress` toward each away person they are close to is raised to
/// `cap · closeness · (1 − e^(−days / missing_days))` if it is lower.
///
/// Raised **to** the level, not by it, so the feeling sits at the cap however many ticks pass and decays
/// away at the normal emotion half-life once the person is back. Runs after the mind has stepped, so the
/// level a host reads is the level the absence has earned, and the mood sees it from the next tick. With `missing_cap` at `0.0` it does nothing.
pub(crate) fn miss_the_away(
    settings: Res<SocietySettings>,
    clock: Res<SocietyClock>,
    index: Res<PersonIndex>,
    aways: Query<&AwaySince, (With<Away>, Without<Deceased>)>,
    mut holders: Query<(&Ties, &mut Emotions), (Without<Away>, Without<Deceased>)>,
) {
    let tuning = &settings.tuning;
    if tuning.missing_cap <= 0.0 {
        return;
    }
    let gone: Vec<(Entity, f64)> = index
        .sorted
        .iter()
        .filter_map(|(_, entity)| {
            aways
                .get(*entity)
                .ok()
                .map(|since| (*entity, clock.now_days - since.0))
        })
        .collect();
    if gone.is_empty() {
        return;
    }
    for (_, holder) in &index.sorted {
        let Ok((ties, mut emotions)) = holders.get_mut(*holder) else {
            continue;
        };
        for &(away, days) in &gone {
            let Some(tie) = ties.get(away) else {
                continue;
            };
            let Some(closeness) = closeness(tie, tuning) else {
                continue;
            };
            let level = missing_level(tuning, closeness, days);
            if level > 0.0 {
                emotions.raise_to(EmotionKind::Distress, level, Some(away));
            }
        }
    }
}

/// Announces every return recorded since the last fixed step and gives the holders who missed the person
/// their reunion: `Joy` toward them of `reunion_fraction` of what the absence had earned.
///
/// Before `apply_appraisals`, so the lift counts in the step it happened. A person who is `Deceased`
/// by now, or no longer in the index, is not announced.
pub(crate) fn announce_returns(
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<Deceased>>,
    away: Query<(), With<Away>>,
    mut queue: ResMut<ReturnQueue>,
    mut returned: MessageWriter<PersonReturned>,
    mut appraisals: MessageWriter<Appraise>,
    ties: Query<&Ties>,
) {
    let returns = std::mem::take(&mut queue.0);
    if returns.is_empty() {
        return;
    }
    let tuning = &settings.tuning;
    let mut ordered: Vec<(u32, Entity, f64)> = returns
        .into_iter()
        .filter_map(|(entity, days)| {
            index
                .sorted
                .iter()
                .find(|(_, e)| *e == entity)
                .map(|(id, _)| (id.0, entity, days))
        })
        .collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.total_cmp(&b.2)));
    ordered.dedup_by_key(|(id, _, _)| *id);
    for (_, person, days_away) in ordered {
        if deceased.get(person).is_ok() {
            continue;
        }
        returned.write(PersonReturned { person, days_away });
        if tuning.missing_cap <= 0.0 {
            continue;
        }
        for (_, holder) in &index.sorted {
            if *holder == person || deceased.get(*holder).is_ok() || away.get(*holder).is_ok() {
                continue;
            }
            let Some(tie) = ties.get(*holder).ok().and_then(|t| t.get(person)) else {
                continue;
            };
            let Some(closeness) = closeness(tie, tuning) else {
                continue;
            };
            let lift = tuning.reunion_fraction * missing_level(tuning, closeness, days_away);
            if lift > 0.0 {
                appraisals.write(Appraise {
                    person: *holder,
                    kind: EmotionKind::Joy,
                    intensity: lift,
                    toward: Some(person),
                });
            }
        }
    }
}

/// Registers the absence machinery. Called by `MindPlugin`, which owns emotions.
pub(crate) fn build(app: &mut App) {
    app.add_message::<PersonReturned>()
        .init_resource::<ReturnQueue>()
        .add_observer(on_away_added)
        .add_observer(on_away_removed)
        .add_observer(on_deceased_ends_absence);
}

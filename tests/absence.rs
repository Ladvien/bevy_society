//! **Absence has a social weight** (issue #35).
//!
//! A party sent out of town is `Away`. The people close to them miss them — a `Distress` toward the away
//! person that grows with the days gone and the closeness of the tie and stops at a cap — and a return
//! is a `PersonReturned` event with a reunion lift. The mechanism is separation distress as the mark of an
//! attachment bond (Hazan & Shaver 1987, DOI `10.1037/0022-3514.52.3.511`); the numbers are design defaults.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{
    Away, AwaySince, Deceased, Dormant, EmotionKind, Emotions, Mood, PersonDied, PersonReturned,
    SocietyClock, SocietyDefs, SocietySettings, TieLabel, Ties,
};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.2)],
            activities: [(
                id: "wander",
                target: Wander(1.0),
                duration_hours: 1.0,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
            )],
        )"#,
    )
    .expect("the test definitions parse")
}

fn app_with(settings: SocietySettings) -> App {
    let mut app = headless(defs(), settings);
    app.update();
    app.update();
    app
}

fn app() -> App {
    app_with(common::settings())
}

fn befriend(app: &mut App, from: Entity, to: Entity, affinity: f32) {
    let mut ties = app.world_mut().get_mut::<Ties>(from).expect("ties");
    let tie = ties.entry_with(to, affinity);
    tie.affinity = affinity;
    tie.familiarity_peak = 0.9;
    tie.label = TieLabel::Friend;
    // And back, at the same level: a one-sided tie is pulled to the middle by dyad balance, which would
    // move the very closeness these tests measure.
    let mut ties = app.world_mut().get_mut::<Ties>(to).expect("ties");
    let tie = ties.entry_with(from, affinity);
    tie.affinity = affinity;
    tie.familiarity_peak = 0.9;
    tie.label = TieLabel::Friend;
}

fn now(app: &App) -> f64 {
    app.world().resource::<SocietyClock>().now_days
}

/// Runs whole ticks until `days` more game-days have passed; returns the returns and deaths announced.
fn run_days(app: &mut App, days: f64) -> (Vec<PersonReturned>, usize) {
    let until = now(app) + days;
    let mut returned = Vec::new();
    let mut deaths = 0;
    while now(app) < until {
        app.update();
        returned.extend(
            app.world_mut()
                .resource_mut::<Messages<PersonReturned>>()
                .drain(),
        );
        deaths += app
            .world_mut()
            .resource_mut::<Messages<PersonDied>>()
            .drain()
            .count();
    }
    (returned, deaths)
}

/// The `Distress` a holder feels toward `away`, `0.0` when there is none.
fn missing(app: &App, holder: Entity, away: Entity) -> f32 {
    app.world()
        .get::<Emotions>(holder)
        .and_then(|emotions| {
            emotions
                .0
                .iter()
                .find(|e| e.kind == EmotionKind::Distress && e.toward == Some(away))
                .map(|e| e.intensity)
        })
        .unwrap_or(0.0)
}

fn joy_toward(app: &App, holder: Entity, other: Entity) -> f32 {
    app.world()
        .get::<Emotions>(holder)
        .and_then(|emotions| {
            emotions
                .0
                .iter()
                .find(|e| e.kind == EmotionKind::Joy && e.toward == Some(other))
                .map(|e| e.intensity)
        })
        .unwrap_or(0.0)
}

fn pleasure(app: &App, who: Entity) -> f32 {
    app.world().get::<Mood>(who).expect("a mood").pleasure
}

/// **Criterion 1.** A friend of the away person is lower in mood than a control with no tie, and the
/// control feels nothing.
#[test]
fn the_friend_of_the_away_misses_them_and_a_stranger_does_not() {
    let mut app = app();
    let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let control = spawn_person(&mut app, "Cole", Vec2::new(-2.0, 0.0));
    let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, friend, away, 0.8);
    let before = pleasure(&app, friend) - pleasure(&app, control);

    app.world_mut().entity_mut(away).insert(Away);
    run_days(&mut app, 2.0);

    assert!(
        missing(&app, friend, away) > 0.1,
        "a friend is distressed at two days away: {}",
        missing(&app, friend, away)
    );
    assert_eq!(
        missing(&app, control, away),
        0.0,
        "a person with no tie to them is not"
    );
    let gap = pleasure(&app, friend) - pleasure(&app, control);
    assert!(
        gap < before - 0.03,
        "the friend's pleasure is lower than the control's by more than it was ({before} then {gap})"
    );
    assert!(
        app.world().get::<Dormant>(away).is_some(),
        "Away implies Dormant"
    );
}

/// **Criterion 2.** Monotone in closeness and in days, and it stops at the cap.
#[test]
fn missing_grows_with_closeness_and_days_and_stops_at_the_cap() {
    let mut settings = common::settings();
    // A short time constant, so saturation is reached in days a test can afford.
    settings.tuning.missing_days = 0.5;
    let mut app = app_with(settings);
    let close = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let fond = spawn_person(&mut app, "Cole", Vec2::new(-2.0, 0.0));
    let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, close, away, 0.95);
    befriend(&mut app, fond, away, 0.2);
    let cap = app.world().resource::<SocietySettings>().tuning.missing_cap;

    app.world_mut().entity_mut(away).insert(Away);
    run_days(&mut app, 0.25);
    let early = missing(&app, close, away);
    run_days(&mut app, 0.75);
    let later = missing(&app, close, away);
    assert!(
        early > 0.0 && later > early,
        "grows with days: {early} -> {later}"
    );
    assert!(
        missing(&app, close, away) > missing(&app, fond, away),
        "grows with closeness"
    );

    run_days(&mut app, 2.0);
    let long = missing(&app, close, away);
    assert!(long <= cap, "never passes the cap: {long} vs {cap}");
    // The tie itself drifts over a few days (relaxation, balance), so saturation is read against the
    // closeness the tie has *now*: the level is the cap times that, with the growth term spent.
    let affinity = app
        .world()
        .get::<Ties>(close)
        .and_then(|ties| ties.get(away))
        .map(|tie| tie.affinity)
        .expect("the tie is kept");
    let ceiling = cap * 0.5 * (1.0 + affinity);
    assert!(
        (long - ceiling).abs() < 0.02 * cap,
        "it sits at cap · closeness, the growth spent: {long} vs {ceiling}"
    );
}

/// **Criterion 3.** One `PersonReturned`, whose length is the elapsed days, and the missing relaxes.
#[test]
fn a_return_is_announced_once_and_the_missing_relaxes() {
    let mut app = app();
    let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, friend, away, 0.9);

    let left = now(&app);
    app.world_mut().entity_mut(away).insert(Away);
    run_days(&mut app, 3.0);
    let missed = missing(&app, friend, away);
    let gone_for = now(&app) - left;
    assert!(
        app.world().get::<AwaySince>(away).is_some(),
        "the crate recorded when they left"
    );

    app.world_mut().entity_mut(away).remove::<Away>();
    let (returned, _) = run_days(&mut app, 0.05);
    let mine: Vec<_> = returned.iter().filter(|r| r.person == away).collect();
    assert_eq!(mine.len(), 1, "one PersonReturned");
    assert!(
        (mine[0].days_away - gone_for).abs() < 1e-3,
        "days away {} vs elapsed {gone_for}",
        mine[0].days_away
    );
    assert!(
        joy_toward(&app, friend, away) > 0.05,
        "a reunion lift for the friend who missed them"
    );
    assert!(
        app.world().get::<AwaySince>(away).is_none(),
        "the record goes with the absence"
    );

    let (more, _) = run_days(&mut app, 1.0);
    assert!(more.iter().all(|r| r.person != away), "and only one");
    assert!(
        missing(&app, friend, away) < missed * 0.1,
        "a day after the return the missing is gone: {} from {missed}",
        missing(&app, friend, away)
    );
}

/// An absence added and removed in the same step has no length and still announces once.
#[test]
fn an_absence_of_no_length_is_a_return_of_zero_days() {
    let mut app = app();
    let away = spawn_person(&mut app, "Ada", Vec2::ZERO);
    app.update();
    app.world_mut().entity_mut(away).insert(Away);
    app.world_mut().entity_mut(away).remove::<Away>();
    let (returned, _) = run_days(&mut app, 0.05);
    assert_eq!(returned.len(), 1);
    assert!(returned[0].days_away.abs() < 1e-9);
}

/// **Edge.** A party away together does not miss each other.
#[test]
fn a_party_away_together_does_not_miss_each_other() {
    let mut app = app();
    let a = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    let b = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, a, b, 0.9);
    befriend(&mut app, b, a, 0.9);
    app.world_mut().entity_mut(a).insert(Away);
    app.world_mut().entity_mut(b).insert(Away);
    run_days(&mut app, 3.0);
    assert_eq!(missing(&app, a, b), 0.0);
    assert_eq!(missing(&app, b, a), 0.0);
}

/// **Criterion 6.** `Deceased` replaces `Away`: one `PersonDied`, no `PersonReturned`.
#[test]
fn a_death_while_away_is_a_death_and_not_a_return() {
    let mut app = app();
    let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, friend, away, 0.9);
    app.world_mut().entity_mut(away).insert(Away);
    run_days(&mut app, 1.0);

    app.world_mut().entity_mut(away).insert(Deceased);
    let (returned, deaths) = run_days(&mut app, 0.02);
    assert_eq!(deaths, 1, "one PersonDied");
    assert!(returned.is_empty(), "no PersonReturned: {returned:?}");
    assert!(
        app.world().get::<Away>(away).is_none(),
        "Deceased ended the absence"
    );
    assert!(
        missing(&app, friend, away) > 0.4,
        "grief at full weight, not the capped missing"
    );
}

/// **Criterion 4.** With `missing_cap` at zero nothing is felt, and a return still happens.
#[test]
fn a_zero_cap_switches_the_weight_off() {
    let mut settings = common::settings();
    settings.tuning.missing_cap = 0.0;
    let mut app = app_with(settings);
    let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, friend, away, 0.9);
    app.world_mut().entity_mut(away).insert(Away);
    run_days(&mut app, 3.0);
    assert_eq!(missing(&app, friend, away), 0.0);
    app.world_mut().entity_mut(away).remove::<Away>();
    let (returned, _) = run_days(&mut app, 0.05);
    assert_eq!(returned.len(), 1);
    assert_eq!(joy_toward(&app, friend, away), 0.0);
}

/// **Criterion 5.** A save in the middle of a trip restores the marker and the length so far.
#[cfg(feature = "serde")]
#[test]
fn a_save_mid_absence_keeps_the_marker_and_the_length() {
    use bevy_society::{PersonKey, capture, restore};

    let build = || {
        let mut app = app();
        let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
        let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
        app.world_mut().entity_mut(friend).insert(PersonKey(1));
        app.world_mut().entity_mut(away).insert(PersonKey(2));
        app.update();
        app.update();
        (app, friend, away)
    };
    let (mut app, friend, away) = build();
    befriend(&mut app, friend, away, 0.9);
    let left = now(&app);
    app.world_mut().entity_mut(away).insert(Away);
    run_days(&mut app, 2.0);
    let elapsed_at_save = now(&app) - left;
    let save = capture(app.world_mut());
    let json = serde_json::to_string(&save).expect("plain data");
    let save = serde_json::from_str(&json).expect("reads back");

    let (mut fresh, friend, away) = build();
    restore(fresh.world_mut(), save).expect("the save applies");
    assert!(fresh.world().get::<Away>(away).is_some(), "still away");
    let _ = friend;

    run_days(&mut fresh, 1.0);
    fresh.world_mut().entity_mut(away).remove::<Away>();
    let (returned, _) = run_days(&mut fresh, 0.05);
    assert_eq!(returned.len(), 1);
    let expected = elapsed_at_save + 1.0;
    assert!(
        (returned[0].days_away - expected).abs() < 0.05,
        "counts the time before the save: {} vs {expected}",
        returned[0].days_away
    );
}

/// **A living town.** A friend who lives an autonomous life keeps deciding while the other leaves:
/// the one away starts no activity for the whole absence, is the one missed, and starts living again
/// after the return.
#[test]
fn an_autonomous_party_member_leaves_and_comes_back_into_the_day() {
    use bevy_society::{ActivityStarted, Autonomous};
    let mut app = app();
    let friend = spawn_person(&mut app, "Bram", Vec2::new(2.0, 0.0));
    let away = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    befriend(&mut app, friend, away, 0.3);
    app.world_mut().entity_mut(friend).insert(Autonomous);
    app.world_mut().entity_mut(away).insert(Autonomous);

    app.world_mut().entity_mut(away).insert(Away);
    app.world_mut()
        .resource_mut::<Messages<ActivityStarted>>()
        .clear();
    let mut started_by = Vec::new();
    let until = now(&app) + 1.5;
    while now(&app) < until {
        app.update();
        started_by.extend(
            app.world_mut()
                .resource_mut::<Messages<ActivityStarted>>()
                .drain()
                .map(|m| m.person),
        );
    }
    assert!(started_by.contains(&friend), "the town keeps living");
    assert!(!started_by.contains(&away), "the one away decides nothing");
    assert!(missing(&app, friend, away) > 0.05);

    app.world_mut().entity_mut(away).remove::<Away>();
    app.world_mut()
        .resource_mut::<Messages<ActivityStarted>>()
        .clear();
    let mut started = Vec::new();
    let mut returned = 0;
    let until = now(&app) + 1.0;
    while now(&app) < until {
        app.update();
        returned += app
            .world_mut()
            .resource_mut::<Messages<PersonReturned>>()
            .drain()
            .count();
        started.extend(
            app.world_mut()
                .resource_mut::<Messages<ActivityStarted>>()
                .drain()
                .map(|m| m.person),
        );
    }
    assert_eq!(returned, 1);
    assert!(started.contains(&away), "after the return they live again");
}

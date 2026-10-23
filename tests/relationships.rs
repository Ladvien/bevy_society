//! Relationship dynamics (plan D4): familiarity, mere exposure, balance, capacity and labels.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{SocietyClock, SocietyDefs, SocietySettings, TieLabel, TieLabelChanged, Ties};
use common::{headless, run, settings, spawn_person};

/// Settings where one game-day is 64 ticks, so `n` updates advance `n / 64` game-days.
fn day_settings() -> SocietySettings {
    let mut s = settings();
    s.seconds_per_day = 1.0;
    s
}

fn now_days(app: &App) -> f32 {
    app.world().resource::<SocietyClock>().now_days as f32
}

fn tuning(app: &App) -> bevy_society::Tuning {
    app.world().resource::<SocietySettings>().tuning.clone()
}

fn set_pos(app: &mut App, entity: Entity, x: f32) {
    if let Some(mut transform) = app.world_mut().get_mut::<Transform>(entity) {
        transform.translation.x = x;
    }
}

fn drain_label_changes(app: &mut App) -> Vec<TieLabelChanged> {
    app.world_mut()
        .resource_mut::<Messages<TieLabelChanged>>()
        .drain()
        .collect()
}

#[test]
fn two_metres_apart_for_a_game_day_builds_familiarity_both_ways() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let b = spawn_person(&mut app, "b", Vec2::new(2.0, 0.0));
    run(&mut app, 64);

    let now = now_days(&app);
    let tuning = tuning(&app);
    let ab = app
        .world()
        .get::<Ties>(a)
        .and_then(|t| t.get(b))
        .map(|tie| tie.familiarity(now, &tuning))
        .expect("a has a tie to b");
    let ba = app
        .world()
        .get::<Ties>(b)
        .and_then(|t| t.get(a))
        .map(|tie| tie.familiarity(now, &tuning))
        .expect("b has a tie to a");
    // The exposure recurrence f += 2·dt·(1 − f) per tick settles near 0.73 after one game-day.
    assert!(ab >= 0.7, "a→b familiarity {ab}");
    assert!(ba >= 0.7, "b→a familiarity {ba}");
}

#[test]
fn familiarity_decays_as_power_law_after_contact_stops() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let b = spawn_person(&mut app, "b", Vec2::new(2.0, 0.0));
    run(&mut app, 64);

    // Move b out of contact range (4 m) and let three game-days pass.
    let b_entity = b;
    set_pos(&mut app, b_entity, 20.0);
    run(&mut app, 192);

    let now = now_days(&app);
    let tuning = tuning(&app);
    let tie = app
        .world()
        .get::<Ties>(a)
        .and_then(|t| t.get(b))
        .copied()
        .expect("a has a tie to b");
    let elapsed = now - tie.last_contact_day;
    assert!((elapsed - 3.0).abs() < 0.02, "elapsed {elapsed} game-days");

    let expected = tie.familiarity_peak * (1.0 + elapsed).powf(-0.5);
    let measured = tie.familiarity(now, &tuning);
    assert!(
        (measured - expected).abs() < 1e-3,
        "familiarity {measured} vs closed form {expected}"
    );
}

#[test]
fn structural_balance_pushes_a_to_c_negative_when_a_likes_b_and_b_dislikes_c() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let b = spawn_person(&mut app, "b", Vec2::new(20.0, 0.0));
    let c = spawn_person(&mut app, "c", Vec2::new(40.0, 0.0));

    app.world_mut()
        .get_mut::<Ties>(a)
        .expect("a has ties")
        .entry(b)
        .affinity = 0.8;
    app.world_mut()
        .get_mut::<Ties>(a)
        .expect("a has ties")
        .entry(c)
        .affinity = 0.0;
    app.world_mut()
        .get_mut::<Ties>(b)
        .expect("b has ties")
        .entry(c)
        .affinity = -0.8;

    run(&mut app, 64);

    let ac = app
        .world()
        .get::<Ties>(a)
        .and_then(|t| t.get(c))
        .map(|tie| tie.affinity)
        .expect("a still has a tie to c");
    assert!(ac < -0.05, "a→c affinity after one game-day: {ac}");
}

#[test]
fn capacity_caps_ties_at_max_ties() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let max_ties = app.world().resource::<SocietySettings>().max_ties;
    let mut people = Vec::new();
    for i in 0..160 {
        let x = i as f32 * 0.005;
        people.push(spawn_person(&mut app, &format!("p{i}"), Vec2::new(x, 0.0)));
    }
    run(&mut app, 3);

    let mut hit_cap = false;
    for &entity in &people {
        let len = app.world().get::<Ties>(entity).map_or(0, Ties::len);
        assert!(len <= max_ties, "ties {len} exceed max_ties {max_ties}");
        hit_cap |= len == max_ties;
    }
    assert!(
        hit_cap,
        "no person reached the tie cap, so the test proves nothing"
    );
}

#[test]
fn despawning_a_person_removes_every_tie_to_it() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let b = spawn_person(&mut app, "b", Vec2::new(2.0, 0.0));
    let c = spawn_person(&mut app, "c", Vec2::new(0.0, 2.0));
    run(&mut app, 10);
    assert!(app.world().get::<Ties>(a).and_then(|t| t.get(b)).is_some());

    app.world_mut().entity_mut(b).despawn();
    run(&mut app, 1);

    for owner in [a, c] {
        let ties = app.world().get::<Ties>(owner).expect("owner has ties");
        assert!(ties.get(b).is_none(), "{owner:?} still holds a tie to b");
    }
}

#[test]
fn crossing_into_friendship_writes_exactly_one_label_change() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let b = spawn_person(&mut app, "b", Vec2::new(20.0, 0.0));
    let now = now_days(&app);
    {
        let mut ties = app.world_mut().get_mut::<Ties>(a).expect("a has ties");
        let tie = ties.entry(b);
        tie.affinity = 0.45;
        tie.familiarity_peak = 0.5;
        tie.last_contact_day = now;
    }

    let mut changes = Vec::new();
    for _ in 0..10 {
        app.update();
        changes.extend(drain_label_changes(&mut app));
    }

    assert_eq!(changes.len(), 1, "label changes: {changes:?}");
    let change = &changes[0];
    assert_eq!(change.from, a);
    assert_eq!(change.to, b);
    assert!(
        matches!(change.new, TieLabel::Friend),
        "new label {:?}",
        change.new
    );
}

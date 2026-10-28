//! Core: spawn-order ids, index maintenance, despawn cleanup, the tick gate and the clock.

use crate::common;

use bevy::app::FixedUpdate;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::ResMut;
use bevy::math::Vec2;
use bevy_society::{PersonId, PersonIndex, SocietyClock, SocietyDefs, SocietySystems, Ties};

/// Counts how many times a system in `SocietySystems::Mind` ran.
#[derive(Resource, Default)]
struct MindRuns(u32);

fn count_mind(mut runs: ResMut<MindRuns>) {
    runs.0 += 1;
}

fn id_of(app: &bevy::app::App, entity: Entity) -> PersonId {
    match app.world().get::<PersonId>(entity) {
        Some(id) => *id,
        None => panic!("entity {entity:?} has no PersonId"),
    }
}

#[test]
fn ids_follow_spawn_order_and_match_the_sorted_index() {
    let mut app = common::headless(SocietyDefs::default(), common::settings());
    let a = common::spawn_person(&mut app, "a", Vec2::ZERO);
    let b = common::spawn_person(&mut app, "b", Vec2::new(1.0, 0.0));
    let c = common::spawn_person(&mut app, "c", Vec2::new(2.0, 0.0));
    app.update();

    let (ia, ib, ic) = (id_of(&app, a), id_of(&app, b), id_of(&app, c));
    assert_eq!(ib.0, ia.0 + 1, "ids are consecutive in spawn order");
    assert_eq!(ic.0, ib.0 + 1, "ids are consecutive in spawn order");

    let index = app.world().resource::<PersonIndex>();
    let order: Vec<Entity> = index.sorted.iter().map(|(_, e)| *e).collect();
    assert_eq!(order, vec![a, b, c], "sorted walks persons in spawn order");
    assert_eq!(
        index.by_id.get(&ib).copied(),
        Some(b),
        "by_id resolves the same person"
    );
}

#[test]
fn removing_a_person_drops_it_from_the_index_and_from_others_ties() {
    let mut app = common::headless(SocietyDefs::default(), common::settings());
    let a = common::spawn_person(&mut app, "a", Vec2::ZERO);
    let b = common::spawn_person(&mut app, "b", Vec2::new(1.0, 0.0));
    let c = common::spawn_person(&mut app, "c", Vec2::new(2.0, 0.0));
    app.update();

    let b_id = id_of(&app, b);
    if let Some(mut ties) = app.world_mut().get_mut::<Ties>(a) {
        ties.entry(b).affinity = 0.5;
    }
    if let Some(mut ties) = app.world_mut().get_mut::<Ties>(c) {
        ties.entry(b).affinity = -0.5;
    }
    if let Some(mut ties) = app.world_mut().get_mut::<Ties>(c) {
        ties.entry(a).affinity = 0.1;
    }

    app.world_mut().despawn(b);
    app.update();

    let index = app.world().resource::<PersonIndex>();
    assert_eq!(index.sorted.len(), 2, "sorted drops the removed person");
    assert!(
        !index.by_id.contains_key(&b_id),
        "by_id drops the removed person"
    );
    let order: Vec<Entity> = index.sorted.iter().map(|(_, e)| *e).collect();
    assert_eq!(order, vec![a, c]);

    for other in [a, c] {
        let ties = match app.world().get::<Ties>(other) {
            Some(t) => t,
            None => panic!("survivor {other:?} lost its Ties component"),
        };
        assert!(
            ties.get(b).is_none(),
            "no tie may still target the removed person"
        );
    }
    let c_ties = match app.world().get::<Ties>(c) {
        Some(t) => t,
        None => panic!("c lost its Ties component"),
    };
    assert!(
        c_ties.get(a).is_some(),
        "unrelated ties survive the removal"
    );
}

#[test]
fn a_gated_set_runs_once_per_due_tick() {
    let mut settings = common::settings();
    settings.tick_secs = 2.0 / 64.0;
    let mut app = common::headless(SocietyDefs::default(), settings);
    app.init_resource::<MindRuns>();
    app.add_systems(FixedUpdate, count_mind.in_set(SocietySystems::Mind));

    // One warm-up update settles the accumulator phase; with a tick every two 1/64 s updates,
    // ten further updates contain exactly five due ticks.
    app.update();
    app.world_mut().resource_mut::<MindRuns>().0 = 0;
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(app.world().resource::<MindRuns>().0, 5);
}

#[test]
fn the_clock_starts_at_the_configured_hour_and_advances() {
    let mut settings = common::settings();
    settings.start_hour = 15.0;
    let mut app = common::headless(SocietyDefs::default(), settings);
    app.update();

    let clock = app.world().resource::<SocietyClock>();
    assert!(
        (clock.hour() - 15.0).abs() < 0.1,
        "clock starts near start_hour, got {}",
        clock.hour()
    );
    let tick_before = clock.tick;
    app.update();
    assert_eq!(app.world().resource::<SocietyClock>().tick, tick_before + 1);
}

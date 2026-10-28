//! **Every tie that becomes a `Rival` must eventually be able to stop being one, and the harness must count
//! both halves.**
//!
//! `summary.json` used to report `rivals` — how many exist — and nothing else. A count that only rises cannot
//! distinguish a society that holds grudges from one that never lets them go, and the harness could not tell you
//! which it had. `became_rival` and `left_rival` now count the `TieLabelChanged` transitions either side of the
//! label, and the difference between them is exactly the number of rival ties standing.
//!
//! That identity is the thing worth pinning: it is arithmetic over the two counters, it holds for any dynamics,
//! and it is what makes the ratio meaningful. Measured on `village-evening` (24 people, 17 game-days, seed 7):
//! **272 formed, 188 ended, 84 standing** — 42 undirected rival pairs, which is what `rivals` reports.
use crate::common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Home, Place, PlaceTags, SocietyDefs, TieLabel, TieLabelChanged, Ties,
};

/// A handful of people in one square with two social activities, so ties form and break.
fn village(seed: u64) -> App {
    let defs = SocietyDefs::from_ron(
        r#"(
            needs: [
                (id: "social", decay_per_day: 3.0, initial: 0.6),
                (id: "fun", decay_per_day: 2.0, initial: 0.6),
            ],
            activities: [
                (
                    id: "chat",
                    target: Person,
                    duration_hours: 0.3,
                    rank: 1,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    interaction: Some("chat"),
                ),
                (
                    id: "insult",
                    target: Person,
                    duration_hours: 0.2,
                    rank: 1,
                    considerations: [(input: Need("fun"), curve: Linear(-1.0, 1.0))],
                    interaction: Some("insult"),
                ),
            ],
            interactions: [
                (id: "chat", accept: (base: 0.9)),
                (
                    id: "insult",
                    on_accept: (target_to_actor: (affinity: -0.6)),
                    accept: (base: 0.9),
                ),
            ],
        )"#,
    )
    .expect("the definitions parse");
    let mut settings = common::settings();
    settings.seed = seed;
    // A short day and a slow relaxation, so a grudge outlives a walk and rivalries accumulate.
    settings.seconds_per_day = 1.0;
    settings.tuning.affinity_relax_per_day = 0.005;
    let mut app = common::headless(defs, settings);
    let square = app
        .world_mut()
        .spawn((
            Name::new("square"),
            Place { capacity: 64 },
            PlaceTags(vec!["gathering".to_owned()]),
            Transform::default(),
        ))
        .id();
    for i in 0..10 {
        let angle = std::f32::consts::TAU * (i as f32) / 10.0;
        let person = common::spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(0.8 * angle.cos(), 0.8 * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Home(square), Autonomous));
    }
    app
}

/// Runs and returns `(became rival, left rival)` exactly as `summary.json` reports them.
fn transitions(app: &mut App, ticks: usize) -> (usize, usize) {
    let (mut became, mut left) = (0usize, 0usize);
    for _ in 0..ticks {
        app.update();
        for message in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TieLabelChanged>>()
            .drain()
        {
            if message.new == TieLabel::Rival {
                became += 1;
            } else if message.old == TieLabel::Rival {
                left += 1;
            }
        }
    }
    (became, left)
}

/// The rival ties standing in the world, counted the way `summary.json` counts them (undirected).
fn standing_rivals(app: &mut App) -> usize {
    let directed = app
        .world_mut()
        .query::<&Ties>()
        .iter(app.world())
        .flat_map(|ties| ties.0.iter())
        .filter(|tie| tie.label == TieLabel::Rival)
        .count();
    directed / 2
}

#[test]
fn becoming_minus_leaving_equals_the_rivals_standing() {
    // **The identity the two counters exist for.** It holds whatever the dynamics do, so it catches a counter
    // wired to the wrong side of the transition — which is the way this metric would silently lie.
    let mut app = village(7);
    let (became, left) = transitions(&mut app, 20_000);
    let standing = standing_rivals(&mut app);
    assert!(
        became > 0,
        "the fixture must form rivalries at all ({became} formed)"
    );
    assert_eq!(
        (became - left) / 2,
        standing,
        "became {became} minus left {left} is {} undirected rivalries, but {standing} are standing",
        (became - left) / 2
    );
}

#[test]
fn a_village_that_forms_rivalries_can_also_end_them() {
    // The property that makes the ratio worth reporting. If `left` were always zero the village would be a
    // feud, and `summary.json` would report a rising `rivals` count with no way to say whether that was
    // accumulation or irreversibility. This does not pin *how often* reconciliation happens — that is a rate
    // question the village measures at 1.27-1.45 across seeds — only that the mechanism exists.
    let mut app = village(11);
    let (became, left) = transitions(&mut app, 20_000);
    assert!(
        became > 0,
        "the fixture must form rivalries ({became} formed)"
    );
    assert!(
        left > 0,
        "{became} rivalries formed and none ended: a tie that can become a `Rival` must be able to stop being \\
         one, or the label is a one-way trap and every `rivals` count is a lifetime total"
    );
    let ratio = became as f64 / left as f64;
    assert!(
        ratio < 20.0,
        "forming outpaces ending by {ratio:.1}x, which is a feud rather than a society"
    );
}

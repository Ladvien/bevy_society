//! Mind: emotion half-life, mood lift and return, and needs decay over simulated time.
#![allow(clippy::unwrap_used)]

mod common;

use bevy::ecs::name::Name;
use bevy::math::Vec2;
use bevy::transform::components::Transform;
use bevy_society::{
    Appraise, EmotionKind, Emotions, Mood, Needs, Person, Personality, SocietyClock, SocietyDefs,
    SocietySettings,
};
use common::{headless, run, settings};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Sixteen game-seconds per day, so one tick at 64 Hz is 1/1024 of a game-day.
fn fast_settings() -> SocietySettings {
    SocietySettings {
        seconds_per_day: 16.0,
        ..settings()
    }
}

/// Runs updates until one fixed step has run. The first `update` only primes `Time<Fixed>`, so
/// systems in fixed steps (appraisals, sizing of `Needs`) have not run yet.
fn warm_up(app: &mut bevy::app::App) {
    for _ in 0..16 {
        if app.world().resource::<SocietyClock>().tick > 0 {
            return;
        }
        app.update();
    }
    assert!(
        app.world().resource::<SocietyClock>().tick > 0,
        "no fixed step ran during warm-up"
    );
}

fn now_days(app: &bevy::app::App) -> f64 {
    app.world().resource::<SocietyClock>().now_days
}

/// Intensity of the `Joy` emotion with no target, or 0 once it has been dropped.
fn joy_intensity(app: &bevy::app::App, person: bevy::ecs::entity::Entity) -> f32 {
    app.world()
        .get::<Emotions>(person)
        .unwrap()
        .0
        .iter()
        .find(|e| e.kind == EmotionKind::Joy && e.toward.is_none())
        .map_or(0.0, |e| e.intensity)
}

fn mood_of(app: &bevy::app::App, person: bevy::ecs::entity::Entity) -> [f32; 3] {
    app.world().get::<Mood>(person).unwrap().as_array()
}

#[test]
fn joy_intensity_halves_every_tenth_of_a_game_day() {
    let mut app = headless(SocietyDefs::default(), fast_settings());
    let person = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    warm_up(&mut app);
    app.world_mut().write_message(Appraise {
        person,
        kind: EmotionKind::Joy,
        intensity: 1.0,
        toward: None,
    });
    app.update();
    let (i0, d0) = (joy_intensity(&app, person), now_days(&app));
    run(&mut app, 100);
    let (i1, d1) = (joy_intensity(&app, person), now_days(&app));
    // Emotion half-life is 0.1 game-days (`Tuning::emotion_half_life_days`), Gebhard 2005 style decay.
    let expected = i0 * 0.5_f32.powf((d1 - d0) as f32 / 0.1);
    assert!(
        (i1 - expected).abs() < 1e-3,
        "intensity {i1} after {} days, expected {expected}",
        d1 - d0
    );
}

#[test]
fn joy_lifts_mood_toward_joy_and_then_returns_to_baseline() {
    let mut app = headless(SocietyDefs::default(), fast_settings());
    // All-zero traits give a low-pleasure baseline, so the Joy lift has room to show.
    let person = app
        .world_mut()
        .spawn((
            Person,
            Name::new("Ada"),
            Transform::default(),
            Personality {
                openness: 0.0,
                conscientiousness: 0.0,
                extraversion: 0.0,
                agreeableness: 0.0,
                neuroticism: 0.0,
            },
        ))
        .id();
    let baseline = app
        .world()
        .get::<Personality>(person)
        .unwrap()
        .baseline_mood()
        .as_array();
    assert_eq!(mood_of(&app, person), baseline, "spawned at baseline");

    app.world_mut().write_message(Appraise {
        person,
        kind: EmotionKind::Joy,
        intensity: 1.0,
        toward: None,
    });
    run(&mut app, 64);
    let lifted = mood_of(&app, person);
    assert!(
        lifted[0] > baseline[0] + 0.05,
        "pleasure {} should rise above baseline {} + 0.05",
        lifted[0],
        baseline[0]
    );

    // Six game-days with no further appraisals: the emotion has decayed away and mood returns.
    run(&mut app, 6144);
    let settled = mood_of(&app, person);
    for axis in 0..3 {
        assert!(
            (settled[axis] - baseline[axis]).abs() < 0.01,
            "axis {axis}: {} should be within 0.01 of baseline {}",
            settled[axis],
            baseline[axis]
        );
    }
}

#[test]
fn needs_fall_at_decay_per_day() -> TestResult {
    let defs = SocietyDefs::from_ron(r#"(needs: [(id: "social", decay_per_day: 0.5)])"#)?;
    let mut app = headless(defs, fast_settings());
    let person = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    warm_up(&mut app);
    let level = |app: &bevy::app::App| app.world().get::<Needs>(person).unwrap().0[0];
    let (n0, d0) = (level(&app), now_days(&app));
    run(&mut app, 512);
    let (n1, d1) = (level(&app), now_days(&app));
    let expected = (n0 - 0.5 * (d1 - d0) as f32).max(0.0);
    assert!(
        (n1 - expected).abs() < 1e-3,
        "need {n1} after {} days, expected {expected}",
        d1 - d0
    );
    Ok(())
}

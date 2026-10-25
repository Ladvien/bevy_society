//! **The crate can say whether its own layout is walkable.**
//!
//! `warn_short_travel_reach` has to stay quiet above a generic floor because `finish()` cannot see the world's
//! places. A system can: every `Place` carries a `Transform`, so at `Startup` the crate knows exactly how far
//! apart they are. `warn_unreachable_places` uses that to name the pair a timeout cannot cover.
//!
//! Measured on the project's own `village-evening`: a 4-game-hour timeout at 64 s/day buys **14.9 m** of
//! walking, the farthest pair is `market -> workshop` at **16.4 m**, and the warning prescribes **4.4** hours.
//! The generic floor did not fire, because 14.9 m is above its 10 m threshold — being above a floor and being
//! able to reach your own tavern are different things.
//!
//! The observable is the warning line itself. It is the whole feature: a host reads it and changes one number.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Place, PlaceTags, SocietyDefs, SocietySettings};

/// A tiny logger that keeps every `WARN` line, so the test can read what the crate told the host.
#[derive(Resource, Default)]
struct Captured(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

fn two_places(settings: SocietySettings, separation: f32) -> (App, Captured) {
    let captured = Captured::default();
    let sink = captured.0.clone();
    let subscriber = bevy::log::tracing_subscriber::fmt()
        .with_max_level(bevy::log::Level::WARN)
        .with_writer(move || Sink(sink.clone()))
        .finish();
    // A global subscriber can be set once per process; whichever test wins installs it, and the sink is
    // shared through `Captured`, so both tests read the same lines. `set_global_default` returning an error
    // means another test already installed one, which is fine.
    let _ = bevy::log::tracing::subscriber::set_global_default(subscriber);

    let mut app = common::headless(
        SocietyDefs::starter().unwrap_or_else(|_| {
            bevy_society::SocietyDefs::from_ron("(needs: [], activities: [])")
                .expect("minimal defs")
        }),
        settings,
    );
    let spawn_place = |name: &str, x: f32, app: &mut App| {
        app.world_mut()
            .spawn((
                Name::new(name.to_owned()),
                Place { capacity: 4 },
                PlaceTags(vec!["gathering".to_owned()]),
                Transform::from_xyz(x, 0.0, 0.0),
            ))
            .id()
    };
    let near = spawn_place("near", 0.0, &mut app);
    let far = spawn_place("far", separation, &mut app);
    for (i, home) in [near, far].into_iter().enumerate() {
        let person = common::spawn_person(&mut app, &format!("P{i}"), Vec2::ZERO);
        app.world_mut()
            .entity_mut(person)
            .insert((Home(home), Autonomous));
    }
    (app, captured)
}

struct Sink(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl std::io::Write for Sink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(text) = std::str::from_utf8(buf)
            && let Ok(mut lines) = self.0.lock()
        {
            lines.push(text.to_owned());
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn warnings(captured: &Captured) -> String {
    captured
        .0
        .lock()
        .map(|lines| lines.join(""))
        .unwrap_or_default()
}

#[test]
fn the_warning_fires_on_an_unreachable_layout_and_stays_quiet_on_a_reachable_one() {
    // **One test, not two.** `tracing` allows a single global subscriber per process, so a second test in this
    // binary would run with no sink attached and read an empty string — it would pass or fail for reasons that
    // have nothing to do with the code under test. Both cases therefore live here, in order.

    // 4 game-hours at 64 s/day buys ~14.9 m of walking, so 30 m apart cannot be reached.
    let mut settings: SocietySettings = common::settings();
    settings.seconds_per_day = 64.0;
    settings.tuning.travel_timeout_hours = Some(4.0);
    let (mut app, captured) = two_places(settings, 30.0);
    app.update();

    let text = warnings(&captured);
    assert!(
        text.contains("farthest pair of places"),
        "the crate must name the unreachable layout; it said: {text}"
    );
    assert!(
        text.contains("'near'") && text.contains("'far'"),
        "the warning must name the pair so a host knows which two places to move: {text}"
    );
    assert!(
        text.contains("30.0 m"),
        "the warning must state the distance it measured: {text}"
    );

    // And a layout the same timeout covers must stay quiet. A fresh app, the same sink.
    let mut settings: SocietySettings = common::settings();
    settings.seconds_per_day = 64.0;
    settings.tuning.travel_timeout_hours = Some(4.0);
    let (mut near_app, near_captured) = two_places(settings, 5.0);
    near_app.update();
    let near_text = warnings(&near_captured);
    assert!(
        !near_text.contains("farthest pair of places"),
        "a reachable layout must not be warned about: {near_text}"
    );
}

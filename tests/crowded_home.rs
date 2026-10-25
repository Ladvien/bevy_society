//! **A home that cannot seat its residents must be named.**
//!
//! Everyone goes to bed at the same hour, so every resident of a place aims at the same coordinate inside
//! `interaction_radius`. ORCA holds two people's centres one body diameter apart, so the disc packs solid and the
//! queue behind it times out. Measured on the project's own village: at every one of **150** interrupted sleeps
//! the goal already held a median of **6** bodies — exactly that home's declared `capacity` — and a 1.5 m disc
//! seats about `(1.5 / 0.7)² ≈ 4.6`, so six residents already do not fit.
//!
//! `warn_unreachable_places` counts residents per home and says so at `Startup`, naming the place, the residents,
//! the seats and the radius that would work. The seat count is a **lower bound** on the crowding: ORCA does not
//! pack around a point, it queues at the centre, so the warning says "about" rather than pretending to simulate.
//!
//! One test, not two: `tracing` allows a single global subscriber per process, and a second test in this binary
//! would read an empty capture and pass for the wrong reason.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Place, PlaceTags, SocietyDefs, SocietySettings};

#[derive(Resource, Default)]
struct Captured(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

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

fn village(settings: SocietySettings, residents: usize) -> (App, Captured) {
    let captured = Captured::default();
    let sink = captured.0.clone();
    let subscriber = bevy::log::tracing_subscriber::fmt()
        .with_max_level(bevy::log::Level::WARN)
        .with_writer(move || Sink(sink.clone()))
        .finish();
    let _ = bevy::log::tracing::subscriber::set_global_default(subscriber);
    let mut app = common::headless(
        SocietyDefs::from_ron("(needs: [], activities: [])").expect("empty defs parse"),
        settings,
    );
    let home = app
        .world_mut()
        .spawn((
            Name::new("dormitory"),
            Place { capacity: 32 },
            PlaceTags(vec!["rest".to_owned()]),
            Transform::default(),
        ))
        .id();
    for i in 0..residents {
        let person = common::spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32, 0.0));
        app.world_mut()
            .entity_mut(person)
            .insert((Home(home), Autonomous));
    }
    (app, captured)
}

#[test]
fn a_home_that_cannot_seat_its_residents_is_named() {
    let mut settings: SocietySettings = common::settings();
    settings.interaction_radius = 1.5;

    // Twelve residents into a disc that seats about four.
    let (mut app, captured) = village(settings, 12);
    app.update();
    let text = captured.0.lock().map(|l| l.join("")).unwrap_or_default();
    assert!(
        text.contains("is home to 12 people"),
        "the crate must name the crowded home; it said: {text}"
    );
    assert!(
        text.contains("'dormitory'"),
        "the warning must name the place: {text}"
    );
    assert!(
        text.contains("seats about 4"),
        "the warning must state how many the disc seats: {text}"
    );

    // And a home that fits must stay quiet.
    let mut settings: SocietySettings = common::settings();
    settings.interaction_radius = 1.5;
    let (mut small, small_captured) = village(settings, 2);
    small.update();
    let quiet = small_captured
        .0
        .lock()
        .map(|l| l.join(""))
        .unwrap_or_default();
    assert!(
        !quiet.contains("is home to"),
        "a home that seats its residents must not be warned about: {quiet}"
    );
}

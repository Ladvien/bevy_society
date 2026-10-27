//! **The crate can say that a good nothing makes can never be had.**
//!
//! A pack may name a good, consume it, and never produce it. Until `satisfies` started paying for what a
//! `consumes` actually found, that was invisible: every meal was paid in full from a shelf that could never
//! hold anything. Now it is a need nobody can meet, and the only thing that distinguishes it from a working
//! village is a store that is empty for the whole run.
//!
//! Measured on this crate's own starter pack (`assets/starter.society.ron`): it declares `bread` and has
//! `eat` consume it, and no activity in the pack produces one — so a host built from `DefsFile::starter()`
//! alone had a village that could never hold a loaf, silently, for as long as the crate has had goods.
//!
//! The observable is the warning line itself, as in `tests/unreachable_places.rs`: it is the whole feature.
mod common;

use bevy::prelude::*;
use bevy_society::SocietyDefs;

/// A logger that keeps every rendered `WARN` line, so the test can read what the crate told the host.
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

/// Builds an app over `defs` with a logger attached, and returns what the crate warned.
fn warnings_for(defs: SocietyDefs) -> String {
    let sink = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = sink.clone();
    let subscriber = bevy::log::tracing_subscriber::fmt()
        .with_max_level(bevy::log::Level::WARN)
        .with_writer(move || Sink(writer.clone()))
        .finish();
    // One global subscriber per process; whichever test wins installs it and the sink is shared, so both
    // cases below read the same lines.
    let _ = bevy::log::tracing::subscriber::set_global_default(subscriber);

    let _app = common::headless(defs, common::settings());
    sink.lock().map(|lines| lines.join("")).unwrap_or_default()
}

/// **A consumable nothing produces is named, and a pack that makes its own goods is not.**
///
/// One test, not two: `tracing` allows a single global subscriber per process, so a second test in this
/// binary would run with no sink attached and read an empty string — passing or failing for reasons that
/// have nothing to do with the code under test.
///
/// The first half used to test `DefsFile::starter()`, which was exactly the pack this defect shipped in:
/// it declared `bread`, had `eat` consume one, and had no producer at all. `bake` now lives in the starter
/// pack (`docs/loop/runs/2026-10-26-12/`), so the malformed pack is written out here instead — which is
/// what a check should be pinned against anyway, rather than against whatever the shipped data happens to
/// contain.
#[test]
fn a_good_nothing_produces_is_named_and_a_fed_pack_stays_quiet() {
    let unmade = SocietyDefs::from_ron(
        r#"(
            goods: [(id: "bread")],
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [
                (id: "eat", target: Home, duration_hours: 0.5,
                 consumes: [("bread", 0.25)],
                 considerations: [], satisfies: [("food", 1.2)], interaction: None, barks: []),
            ],
        )"#,
    )
    .expect("the definitions parse");
    let warned = warnings_for(unmade);
    assert!(
        warned.contains("no activity produces"),
        "the crate must name a good nothing makes; it said: {warned}"
    );
    assert!(
        warned.contains("bread"),
        "the warning must name the good so a host knows which one to give a producer: {warned}"
    );

    // A pack that produces what it eats says nothing.
    let fed = SocietyDefs::from_ron(
        r#"(
            goods: [(id: "bread")],
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [
                (id: "bake", target: Home, duration_hours: 1.0,
                 produces: [("bread", 3.0)],
                 considerations: [], satisfies: [], interaction: None, barks: []),
                (id: "eat", target: Home, duration_hours: 0.5,
                 consumes: [("bread", 0.25)],
                 considerations: [], satisfies: [("food", 1.2)], interaction: None, barks: []),
            ],
        )"#,
    )
    .expect("the definitions parse");
    let quiet = warnings_for(fed);
    assert!(
        !quiet.contains("no activity produces"),
        "a pack whose own activity makes the good must not be warned about: {quiet}"
    );
}

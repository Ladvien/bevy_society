//! **The cost harness.** Runs a settled society of a given size for a fixed number of ticks and
//! reports what one society tick costs.
//!
//! ```text
//! cargo run --release --example benchmark -- --people 1000 --ticks 200
//! ```
//!
//! Perception is brute force over every pair of persons (`docs/guide/concepts.md` states the design
//! and claims it is "fine" to about a thousand), so the number here is what a host is buying when
//! it spawns that many people. It is reported as **milliseconds per society tick**, not per update:
//! a host's frame budget is per frame, and a society tick is whatever fraction of a second
//! [`SocietySettings::tick_secs`] makes it.
//!
//! Timing is wall clock and therefore not deterministic, which is fine: this is a measurement
//! programme, not a simulation output. Nothing here is part of the crate.

mod common;

use bevy::app::PluginGroup;
use bevy::ecs::name::Name;
use bevy::prelude::{Entity, MinimalPlugins, Transform, TransformPlugin};
use bevy::time::TimeUpdateStrategy;
use bevy_society::{
    Autonomous, Home, Person, PersonIndex, Place, PlaceTags, SocietyClock, SocietyDefs,
    SocietyPlugins, SocietySettings,
};
use std::time::Instant;

fn main() {
    let args = Args::parse();
    let defs = match SocietyDefs::starter() {
        Ok(defs) => defs,
        Err(err) => {
            eprintln!("benchmark: the starter pack does not compile: {err}");
            std::process::exit(1);
        }
    };

    let mut app = bevy_app();
    app.insert_resource(SocietySettings {
        // One society tick per fixed step, so a measurement of N updates is N ticks.
        tick_secs: 1.0 / 64.0,
        seconds_per_day: 4.0,
        ..SocietySettings::default()
    })
    .insert_resource(defs);
    let plugins = SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();

    let places = spawn_places(&mut app, args.people);
    spawn_people(&mut app, args.people, &places);

    // Settle first: an unsettled society has everyone idle, and idle people skip the decision and
    // exchange systems that cost anything.
    let settle = args.settle;
    for _ in 0..settle {
        app.update();
    }
    let settled = app.world().resource::<SocietyClock>().tick;

    // The measurement. Each update past the first is exactly one society tick.
    let start = Instant::now();
    for _ in 0..args.ticks {
        app.update();
    }
    let elapsed = start.elapsed();
    let ticks = app.world().resource::<SocietyClock>().tick - settled;

    if ticks == 0 {
        eprintln!("benchmark: no ticks elapsed; the clock is not advancing");
        std::process::exit(1);
    }
    let per_tick_ms = elapsed.as_secs_f64() * 1000.0 / ticks as f64;
    let living = app.world().resource::<PersonIndex>().sorted.len();
    let pairs = (living as u64).saturating_mul(living.saturating_sub(1) as u64) / 2;

    println!(
        "{{\"people\": {living}, \"ticks\": {ticks}, \"ms_per_tick\": {per_tick_ms:.3}, \
         \"pairs_per_tick\": {pairs}, \"ms_per_million_pairs\": {:.1}}}",
        if pairs == 0 {
            0.0
        } else {
            per_tick_ms * 1_000_000.0 / pairs as f64
        }
    );
}

/// A minimal headless app with a pinned fixed step, matching the evidence harness.
fn bevy_app() -> bevy::app::App {
    let mut app = bevy::app::App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        bevy::log::LogPlugin::default(),
    ))
    // A headless app has no frame clock: without this `Time<Fixed>` never advances.
    .insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 64.0),
    ));
    app
}

/// Enough tagged places that a crowd has somewhere to go, scaled with the population.
fn spawn_places(app: &mut bevy::app::App, people: usize) -> Vec<Entity> {
    let count = (people / 20).max(4);
    let mut places = Vec::with_capacity(count);
    for i in 0..count {
        // A ring, so distances vary and perception has real work to do.
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        let radius = 10.0 + 2.0 * (i / 8) as f32;
        places.push(
            app.world_mut()
                .spawn((
                    Name::new(format!("place{i}")),
                    Place {
                        capacity: (people / count).max(1) as u32,
                    },
                    PlaceTags(vec![
                        "gathering".to_owned(),
                        "food".to_owned(),
                        "work".to_owned(),
                    ]),
                    Transform::from_xyz(radius * angle.cos(), 0.0, radius * angle.sin()),
                ))
                .id(),
        );
    }
    places
}

/// `people` persons in a disc, all autonomous, each with a home.
fn spawn_people(app: &mut bevy::app::App, people: usize, places: &[Entity]) {
    let home = places.first().copied();
    for i in 0..people {
        // A golden-angle spiral: even spread, no grid artefacts, and cheap to compute.
        let fraction = i as f32 / people.max(1) as f32;
        let radius = 40.0 * fraction.sqrt();
        let angle = i as f32 * 2.399_963;
        let mut entity = app.world_mut().spawn((
            Person,
            Autonomous,
            Name::new(format!("P{i}")),
            Transform::from_xyz(radius * angle.cos(), 0.0, radius * angle.sin()),
        ));
        if let Some(home) = home {
            entity.insert(Home(home));
        }
        #[cfg(feature = "steering")]
        entity.insert(bevy_society::SocialMover::default());
    }
}

/// `--people <n>`, `--ticks <n>`, `--settle <n>`.
struct Args {
    people: usize,
    ticks: usize,
    settle: usize,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            people: 1000,
            ticks: 200,
            settle: 400,
        };
        let mut argv = std::env::args().skip(1);
        while let Some(flag) = argv.next() {
            let mut value = || {
                argv.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                    eprintln!("benchmark: {flag} needs a number");
                    std::process::exit(1);
                })
            };
            match flag.as_str() {
                "--people" => args.people = value(),
                "--ticks" => args.ticks = value(),
                "--settle" => args.settle = value(),
                other => {
                    eprintln!("benchmark: unknown argument {other}");
                    std::process::exit(1);
                }
            }
        }
        args
    }
}

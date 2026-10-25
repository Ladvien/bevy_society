//! **The cost harness.** Runs a settled society of a given size for a fixed number of ticks and
//! reports what one society tick costs.
//!
//! ```text
//! cargo run --release --example benchmark -- --people 1000 --ticks 200
//! ```
//!
//! `docs/guide/concepts.md` records the design and the measured numbers. Briefly: perception runs through
//! a uniform grid, and the cost that scales is **scoring every neighbour against every `Person`-targeted
//! activity**, not the pair search — measured, an all-pairs scan was 36 ms of a 254 ms tick at 5 000
//! people, while the scoring is `people x activities x neighbours`. It is reported as **milliseconds per
//! society tick**, not per update:
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
    if std::env::var("BENCH_TRACE").is_ok() {
        app.insert_resource(bevy_society::DecisionTrace::with_capacity(64, 8));
    }
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

    // **Repetition, because a single window is not a measurement.** The same binary on the same machine
    // has produced 36.8, 46.2, 50.9, 51.1 and 54.7 ms/tick at 1 000 people depending on what else was
    // running. One window cannot distinguish the code from the load, so this reports the **median** and the
    // **spread** of `--reps` windows and lets a reader judge whether the machine was quiet enough to quote.
    let mut per_tick: Vec<f64> = Vec::with_capacity(args.reps);
    let mut ticks = 0u64;
    for _ in 0..args.reps {
        let before = app.world().resource::<SocietyClock>().tick;
        let start = Instant::now();
        for _ in 0..args.ticks {
            app.update();
        }
        let elapsed = start.elapsed();
        let ran = app.world().resource::<SocietyClock>().tick - before;
        ticks = ran;
        if ran > 0 {
            per_tick.push(elapsed.as_secs_f64() * 1000.0 / ran as f64);
        }
    }

    if ticks == 0 || per_tick.is_empty() {
        eprintln!("benchmark: no ticks elapsed; the clock is not advancing");
        std::process::exit(1);
    }
    per_tick.sort_by(f64::total_cmp);
    let median = per_tick[per_tick.len() / 2];
    let min = per_tick[0];
    let max = per_tick[per_tick.len() - 1];
    // Spread as a fraction of the median: the number that says whether this machine was quiet enough for
    // the figure to mean anything.
    let spread_pct = if median > 0.0 {
        (max - min) / median * 100.0
    } else {
        0.0
    };
    let living = app.world().resource::<PersonIndex>().sorted.len();
    let pairs = (living as u64).saturating_mul(living.saturating_sub(1) as u64) / 2;

    println!(
        "{{\"people\": {living}, \"ticks\": {ticks}, \"reps\": {}, \"ms_per_tick_median\": {median:.3}, \
         \"ms_per_tick_min\": {min:.3}, \"ms_per_tick_max\": {max:.3}, \"spread_pct\": {spread_pct:.1}, \
         \"pairs_per_tick\": {pairs}, \"ms_per_million_pairs\": {:.1}}}",
        per_tick.len(),
        if pairs == 0 {
            0.0
        } else {
            median * 1_000_000.0 / pairs as f64
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
    reps: usize,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            people: 1000,
            ticks: 200,
            settle: 400,
            reps: 5,
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
                "--reps" => args.reps = value().max(1),
                other => {
                    eprintln!("benchmark: unknown argument {other}");
                    std::process::exit(1);
                }
            }
        }
        args
    }
}

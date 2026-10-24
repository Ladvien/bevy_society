//! The smallest society that does something: eight villagers, one well, the starter pack.
//!
//! This is the README's quick start. It runs headlessly, so it needs no window, no camera and no
//! font — the society is the point, the balloons are a host's choice.
//!
//! ```text
//! cargo run --example quickstart
//! ```

use bevy::app::App;
use bevy::ecs::name::Name;
use bevy::ecs::resource::Resource;
use bevy::log::info;
use bevy::prelude::{MinimalPlugins, PluginGroup, Transform, TransformPlugin};
use bevy_society::{
    Autonomous, Home, InteractionResolved, Person, Place, PlaceTags, SocietyDefs, SocietyPlugins,
    SocietySettings,
};

/// What the example counted, for its one-line report.
#[derive(Resource, Default)]
struct Tally {
    exchanges: u32,
    accepted: u32,
}

fn main() {
    let Ok(defs) = SocietyDefs::starter() else {
        eprintln!("quickstart: the starter pack does not compile");
        std::process::exit(1);
    };

    // A short game-day. At the default 240 s a need decays too slowly for 600 ticks to make anyone
    // want to socialise, and the example would show a village that never speaks.
    let settings = SocietySettings {
        seconds_per_day: 16.0,
        ..SocietySettings::default()
    };

    let mut app = App::new();
    // `LogPlugin` so the run actually reports; `MinimalPlugins` alone has no logger.
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        bevy::log::LogPlugin::default(),
    ))
    // A headless app has no frame clock, so `Time<Fixed>` never advances on its own: pin one
    // fixed step per update. Without this the society takes zero ticks in 600 updates.
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 64.0),
    ))
    .insert_resource(settings)
    .insert_resource(defs)
    .insert_resource(Tally::default());
    // The dialogue plugin needs a speech-bubbles plugin over a camera, and this example has neither,
    // so it is disabled rather than faked.
    let plugins = SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);

    // Somewhere to gather, and where everyone calls home.
    let well = app
        .world_mut()
        .spawn((
            Name::new("well"),
            Place { capacity: 6 },
            PlaceTags(vec!["gathering".to_owned()]),
            Transform::default(),
        ))
        .id();

    for (i, name) in [
        "Bram", "Alys", "Corin", "Dena", "Edric", "Fenna", "Garet", "Hilda",
    ]
    .iter()
    .enumerate()
    {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        app.world_mut().spawn((
            Person,
            Autonomous,
            Name::new(*name),
            Home(well),
            Transform::from_xyz(3.0 * angle.cos(), 0.0, 3.0 * angle.sin()),
        ));
    }

    // 600 ticks at 1/64 s each is about nine game-days at 16 s per day. Messages are drained every
    // update: Bevy keeps one for two buffer swaps, and `InteractionResolved` is written in
    // `FixedUpdate`, so counting only at the end would miss nearly all of them.
    for _ in 0..600 {
        app.update();
        let resolved: Vec<InteractionResolved> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
            .drain()
            .collect();
        if !resolved.is_empty() {
            let mut tally = app.world_mut().resource_mut::<Tally>();
            for exchange in resolved {
                tally.exchanges += 1;
                if exchange.accepted {
                    tally.accepted += 1;
                }
            }
        }
    }
    let tally = app.world().resource::<Tally>();
    info!(
        "quickstart: {} social exchanges, {} accepted",
        tally.exchanges, tally.accepted
    );
}

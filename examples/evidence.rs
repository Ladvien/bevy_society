//! **The evidence harness.** Runs one scenario headlessly and writes what happened, so an iteration
//! can look at the society instead of arguing about it.
//!
//! ```text
//! cargo run --example evidence -- --scenario village-evening --out docs/loop/runs/<run>/evidence
//! ```
//!
//! Three artefacts per run:
//!
//! - `narrative.txt` — one person's day in readable lines: when they chose what, whom they talked to,
//!   what it did to their mood and needs. Read it as a stranger would.
//! - `series.csv` — one row per tick: the aggregates a plot needs (mean need levels, mood, opinion
//!   variance, rumor reach, friendship and rivalry counts). `tools/plots.py` turns it into PNGs.
//! - `summary.json` — the end state, for a test or a log entry to assert against.
//!
//! The scenarios themselves are declared in `docs/loop/scenarios.yaml`; this file implements the ones
//! it can build from the crate's own definitions and prints a clear error for the rest.

mod common;

use bevy::app::{App, PluginGroup};
use bevy::ecs::name::Name;
use bevy::ecs::world::World as EcsWorld;
use bevy::prelude::{Entity, MinimalPlugins, Transform, TransformPlugin, Vec2};
use bevy::time::TimeUpdateStrategy;
use bevy_society::{
    ActivityEnded, ActivityStarted, Autonomous, Doing, Home, InteractionResolved, Knowledge, Mood,
    Needs, Opinions, Person, Place, PlaceTags, PlantRumor, RumorBook, SocietyClock, SocietyDefs,
    SocietyPlugins, SocietySettings, TieLabel, TieLabelChanged, Ties,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let args = Args::parse();
    let defs = match society_defs(&args.scenario) {
        Ok(defs) => defs,
        Err(err) => {
            eprintln!("evidence: {err}");
            std::process::exit(1);
        }
    };
    let mut settings = settings_for(&args.scenario);
    if let Some(spd) = args.seconds_per_day {
        settings.seconds_per_day = spd;
    }
    if let Some(offset) = args.positivity_offset {
        settings.tuning.positivity_offset = offset;
    }
    if let Some(bar) = args.friend_affinity {
        settings.tuning.friend_affinity = bar;
    }
    if let Some(band) = args.label_hysteresis {
        settings.tuning.label_hysteresis = band;
    }
    if let Some(band) = args.friend_band {
        settings.tuning.friend_band = band;
    }
    if let Some(seed) = args.seed {
        settings.seed = seed;
    }

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        bevy::log::LogPlugin::default(),
    ))
    // A headless app has no frame clock; pin one fixed step per update.
    .insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 64.0),
    ))
    .insert_resource(settings)
    .insert_resource(bevy_society::DecisionTrace::with_capacity(96, 6))
    .insert_resource(defs);
    let plugins = SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();

    let world = build(&mut app, &args.scenario, args.people);
    let Some(world) = world else {
        eprintln!("evidence: unknown scenario '{}'", args.scenario);
        std::process::exit(1);
    };

    // `--days` overrides the scenario's own `ticks:` with `days · seconds_per_day / tick_secs`, so a
    // thirty-day run is one flag rather than an arithmetic exercise.
    let day_secs = args
        .seconds_per_day
        .unwrap_or_else(|| app.world().resource::<SocietySettings>().seconds_per_day);
    let tick_secs = app
        .world()
        .resource::<SocietySettings>()
        .tick_secs
        .max(1.0e-6);
    let ticks = match args.days {
        Some(days) => (days * day_secs / tick_secs).round().max(1.0) as usize,
        None => ticks_for(&args.scenario, args.seconds_per_day),
    };
    let tracked = world.villagers.first().copied();
    let mut rows = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut planted: Option<bevy_society::RumorId> = None;
    // The highest reach and knower count the planted rumor ever had. A rumor is forgotten as well as
    // spread — its knowers evict it once it ages past `knowledge_capacity` newer ones — so the final
    // state is not the answer to "did it spread".
    let mut completed_endings = 0usize;
    let mut interrupted_endings = 0usize;
    // Which activity, not just how many. A total tells you there is a problem; the breakdown tells
    // you whose it is, and a fix aimed at the wrong activity is a fix that does nothing.
    let mut failed_by_activity: BTreeMap<String, usize> = BTreeMap::new();
    // **Does the village ever make up?** A rival count that only ever rises says hostility accumulates; a
    // count that rises *and falls* says the society reconciles. `TieLabelChanged` fires on every transition,
    // so counting the ones that *leave* `Rival` is the direct measurement of recovery, and nothing was
    // recording it. Measured on `village-evening`: rivals 0 -> 42 over 17 game-days while friends stayed 13-18.
    let mut accepted_compliments = 0usize;
    let mut became_rival = 0usize;
    // **Every** label transition, not only the rival ones. A host that shows "you are now friends"
    // or unlocks a conversation on `TieLabelChanged` cares about how many arrive and whether they mean
    // anything: measured before hysteresis, 58% of them came from a value resting within 0.01 of the bar.
    let mut label_changes = 0usize;
    let mut left_rival = 0usize;
    let mut peak_reach = 0.0_f32;
    let mut peak_known_by = 0_usize;
    // Every accepted gossip exchange per person, accumulated so the plant can choose a teller who has
    // actually told something.
    let mut gossipers: Vec<(Entity, u32)> = Vec::new();
    let mut seen_exchanges: Vec<(Entity, u32)> = Vec::new();
    let _ = &mut seen_exchanges;
    // Hoisted: `plant_at` parses `scenarios.yaml`, and calling it inside the loop parsed the whole file
    // once per tick — 57 000 times in a 67 200-tick run, each printing a warning for every `settings:`
    // key the harness does not read.
    let plant_tick = plant_at(&args.scenario);
    for tick in 0..ticks {
        app.update();
        if tick == plant_tick {
            planted = plant(&mut app, &world, &gossipers);
        }
        {
            let mut exchanges = app
                .world_mut()
                .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>();
            for resolved in exchanges.drain() {
                // **Both sides of the affinity ledger.** `compliment` is the only activity that raises
                // affinity between two people; a witnessed deed lowers it for every witness. Without
                // counting both, a claim about which side outweighs the other cannot be checked — and an
                // earlier iteration's "bad outweighs good 4.8:1" was arithmetic on the wrong variable.
                if resolved.interaction == "compliment" && resolved.accepted {
                    accepted_compliments += 1;
                }
                if resolved.interaction != "gossip" {
                    continue;
                }
                match gossipers.iter_mut().find(|(p, _)| *p == resolved.actor) {
                    Some((_, count)) => *count += 1,
                    None => gossipers.push((resolved.actor, 1)),
                }
            }
        }
        // **Tick 0 is not a sample of the society; it is before one exists.** `size_person_state` fills
        // `Needs` and `Opinions` on the first tick, so a row taken before the first update reads
        // `Needs([])` and averages to **0.000 for every need**. Two iterations of "all four needs cycle"
        // rested on that one row: the claim was true of the mean only because of a startup artefact, and
        // false per person — measured, `social` never falls below 0.39 and `rest` never below 0.50.
        //
        // The first row is therefore written with no needs at all, and `needs` is empty in the CSV for
        // it. A reader, and any check, then skips it because there is nothing to average.
        // **Whole-village ending counts.** The narrative tracks one person, so quoting its
        // interrupted/completed split as if it were the village's was a 24x sampling error. These
        // counters drain every `ActivityEnded` for everybody, which is what a steering change has to
        // be judged on.
        {
            let mut ended = app
                .world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>();
            for message in ended.drain() {
                if message.completed {
                    completed_endings += 1;
                } else {
                    interrupted_endings += 1;
                    *failed_by_activity
                        .entry(message.activity.clone())
                        .or_insert(0) += 1;
                }
            }
        }
        {
            let mut changed = app
                .world_mut()
                .resource_mut::<bevy::ecs::message::Messages<TieLabelChanged>>();
            for message in changed.drain() {
                label_changes += 1;
                if message.new == TieLabel::Rival {
                    became_rival += 1;
                } else if message.old == TieLabel::Rival {
                    left_rival += 1;
                }
            }
        }
        let row = snapshot_row(&app, tick as f64, &world, planted);
        // The first row is the pre-society state: `size_person_state` has not run, so `Needs` is empty
        // and every need reads 0.000. Recording it made "all four needs cross below 0.35" true of the
        // mean on the strength of one row at tick 0, while no individual ever came close.
        if tick > 0 {
            peak_reach = peak_reach.max(row.reach);
            peak_known_by = peak_known_by.max(row.known_by);
            rows.push(row);
        }
        drain_narrative(&mut app, &world, tracked, &mut lines);
    }

    let out = PathBuf::from(&args.out);
    if let Err(err) = std::fs::create_dir_all(&out) {
        eprintln!("evidence: cannot create {}: {err}", out.display());
        std::process::exit(1);
    }
    write_narrative(&mut app, tracked, &lines, &out);
    write_why(&mut app, tracked, &out);
    series(&rows, &out);
    summary(
        app.world(),
        &world,
        &out,
        planted,
        peak_reach,
        peak_known_by,
        Endings {
            counts: (completed_endings, interrupted_endings),
            failed_by_activity: &failed_by_activity,
            rival_transitions: (became_rival, left_rival),
            accepted_compliments,
            label_changes,
        },
    );
    println!(
        "evidence: {} ticks, {} people, {} places, wrote {}",
        ticks,
        world.villagers.len(),
        world.places.len(),
        out.display()
    );
}

// -------------------------------------------------------------------------------------------------
// Scenarios
// -------------------------------------------------------------------------------------------------

/// The entities a scenario spawned.
struct World {
    places: Vec<Entity>,
    villagers: Vec<Entity>,
    /// The rumor planted at `plant_at`, if the scenario plants one.
    subject: Option<Entity>,
    origin: Option<Entity>,
}

/// One row of the time series.
struct Row {
    tick: f64,
    needs: Vec<f32>,
    pleasure: f32,
    arousal: f32,
    dominance: f32,
    opinion_variance: f32,
    reach: f32,
    /// How many villagers hold the planted rumor at this tick.
    known_by: usize,
    friends: u32,
    rivals: u32,
    doing: BTreeMap<String, u32>,
}

/// The starter pack, plus whatever the village scenario adds.
fn society_defs(scenario: &str) -> Result<SocietyDefs, String> {
    let mut file = bevy_society::DefsFile::starter().map_err(|e| e.to_string())?;
    if scenario.starts_with("village") {
        let additions =
            bevy_society::DefsFile::from_ron(include_str!("assets/village.society.ron"))
                .map_err(|e| e.to_string())?;
        file.merge(additions).map_err(|e| e.to_string())?;
    }
    SocietyDefs::compile(file).map_err(|e| e.to_string())
}

/// One scenario's declared settings, read from `docs/loop/scenarios.yaml`.
#[derive(Debug, Clone, Copy)]
struct Spec {
    seconds_per_day: f32,
    start_hour: f32,
    travel_timeout_hours: Option<f32>,
    /// The scenario's `ticks:`, counted at **its own** `seconds_per_day`.
    ticks: usize,
    /// The scenario's `seed:`. Unread until iteration 2026-11-17: every scenario ran at the crate
    /// default, so the file's declared seeds were decoration and two scenarios that differed only in
    /// seed were in fact the same run.
    seed: Option<u64>,
    /// The scenario's `contact_radius:`. **Unread until 2026-11-20**: `rumor-crosses-a-cluster` declares
    /// 4.0 and the harness used the crate default of 4.0 anyway, so the value was right by luck; any
    /// other number would have been silently ignored.
    contact_radius: Option<f32>,
}

impl Default for Spec {
    fn default() -> Self {
        let d = SocietySettings::default();
        Self {
            seconds_per_day: d.seconds_per_day,
            start_hour: d.start_hour,
            travel_timeout_hours: d.tuning.travel_timeout_hours,
            ticks: 800,
            seed: None,
            contact_radius: None,
        }
    }
}

/// Reads `docs/loop/scenarios.yaml` and returns the settings declared for `name`.
///
/// **The file is the spec, and this reads it.** Until iteration 2026-11-16 the harness hardcoded its
/// own values, and for `village-evening` it used `seconds_per_day = 4.0` where the file says **64.0** —
/// a day sixteen times shorter. Every "needs starve" and "the rumor never spreads" measurement in the
/// eight iterations before that was taken at the wrong day length, where a four-hour activity lasts
/// four ticks and a walk to the next place takes three hundred.
///
/// A scenario with no `settings:` line, or a name the file does not mention, gets the crate defaults:
/// a missing spec should reproduce the crate's own behaviour, not invent one.
fn scenario_spec(name: &str) -> Spec {
    const YAML: &str = include_str!("../docs/loop/scenarios.yaml");
    let mut spec = Spec::default();
    let mut in_scenario = false;
    for line in YAML.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- name:") {
            in_scenario = rest.trim() == name;
            continue;
        }
        if !in_scenario {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("seed:") {
            spec.seed = rest.trim().parse::<u64>().ok();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("ticks:") {
            if let Ok(number) = rest.trim().parse::<usize>() {
                spec.ticks = number;
            }
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("settings:") else {
            continue;
        };
        let body = rest.trim().trim_start_matches('{').trim_end_matches('}');
        for pair in body.split(',') {
            let Some((key, value)) = pair.split_once(':') else {
                continue;
            };
            let Ok(number) = value.trim().parse::<f32>() else {
                continue;
            };
            match key.trim() {
                "seconds_per_day" => spec.seconds_per_day = number,
                "start_hour" => spec.start_hour = number,
                "travel_timeout_hours" => spec.travel_timeout_hours = Some(number),
                "contact_radius" => spec.contact_radius = Some(number),
                other => {
                    // Once per parse, and `scenario_spec` is called a handful of times per run.
                    eprintln!(
                        "evidence: scenarios.yaml sets '{other}', which this harness ignores"
                    );
                }
            }
        }
    }
    spec
}

/// The scenario's own settings, at the tick length the harness pins.
///
/// `tick_secs` stays `1/64` for every scenario so its tick count is its number of updates whatever its
/// day length; the two are independent, and conflating them is what let the harness drift from the spec
/// unnoticed.
fn settings_for(scenario: &str) -> SocietySettings {
    let spec = scenario_spec(scenario);
    SocietySettings {
        seed: spec.seed.unwrap_or(SocietySettings::default().seed),
        seconds_per_day: spec.seconds_per_day,
        start_hour: spec.start_hour,
        contact_radius: spec
            .contact_radius
            .unwrap_or(SocietySettings::default().contact_radius),
        tick_secs: 1.0 / 64.0,
        tuning: bevy_society::Tuning {
            travel_timeout_hours: spec.travel_timeout_hours,
            ..Default::default()
        },
        ..SocietySettings::default()
    }
}

fn ticks_for(scenario: &str, day_override: Option<f32>) -> usize {
    let spec = scenario_spec(scenario);
    let day = day_override.unwrap_or(spec.seconds_per_day);
    // The file's `ticks:` is counted at its own `seconds_per_day`; converting keeps the scenario
    // spanning the same number of **game-days** whatever day length it declares, which is what makes
    // its observable meaningful.
    let harness_day = 4.0_f64;
    let scale = f64::from(day) / harness_day;
    ((spec.ticks as f64) * scale).round() as usize
}

/// The tick a scenario plants its rumor, converted from the tick count it was authored at.
///
/// These were written against the harness's old four-second day. At a scenario's own day length the
/// same *tick* is a far shorter span of game-time — 560 ticks is 3.3 game-hours at 64 s/day — and the
/// village has not yet gossiped, so the rumor is planted among people with nothing to say and never
/// spreads. Measured: reach 0.000 over 17 game-days, against 0.375 when planted after the village had
/// a history.
///
/// **So this is not the tick `scenarios.yaml` names.** The file says "a rumor planted at tick 300"; this
/// function returns **8 960** for `village-evening` at 64 s/day. The same conversion is applied to the
/// `ticks:` count above, so both mean "the same point in game-days", but the two numbers in the
/// documentation and in the run disagree by a factor of thirty. Measured, the observable holds at **7 of 7
/// seeds when planted at 300** (reaching 2-8 villagers) and at 8 of 10 at 8 960 — a tick that late is
/// past the village's busiest period, so a single planted rumor competes with more of the ~39 a day it
/// creates. `tests/village_evening.rs` asserts the condition at 300; `scenarios.yaml` now states both ticks.
fn plant_at(scenario: &str) -> usize {
    let authored = match scenario {
        "village-evening" | "village" => 560,
        "rumor-crosses-a-cluster" => 200,
        _ => return usize::MAX,
    };
    let harness_day = 4.0_f64;
    let scale = f64::from(scenario_spec(scenario).seconds_per_day) / harness_day;
    (authored as f64 * scale).round() as usize
}

/// Builds a scenario. `None` when the name is not one this harness knows.
fn build(app: &mut App, scenario: &str, people: Option<usize>) -> Option<World> {
    let some = people.unwrap_or(8);
    match scenario {
        "need-cycle" => Some(one_place_with(app, true, some)),
        "village-evening" | "village" => Some(village_with(app, people.unwrap_or(24))),
        "rumor-crosses-a-cluster" => Some(cluster(app)),
        "deed-draws-a-response" => Some(one_place_with(app, false, some)),
        _ => None,
    }
}

/// Eight people round a well, all autonomous: the simplest society that can show a need cycle.
/// `count` villagers round one place, so `--people` can change the population without changing the
/// scenario's shape.
fn one_place_with(app: &mut App, scatter: bool, count: usize) -> World {
    let well = app
        .world_mut()
        .spawn((
            Name::new("well"),
            Place { capacity: 8 },
            PlaceTags(vec![
                "gathering".to_owned(),
                "food".to_owned(),
                "work".to_owned(),
            ]),
            Transform::default(),
        ))
        .id();
    let mut villagers = Vec::new();
    let count = count.max(1);
    for i in 0..count {
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        let radius = if scatter { 2.0 } else { 0.5 };
        let entity = app
            .world_mut()
            .spawn((
                Person,
                Autonomous,
                Name::new(format!("V{i}")),
                Home(well),
                Transform::from_xyz(radius * angle.cos(), 0.0, radius * angle.sin()),
            ))
            .id();
        #[cfg(feature = "steering")]
        app.world_mut()
            .entity_mut(entity)
            .insert(bevy_society::SocialMover::default());
        villagers.push(entity);
    }
    World {
        places: vec![well],
        villagers,
        subject: None,
        origin: None,
    }
}

/// The F1 village: four places, 24 villagers in three factions of eight.
fn village_with(app: &mut App, people: usize) -> World {
    let places: Vec<Entity> = [
        ("well", Vec2::new(0.0, 0.0), vec!["gathering"], 6u32),
        (
            "tavern",
            Vec2::new(6.5, -5.0),
            vec!["gathering", "food"],
            10,
        ),
        ("market", Vec2::new(-6.5, -5.0), vec!["food", "work"], 8),
        ("workshop", Vec2::new(6.5, 5.0), vec!["work"], 6),
    ]
    .iter()
    .map(|(name, pos, tags, capacity)| {
        app.world_mut()
            .spawn((
                Name::new(name.to_owned()),
                Place {
                    capacity: *capacity,
                },
                PlaceTags(tags.iter().map(|t| (*t).to_string()).collect()),
                Transform::from_xyz(pos.x, 0.0, pos.y),
            ))
            .id()
    })
    .collect();
    // Each faction's people live at **their own** place. Every villager used to share `places[0]`,
    // while starting at their faction's place: so every sleep at Home was a cross-village walk that,
    // at 64 s/day, could not be finished inside the 4-hour travel timeout. Measured on the resulting
    // narrative: **25 interrupted sleeps against 4 completed**, each cut exactly four hours in and
    // restarted. A home a person cannot reach is not a home.
    let home_of = |column: usize| places[column % places.len()];

    let defs = app.world().resource::<SocietyDefs>().clone();
    let mut villagers = Vec::new();
    // The 24 names the village was authored with; `--people` beyond 24 gets `V25`, `V26`, ... so a
    // fresh-eyes run with a bigger population still works.
    const NAMES: [&str; 24] = [
        "Bram", "Alys", "Corin", "Dena", "Edric", "Fenna", "Garet", "Hilda", "Ivo", "Jena", "Kell",
        "Lira", "Milo", "Nera", "Oswin", "Perrin", "Quill", "Rhea", "Sable", "Tomas", "Ulric",
        "Vesna", "Wren", "Ysolde",
    ];
    let count = people.max(1);
    for i in 0..count {
        let owned;
        let name: &str = match NAMES.get(i) {
            Some(name) => name,
            None => {
                owned = format!("V{}", i + 1);
                &owned
            }
        };
        let column = i / 8;
        let row = i % 8;
        // Three factions of eight; a population past 24 wraps round them rather than panicking.
        let faction = ["millers", "weavers", "watch"][column % 3];
        let honesty = if faction == "watch" {
            0.7
        } else {
            -0.2 + 0.8 * ((i as f32 * 0.618).fract())
        };
        let memberships = defs.memberships([faction]).unwrap_or_default();
        let opinions = defs
            .opinions([
                ("honesty", honesty),
                ("kindness", -0.2 + 0.8 * (i as f32 * 0.37).fract()),
            ])
            .unwrap_or_default();
        let entity = app
            .world_mut()
            .spawn((
                Person,
                Autonomous,
                Name::new(name.to_owned()),
                Home(home_of(column)),
                // 2.5 m spacing: a walk to a neighbour is ~110 ticks, not 640. The village is small
                // enough that its people can actually reach each other inside a run.
                // Start within `interaction_radius` of the faction's place: a walk is real-time
                // (`speed / 64` m per tick) while a duration is game-time, so a grid walk of 2.5 m
                // (160 ticks) outlasts a 0.3 game-hour chat (3 ticks) and nobody ever arrives.
                Transform::from_xyz(
                    PLACE_POSITIONS[column % PLACE_POSITIONS.len()].x + 0.6 * (row as f32).cos(),
                    0.0,
                    PLACE_POSITIONS[column % PLACE_POSITIONS.len()].y + 0.6 * (row as f32).sin(),
                ),
                memberships,
                opinions,
            ))
            .id();
        #[cfg(feature = "steering")]
        app.world_mut()
            .entity_mut(entity)
            .insert(bevy_society::SocialMover::default());
        villagers.push(entity);
    }
    // The origin must be someone who will actually gossip: `gossip` is gated on `ctx.tellable`, and
    // a villager with nothing to say scores zero and chats instead. Measured: an arbitrary villager
    // gossiped twice in 3 600 ticks and the planted rumor never left it (reach 0.042 = 1/24), while
    // 63 gossip exchanges happened around it. The tavern's crowd is the natural teller.
    let subject = villagers.first().copied();
    let origin = villagers.get(8).copied();
    World {
        places,
        villagers,
        subject,
        origin,
    }
}

/// Where the village's four places stand, in `PLACES` order, mirrored from
/// `examples/common/scene.rs` so the harness and the capture frame the same village.
const PLACE_POSITIONS: [Vec2; 4] = [
    common::scene::PLACES[0].1,
    common::scene::PLACES[1].1,
    common::scene::PLACES[2].1,
    common::scene::PLACES[3].1,
];

/// Two groups far apart, so a rumor has to cross a gap.
fn cluster(app: &mut App) -> World {
    // Each group gets a hearth. Without one the only activities available are Person targets, so
    // the crowd gossips and snubs with nothing else to do: measured, `chat` never once ran in 2000
    // ticks because gossip monopolised the social need, and affinity ran away to -0.96.
    let mut places = Vec::new();
    for (i, centre) in [-20.0_f32, 20.0].iter().enumerate() {
        places.push(
            app.world_mut()
                .spawn((
                    Name::new(format!("hearth{i}")),
                    Place { capacity: 6 },
                    PlaceTags(vec![
                        "gathering".to_owned(),
                        "food".to_owned(),
                        "work".to_owned(),
                    ]),
                    Transform::from_xyz(*centre, 0.0, 0.0),
                ))
                .id(),
        );
    }
    let mut villagers = Vec::new();
    for i in 0..12 {
        let group = i / 6;
        let angle = (i % 6) as f32 * std::f32::consts::TAU / 6.0;
        let centre = if group == 0 { -20.0 } else { 20.0 };
        let entity = app
            .world_mut()
            .spawn((
                Person,
                Autonomous,
                Name::new(format!("G{group}{}", i % 6)),
                Transform::from_xyz(centre + 2.0 * angle.cos(), 0.0, 2.0 * angle.sin()),
            ))
            .id();
        #[cfg(feature = "steering")]
        app.world_mut()
            .entity_mut(entity)
            .insert(bevy_society::SocialMover::default());
        villagers.push(entity);
    }
    let subject = villagers.first().copied();
    let origin = villagers.get(6).copied();
    World {
        places,
        villagers,
        subject,
        origin,
    }
}

/// Plants the scenario's rumor, if it has one, and returns the id it got.
///
/// The id is taken from the book's length **before** the message is applied, because
/// `plant_rumor` appends. The column in `series.csv` used to be `RumorId(0)` — the first rumor the run
/// ever made, which for the village is a deed from tick 0 and has nothing to do with the one this
/// scenario plants. Every "rumor reach is a step function" reading in earlier runs was that mistake.
fn plant(
    app: &mut App,
    world: &World,
    gossipers: &[(Entity, u32)],
) -> Option<bevy_society::RumorId> {
    // The origin must be someone who will actually gossip, and a fixed index is not that. Measured
    // twice: at 4 s/day an arbitrary villager gossiped **twice in 3 600 ticks** and the planted rumor
    // never left it (reach 0.042); at the scenario's own 64 s/day the same index reached 0.000 over 17
    // game-days. `gossip` is gated on `ctx.tellable` and an `Affinity` curve, and which villager
    // satisfies both is a property of the run, not of the index.
    //
    // So the plant picks the villager who has gossiped most so far, read from the run itself. The
    // scenario then tests *whether gossip spreads a rumor* rather than whether one particular stranger
    // happens to be talkative.
    let origin = gossipers
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(person, _)| *person)
        .or(world.origin)?;
    let subject = world.subject?;
    // Take the id from the book **after** the message applies, not from its length before.
    //
    // The old version read `rumors.len()` and assumed the plant would land at that index. It does not
    // when a deed creates a rumor in the same update: measured, the harness tracked `RumorId(29)`, whose
    // `known_by` was 2, while none of the 24 villagers held it — the planted rumor was one index further
    // along, and the id it read belonged to a stranger's deed.
    //
    // The planted rumor is the last one the book gained whose origin is the person who asked for it, and
    // `plant_rumor` appends, so the search walks backwards and stops at the first match.
    app.world_mut().write_message(PlantRumor {
        origin,
        subject,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
    // The message is drained on the next fixed step, so this runs the app once.
    app.update();
    let book = app.world().resource::<RumorBook>();
    book.rumors
        .iter()
        .rposition(|rumor| rumor.origin == origin && rumor.subject == subject)
        .map(|index| bevy_society::RumorId(index as u32))
}

// -------------------------------------------------------------------------------------------------
// Collecting
// -------------------------------------------------------------------------------------------------

/// The aggregates one tick contributes, read straight from the world.
fn snapshot_row(
    app: &App,
    tick: f64,
    world: &World,
    planted: Option<bevy_society::RumorId>,
) -> Row {
    let world_ref = app.world();
    let living = world.villagers.len().max(1) as f32;
    let (mut needs_sum, mut need_count) = (vec![0.0_f32; 4], 0usize);
    let (mut p, mut a, mut d) = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut opinions = Vec::new();
    let (mut friends, mut rivals) = (0u32, 0u32);
    let mut doing: BTreeMap<String, u32> = BTreeMap::new();
    let mut n = 0.0_f32;

    for &entity in &world.villagers {
        if let Some(mood) = world_ref.get::<Mood>(entity) {
            p += mood.pleasure;
            a += mood.arousal;
            d += mood.dominance;
            n += 1.0;
        }
        if let Some(needs) = world_ref.get::<Needs>(entity) {
            for (slot, level) in needs.0.iter().enumerate() {
                if let Some(sum) = needs_sum.get_mut(slot) {
                    *sum += level;
                }
            }
            // Only count people whose needs have been sized. Before `size_person_state` runs, the
            // component exists but is empty, and counting it averages a real 1.0 down to 0.000.
            if !needs.0.is_empty() {
                need_count += 1;
            }
        }
        if let Some(opinion) = world_ref.get::<Opinions>(entity) {
            opinions.push(opinion.value(0));
        }
        if let Some(ties) = world_ref.get::<Ties>(entity) {
            for tie in &ties.0 {
                match tie.label {
                    TieLabel::Friend => friends += 1,
                    TieLabel::Rival => rivals += 1,
                    _ => {}
                }
            }
        }
        if let Some(activity) = world_ref.get::<Doing>(entity) {
            // **A person still travelling is not doing the activity.** `Doing::label` owns this convention so
            // a host does not have to rediscover it; tallying `activity` raw reports a village asleep 36% of
            // the time against the 33% one 8-hour sleep a day allows, and the gap is people walking to bed.
            let name = activity.label(world_ref.resource::<SocietyDefs>());
            *doing.entry(name).or_insert(0) += 1;
        }
    }
    let n = n.max(1.0);
    let variance = if opinions.len() > 1 {
        let mean = opinions.iter().sum::<f32>() / opinions.len() as f32;
        opinions.iter().map(|o| (o - mean).powi(2)).sum::<f32>() / opinions.len() as f32
    } else {
        0.0
    };
    // The planted rumor, not `RumorId(0)`: the first rumor a run creates is unrelated to the one
    // the scenario plants, and following it reported a flat line for three iterations.
    let book = world_ref.resource::<RumorBook>();
    // Zero before the plant: splicing another rumor's reach in front of the planted one drew a line
    // that fell to zero at the plant tick, which is not a thing that happened.
    let reach = planted.map_or(0.0, |id| book.reach(id, living as usize));
    let known_by = planted.map_or(0, |id| {
        world
            .villagers
            .iter()
            .filter(|&&e| {
                world_ref
                    .get::<Knowledge>(e)
                    .is_some_and(|k| k.get(id).is_some())
            })
            .count()
    });
    Row {
        tick,
        needs: if need_count == 0 {
            Vec::new()
        } else {
            needs_sum.iter().map(|s| s / need_count as f32).collect()
        },
        pleasure: p / n,
        arousal: a / n,
        dominance: d / n,
        opinion_variance: variance,
        reach,
        known_by,
        friends: friends / 2,
        rivals: rivals / 2,
        doing,
    }
}

/// Reads the messages a tracked person produced this tick into `lines`.
fn drain_narrative(app: &mut App, world: &World, tracked: Option<Entity>, lines: &mut Vec<String>) {
    let Some(tracked) = tracked else {
        return;
    };
    let clock = *app.world().resource::<SocietyClock>();
    {
        let mut started = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>();
        for message in started.drain() {
            if message.person == tracked {
                lines.push(format!(
                    "{:>5}: started {}",
                    clock_label(&clock),
                    message.activity
                ));
            }
        }
        let mut ended = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>();
        for message in ended.drain().filter(|m| m.person == tracked) {
            {
                lines.push(format!(
                    "{:>5}: finished {} ({})",
                    clock_label(&clock),
                    message.activity,
                    if message.completed {
                        "completed"
                    } else {
                        "interrupted"
                    }
                ));
            }
        }
        let resolved_messages: Vec<InteractionResolved> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
            .drain()
            .collect();
        for message in resolved_messages {
            if message.actor == tracked || message.target == tracked {
                let other = if message.actor == tracked {
                    message.target
                } else {
                    message.actor
                };
                let other_name = app
                    .world()
                    .get::<Name>(other)
                    .map_or_else(|| "someone".to_owned(), |n| n.as_str().to_owned());
                let role = if message.actor == tracked {
                    "with"
                } else {
                    "approached by"
                };
                lines.push(format!(
                    "{:>5}: {} {} ({}{})",
                    clock_label(&clock),
                    message.interaction,
                    format_args!("{role} {other_name}"),
                    if message.accepted {
                        "accepted"
                    } else {
                        "refused"
                    },
                    if message.refused_by_rule {
                        ", by rule"
                    } else {
                        ""
                    }
                ));
            }
        }
    }
    let _ = world;
}

/// Writes `why.txt`: the tracked person's last decision, with every option it weighed.
fn write_why(app: &mut App, tracked: Option<Entity>, out: &Path) {
    let Some(tracked) = tracked else {
        return;
    };
    let Some(trace) = app.world().get_resource::<bevy_society::DecisionTrace>() else {
        return;
    };
    let Some(decision) = trace.last_for(tracked) else {
        return;
    };
    let mut text = format!("# {}\n\n", decision.describe(&[]));
    for option in &decision.options {
        let _ = writeln!(
            text,
            "{:>6.3}  {}{}",
            option.score,
            option.activity,
            if option.chosen { "   <- chosen" } else { "" }
        );
    }
    let path = out.join("why.txt");
    if let Err(err) = std::fs::write(&path, text) {
        eprintln!("evidence: cannot write {}: {err}", path.display());
    }
}

/// Writes the tracked person's day as readable prose.
fn write_narrative(app: &mut App, tracked: Option<Entity>, lines: &[String], out: &Path) {
    let Some(tracked) = tracked else {
        return;
    };
    let name = app
        .world()
        .get::<Name>(tracked)
        .map_or_else(|| "someone".to_owned(), |n| n.as_str().to_owned());
    let needs = app.world().get::<Needs>(tracked).map(|n| n.0.clone());
    let mood = app.world().get::<Mood>(tracked).copied();
    let doing = app
        .world()
        .get::<Doing>(tracked)
        .map(|d| (d.activity, d.target));
    let doing = doing.map(|(activity, target)| {
        let name = app
            .world()
            .resource::<SocietyDefs>()
            .activity_name(activity)
            .to_owned();
        (name, target)
    });

    let mut text = format!(
        "# {name}'s day\n\nAt the end: doing {:?}, needs {:?}, mood P{:.2} A{:.2} D{:.2}\n\n",
        doing.as_ref().map(|(n, _)| n.as_str()),
        needs,
        mood.map_or(0.0, |m| m.pleasure),
        mood.map_or(0.0, |m| m.arousal),
        mood.map_or(0.0, |m| m.dominance),
    );
    for line in lines {
        let _ = writeln!(text, "{line}");
    }
    if lines.is_empty() {
        text.push_str(
            "(no messages involved this person: a dead agent, or a scenario too short)\n",
        );
    }
    let path = out.join("narrative.txt");
    if let Err(err) = std::fs::write(&path, text) {
        eprintln!("evidence: cannot write {}: {err}", path.display());
    }
}

/// `d2 08h26` — the day matters: without it a multi-day narrative reads as though the clock ran
/// backwards at midnight (`20h43` followed by `08h26`), which is exactly how a reader first met it.
fn clock_label(clock: &SocietyClock) -> String {
    let day = clock.now_days.floor() as u32 + 1;
    format!(
        "d{day} {:02}h{:02}",
        clock.hour() as u32,
        (clock.hour().fract() * 60.0) as u32
    )
}

/// Writes `series.csv`, one row per tick.
fn series(rows: &[Row], out: &Path) {
    let mut csv = String::from(
        "tick,need0,need1,need2,need3,pleasure,arousal,dominance,opinion_variance,rumor_reach,friends,rivals,doing\n",
    );
    for row in rows {
        let needs: Vec<String> = (0..4)
            .map(|i| format!("{:.4}", row.needs.get(i).copied().unwrap_or(0.0)))
            .collect();
        let doing: Vec<String> = row
            .doing
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect();
        let _ = writeln!(
            csv,
            "{},{},{:.4},{:.4},{:.4},{:.5},{},{},{},\"{}\"",
            row.tick,
            needs.join(","),
            row.pleasure,
            row.arousal,
            row.dominance,
            row.opinion_variance,
            row.reach,
            row.friends,
            row.rivals,
            doing.join(" "),
        );
    }
    let path = out.join("series.csv");
    if let Err(err) = std::fs::write(&path, csv) {
        eprintln!("evidence: cannot write {}: {err}", path.display());
    }
}

/// The ending counts `summary()` reports, grouped so the function stays inside clippy's argument limit.
struct Endings<'a> {
    counts: (usize, usize),
    failed_by_activity: &'a BTreeMap<String, usize>,
    /// `(became rival, left rival)` — the two halves of reconciliation.
    rival_transitions: (usize, usize),
    /// Accepted `compliment`s — the only affinity-raising interaction the starter pack has.
    accepted_compliments: usize,
    /// Every `TieLabelChanged`, in either direction.
    label_changes: usize,
}

/// Writes `summary.json`.
fn summary(
    ecs: &EcsWorld,
    world: &World,
    out: &Path,
    planted: Option<bevy_society::RumorId>,
    peak_reach: f32,
    peak_known_by: usize,
    endings: Endings<'_>,
) {
    let Endings {
        counts: (completed_endings_unused, interrupted_endings_unused),
        failed_by_activity,
        rival_transitions,
        accepted_compliments,
        label_changes,
    } = endings;
    let endings = (completed_endings_unused, interrupted_endings_unused);
    let world_ref = ecs;
    let living = world.villagers.len().max(1);
    let book = world_ref.resource::<RumorBook>();
    // **The peak, not the value at the end.** A planted rumor spreads and is then *forgotten*: a person
    // keeps `knowledge_capacity` rumors (64) and evicts the least-retained, and a rumor heard at day 2
    // of a 17-day run has aged 360 hours — retention `0.995^360 = 0.165`. Measured: a `steal` planted
    // among 13 active gossipers reached **2 of 24** early and was held by **nobody** at the end, while
    // this line reported 0 because it read the final state. The peak is what the scenario asks about.
    let knowers = peak_known_by;
    let _ = (planted, world_ref);
    let (mut friends, mut rivals) = (0u32, 0u32);
    for &entity in &world.villagers {
        if let Some(ties) = world_ref.get::<Ties>(entity) {
            for tie in &ties.0 {
                match tie.label {
                    TieLabel::Friend => friends += 1,
                    TieLabel::Rival => rivals += 1,
                    _ => {}
                }
            }
        }
    }
    let clock = world_ref.resource::<SocietyClock>();
    let json = format!(
        "{{\n  \"ticks\": {},\n  \"people\": {},\n  \"rumors\": {},\n  \"rumor_known_by\": {},\n  \"rumor_reach\": {:.3},\n  \"friends\": {},\n  \"rivals\": {},\n  \"completed_endings\": {},\n  \"interrupted_endings\": {},\n  \"became_rival\": {},\n  \"left_rival\": {},\n  \"accepted_compliments\": {},\n  \"label_changes\": {},\n  \"game_days\": {:.2}\n}}\n",
        clock.tick,
        living,
        book.rumors.len(),
        knowers,
        peak_reach,
        friends / 2,
        rivals / 2,
        endings.0,
        endings.1,
        rival_transitions.0,
        rival_transitions.1,
        accepted_compliments,
        label_changes,
        clock.now_days,
    );
    let mut failed = String::new();
    for (index, (activity, count)) in failed_by_activity.iter().enumerate() {
        let comma = if index + 1 == failed_by_activity.len() {
            ""
        } else {
            ","
        };
        failed.push_str(&format!("\n    \"{activity}\": {count}{comma}"));
    }
    let json = json.replace(
        "\n  \"game_days\"",
        &format!("\n  \"interrupted_by_activity\": {{{failed}\n  }},\n  \"game_days\""),
    );
    let path = out.join("summary.json");
    if let Err(err) = std::fs::write(&path, json) {
        eprintln!("evidence: cannot write {}: {err}", path.display());
    }
}

// -------------------------------------------------------------------------------------------------
// Flags
// -------------------------------------------------------------------------------------------------

/// `--scenario <name>`, `--out <dir>`, `--seed <n>`, `--people <n>`.
struct Args {
    scenario: String,
    out: String,
    /// Overrides the scenario's seed, so the same scenario can be re-read with different people.
    seed: Option<u64>,
    /// Overrides the scenario's population.
    people: Option<usize>,
    /// Overrides `Tuning::friend_affinity`, so the same scenario can be read with a stricter or looser bar
    /// for friendship.
    friend_affinity: Option<f32>,
    /// Overrides `Tuning::label_hysteresis`, so a run can be read with a band instead of a single bar.
    label_hysteresis: Option<f32>,
    /// Overrides `Tuning::friend_band`, so a run can be read with a friendship held below its bar.
    friend_band: Option<f32>,
    /// Overrides `Tuning::positivity_offset`, so the same scenario can be re-read with a different
    /// resting mood.
    positivity_offset: Option<f32>,
    /// Overrides the scenario's `seconds_per_day`, so the same scenario can be re-read at a different
    /// watch speed. The **observable** should not move: `seconds_per_day` is how fast a host watches a day,
    /// not how much of it a person spends walking.
    seconds_per_day: Option<f32>,
    /// Runs for this many game-days instead of the scenario's `ticks:`, so a long run is one flag rather
    /// than an arithmetic exercise. The friendship bound needs thirty days.
    days: Option<f32>,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            scenario: "village-evening".to_owned(),
            out: "docs/loop/evidence".to_owned(),
            seed: None,
            people: None,
            seconds_per_day: None,
            positivity_offset: None,
            friend_affinity: None,
            label_hysteresis: None,
            friend_band: None,
            days: None,
        };
        let mut argv = std::env::args().skip(1);
        while let Some(flag) = argv.next() {
            match flag.as_str() {
                "--scenario" => args.scenario = argv.next().unwrap_or_default(),
                "--out" => args.out = argv.next().unwrap_or_default(),
                "--seed" => args.seed = argv.next().and_then(|v| v.parse().ok()),
                "--people" => args.people = argv.next().and_then(|v| v.parse().ok()),
                "--seconds-per-day" => {
                    args.seconds_per_day = argv.next().and_then(|v| v.parse().ok())
                }
                "--positivity-offset" => {
                    args.positivity_offset = argv.next().and_then(|v| v.parse().ok())
                }
                "--label-hysteresis" => {
                    args.label_hysteresis = argv.next().and_then(|v| v.parse().ok())
                }
                "--friend-band" => args.friend_band = argv.next().and_then(|v| v.parse().ok()),
                "--friend-affinity" => {
                    args.friend_affinity = argv.next().and_then(|v| v.parse().ok())
                }
                "--days" => args.days = argv.next().and_then(|v| v.parse().ok()),
                other => {
                    eprintln!("evidence: unknown argument {other}");
                    std::process::exit(1);
                }
            }
        }
        args
    }
}

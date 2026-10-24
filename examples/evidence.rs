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
    SocietyPlugins, SocietySettings, TieLabel, Ties,
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

    let ticks = ticks_for(&args.scenario);
    let tracked = world.villagers.first().copied();
    let mut rows = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut planted: Option<bevy_society::RumorId> = None;
    for tick in 0..ticks {
        app.update();
        if tick == plant_at(&args.scenario) {
            planted = plant(&mut app, &world);
        }
        rows.push(snapshot_row(&app, tick as f64, &world, planted));
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
    summary(app.world(), &world, &out, planted);
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

/// One game-day is 64 ticks: short enough that a scenario of a few thousand ticks spans days.
fn settings_for(scenario: &str) -> SocietySettings {
    let mut settings = SocietySettings {
        // Four seconds per game-day: at `tick_secs = 1/64` a tick is 0.094 game-hours, so a 0.1-hour
        // `snub` lasts about one tick and a 1200-tick run spans 4.9 days. The other values tried in
        // this file's history (`1.0`, `240.0`) put a snapshot of the same run at 19 days or at 4.9,
        // which is why the scenarios' rivals counts were not comparable across iterations.
        seconds_per_day: 4.0,
        // One society tick per update, the same rate the tests use, so a scenario's tick count is
        // the number of updates it asked for regardless of which scenario it is.
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    if scenario.starts_with("village") {
        // A four-second game-day. Activities in the starter pack last 0.1-8 game-hours, so this must
        // stay short enough that a few thousand ticks span several days: at 4.0 a tick is 0.094
        // game-hours and 4200 ticks is 17 days.
        settings.seconds_per_day = 4.0;
        settings.start_hour = 15.0;
        // Walking advances `speed / 64` metres per tick (real time) while this timeout is game
        // hours, so the two only correspond for a given `seconds_per_day`. At 4.0 one tick is 5.6
        // game-minutes, so 40 hours is 427 ticks: enough for the 6 m walk this scenario's spawn
        // layout asks for (274 ticks) and short enough that a stuck walker still gives up. The
        // coupling itself is recorded in `needs-decision/travel-time-in-two-clocks.md`.
        settings.tuning.travel_timeout_hours = 40.0;
    }
    settings
}

fn ticks_for(scenario: &str) -> usize {
    match scenario {
        "need-cycle" => 1500,
        "rumor-crosses-a-cluster" => 1200,
        // Long enough for ties to reach Friend and Rival: the `rumor-crosses-a-cluster` scenario
        // forms 19 friendships and 11 rivalries in 19 game-days, and a 4-day village forms none.
        "village-evening" | "village" => 4200,
        "deed-draws-a-response" => 400,
        _ => 800,
    }
}

fn plant_at(scenario: &str) -> usize {
    match scenario {
        "village-evening" | "village" => 560,
        "rumor-crosses-a-cluster" => 200,
        _ => usize::MAX,
    }
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
    let home = places[0];

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
                Home(home),
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
    Vec2::new(0.0, 0.0),
    Vec2::new(6.5, -5.0),
    Vec2::new(-6.5, -5.0),
    Vec2::new(6.5, 5.0),
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
fn plant(app: &mut App, world: &World) -> Option<bevy_society::RumorId> {
    let (origin, subject) = (world.origin?, world.subject?);
    let next = app.world().resource::<RumorBook>().rumors.len() as u32;
    app.world_mut().write_message(PlantRumor {
        origin,
        subject,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
    Some(bevy_society::RumorId(next))
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
            need_count += 1;
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
            let name = world_ref
                .resource::<SocietyDefs>()
                .activity_name(activity.activity)
                .to_owned();
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
    Row {
        tick,
        needs: needs_sum
            .iter()
            .map(|s| s / need_count.max(1) as f32)
            .collect(),
        pleasure: p / n,
        arousal: a / n,
        dominance: d / n,
        opinion_variance: variance,
        reach,
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
        for message in ended.drain() {
            if message.person == tracked {
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

/// Writes `summary.json`.
fn summary(ecs: &EcsWorld, world: &World, out: &Path, planted: Option<bevy_society::RumorId>) {
    let world_ref = ecs;
    let living = world.villagers.len().max(1);
    let book = world_ref.resource::<RumorBook>();
    // The planted rumor, exactly as `series.csv` does it. This counted `RumorId(0)` — the first rumor
    // the run ever made — and so reported `rumor_known_by: 0` for every scenario that plants a
    // different one, which is every scenario with a rumor.
    let knowers = planted.map_or(0, |id| {
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
        "{{\n  \"ticks\": {},\n  \"people\": {},\n  \"rumors\": {},\n  \"rumor_known_by\": {},\n  \"rumor_reach\": {:.3},\n  \"friends\": {},\n  \"rivals\": {},\n  \"game_days\": {:.2}\n}}\n",
        clock.tick,
        living,
        book.rumors.len(),
        knowers,
        knowers as f32 / living as f32,
        friends / 2,
        rivals / 2,
        clock.now_days,
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
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            scenario: "village-evening".to_owned(),
            out: "docs/loop/evidence".to_owned(),
            seed: None,
            people: None,
        };
        let mut argv = std::env::args().skip(1);
        while let Some(flag) = argv.next() {
            match flag.as_str() {
                "--scenario" => args.scenario = argv.next().unwrap_or_default(),
                "--out" => args.out = argv.next().unwrap_or_default(),
                "--seed" => args.seed = argv.next().and_then(|v| v.parse().ok()),
                "--people" => args.people = argv.next().and_then(|v| v.parse().ok()),
                other => {
                    eprintln!("evidence: unknown argument {other}");
                    std::process::exit(1);
                }
            }
        }
        args
    }
}

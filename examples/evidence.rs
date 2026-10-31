//! **The evidence harness.** Runs one scenario headlessly and writes what happened, so an iteration
//! can look at the society instead of arguing about it.
//!
//! ```text
//! cargo run --example evidence -- --scenario village-evening --out docs/loop/runs/<run>/evidence
//! ```
//!
//! The artefacts of a run:
//!
//! - `narrative.txt` — one person's day in readable lines: when they chose what, whom they talked to,
//!   what it did to their mood and needs. Read it as a stranger would.
//! - `shares.txt` — one line per accepted `share`: when, who gave to whom, and both stores before and after that tick.
//! - `series.csv` — one row per tick: the aggregates a plot needs (mean need levels, mood, opinion
//!   variance, rumor reach, friendship and rivalry counts). `tools/plots.py` turns it into PNGs.
//! - `activities.txt` — per game-day, how many times each activity was started, completed and interrupted, and the
//!   clock hour each was chosen at: "nobody bakes" told apart from "nobody decides inside the baking window".
//! - `floor.txt` — who is at the floor on `food` and what they are doing, and how long each spell of hunger lasts
//!   from the tick `food` falls under the `eat` midpoint to the tick it is back over it.
//! - `summary.json` — the end state, for a test or a log entry to assert against.
//!
//! `--provisioning` runs the scenario on `examples/assets/provisioning.society.ron`, whose `bake` and `eat` read the
//! actor's own shelf (`Input::Stock`), so the same seed can be read with and without households that provision themselves.
//! That overlay also carries an `urgent` rank on both (hunger takes precedence); `--no-urgent` removes it and
//! `--urgent BELOW` sets the level, so the same seed can be read with and without it.
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
    ActivityEnded, ActivityStarted, Autonomous, Doing, GoodId, Home, InteractionResolved,
    Knowledge, Mood, Needs, Opinions, Person, Place, PlaceTags, PlantRumor, RumorBook,
    SocietyClock, SocietyDefs, SocietyPlugins, SocietySettings, TieLabel, TieLabelChanged, Ties,
};
use bevy_society::{Chronotype, Memories, MemoryKind};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let args = Args::parse();
    let defs = match society_defs(
        &args.scenario,
        args.provisioning,
        args.urgent,
        args.sleepers_available,
    ) {
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
    if let Some(band) = args.rival_band {
        settings.tuning.rival_band = band;
    }
    if let Some(rate) = args.dyad_balance {
        settings.tuning.dyad_balance_per_day = rate;
    }
    if let Some(exponent) = args.witness_dilution {
        settings.tuning.witness_dilution = exponent;
    }
    if let Some(ratio) = args.negativity_ratio {
        settings.tuning.negativity_ratio = ratio;
    }
    if let Some(penalty) = args.rejection_penalty {
        settings.tuning.rejection_affinity_penalty = penalty;
    }
    if let Some(hours) = args.stall_timeout {
        settings.tuning.stall_timeout_hours = Some(hours);
    }
    if let Some(capacity) = args.friend_capacity {
        settings.tuning.friend_capacity = capacity;
    }
    if let Some(hours) = args.retaliation_refractory {
        settings.tuning.retaliation_refractory_hours = Some(hours);
    }
    if let Some(gain) = args.forgiveness {
        settings.tuning.forgiveness = gain;
    }
    if let Some(hours) = args.failed_place_memory {
        settings.tuning.failed_place_memory_hours = Some(hours);
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
    if let Some(sd) = args.chronotype_sd {
        give_body_clocks(&mut app, &world, sd);
    }

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
    let tracked = world
        .villagers
        .get(args.track.unwrap_or(0))
        .copied()
        .or_else(|| world.villagers.first().copied());
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
    // **Does a person keep going back to the same partner?** The last partner of each initiator, and how
    // many exchanges were with that same partner again.
    let mut last_partner: BTreeMap<Entity, Entity> = BTreeMap::new();
    let mut repeat_exchanges = 0usize;
    let mut all_exchanges = 0usize;
    let mut repeat_by_interaction: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    // **The victim-side counter.** Every accepted `insult`, and how many of those left the victim's affinity
    // for its insulter below zero. Before the patient's affinity write existed this was `0 of N` by
    // construction — the tie was whatever mere exposure had made it, measured at `+0.034` on this scenario.
    let mut grudge_sources: BTreeMap<String, usize> = BTreeMap::new();
    let (mut insult_deeds, mut disliked_insulter) = (0usize, 0usize);
    // **Sharing, counted where it resolves.** An accepted `share` is one unit changing hands; the number is
    // what a plot of the store spread needs next to it, because a flat spread with no shares is a village
    // that never traded and a flat spread with many is one that is already level.
    let mut shares = 0usize;
    let mut share_log: Vec<String> = Vec::new();
    let mut became_rival = 0usize;
    // **Every** label transition, not only the rival ones. A host that shows "you are now friends"
    // or unlocks a conversation on `TieLabelChanged` cares about how many arrive and whether they mean
    // anything: measured before hysteresis, 58% of them came from a value resting within 0.01 of the bar.
    let mut label_changes = 0usize;
    // **Every approach**, accepted or refused: the quantity a reciprocity step should move, because a
    // person choosing an approach with its own tie and being judged by the other's is the asymmetry.
    let (mut accepted_approaches, mut refused_approaches) = (0usize, 0usize);
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
    // **Who did what, when.** One count per game-day and activity, and a histogram of the hour each
    // activity was chosen at, so "nobody bakes" can be told from "nobody decides inside the baking window".
    let mut ledger = Ledger::default();
    let mut floor_ledger = FloorLedger::new(app.world().resource::<SocietyDefs>());
    let mut sleep_ledger = SleepLedger::default();
    for tick in 0..ticks {
        // Each villager's first-good store **before** this update, so a share can be logged as
        // `before -> after` for both sides. After alone read as a bug: "Dena holds 0.0, Hilda holds 0.0" is
        // a loaf that arrived and was eaten in the same tick, and cannot be told from a transfer that failed.
        let held_before: Vec<(Entity, f32)> = world
            .villagers
            .iter()
            .map(|&e| {
                let held = app
                    .world()
                    .get::<bevy_society::Stock>(e)
                    .map_or(0.0, |stock| stock.get(GoodId(0)));
                (e, held)
            })
            .collect();
        app.update();
        if tick == plant_tick {
            planted = plant(&mut app, &world, &gossipers);
        }
        let mut shared_now: Vec<(Entity, Entity)> = Vec::new();
        // **Two messages are read by two consumers, so they are drained once into a `Vec`.** The
        // narrative needs every `InteractionResolved` and `ActivityEnded` that touches the tracked
        // person, and the counters below need all of them for the whole village. Draining in the counters
        // first — which is what this did — left the narrative with **no interaction line and no
        // `finished` line at all**: a reader saw `started snub` with no target and no outcome, on the
        // change whose whole subject is which pairs are rivals. Nothing said so; the file simply read
        // plausibly.
        let resolved_now: Vec<InteractionResolved> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
            .drain()
            .collect();
        let ended_now: Vec<ActivityEnded> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
            .drain()
            .collect();
        {
            for resolved in &resolved_now {
                // **Both sides of the affinity ledger.** `compliment` is the only activity that raises
                // affinity between two people; a witnessed deed lowers it for every witness. Without
                // counting both, a claim about which side outweighs the other cannot be checked — and an
                // earlier iteration's "bad outweighs good 4.8:1" was arithmetic on the wrong variable.
                if resolved.interaction == "compliment" && resolved.accepted {
                    accepted_compliments += 1;
                }
                if resolved.interaction == "insult" && resolved.accepted {
                    insult_deeds += 1;
                    // Read *after* the tick that resolved the exchange: `resolve_interactions` writes the
                    // patient's tie in the same system, so by the time this message is drained the affinity
                    // is the post-deed one.
                    let affinity = app
                        .world()
                        .get::<Ties>(resolved.target)
                        .and_then(|ties| ties.get(resolved.actor))
                        .map_or(0.0, |tie| tie.affinity);
                    if affinity < 0.0 {
                        disliked_insulter += 1;
                    }
                }
                if resolved.interaction == "share" && resolved.accepted {
                    shares += 1;
                    shared_now.push((resolved.actor, resolved.target));
                }
                // Every approach, accepted or refused. A reciprocity step should move this ratio without
                // touching the acceptance model, because the model already reads the *target's* tie.
                if resolved.refused_by_rule {
                    continue;
                }
                all_exchanges += 1;
                let slept = app
                    .world()
                    .get::<Doing>(resolved.target)
                    .is_some_and(|doing| {
                        doing.label(app.world().resource::<SocietyDefs>()) == "sleep"
                    });
                sleep_ledger.exchange(&resolved.interaction, slept);
                let counts = repeat_by_interaction
                    .entry(resolved.interaction.clone())
                    .or_default();
                counts.0 += 1;
                if last_partner.insert(resolved.actor, resolved.target) == Some(resolved.target) {
                    repeat_exchanges += 1;
                    counts.1 += 1;
                }
                if resolved.accepted {
                    accepted_approaches += 1;
                } else {
                    refused_approaches += 1;
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
        // **One line per loaf shared**, with both stores *after* the move, so a reader can see the loaf go
        // from the shelf that had it to the shelf that did not. The counter says how many; this says who.
        for (giver, receiver) in shared_now {
            let world_ref = app.world();
            let name = |e: Entity| {
                world_ref
                    .get::<Name>(e)
                    .map_or_else(|| "someone".to_owned(), |n| n.as_str().to_owned())
            };
            let held = |e: Entity| {
                world_ref
                    .get::<bevy_society::Stock>(e)
                    .map_or(0.0, |stock| stock.get(GoodId(0)))
            };
            let was = |e: Entity| {
                held_before
                    .iter()
                    .find(|(other, _)| *other == e)
                    .map_or(0.0, |(_, held)| *held)
            };
            share_log.push(format!(
                "{}  {} -> {}  ({} {:.1} -> {:.1}, {} {:.1} -> {:.1}, over this tick)",
                clock_label(world_ref.resource::<SocietyClock>()),
                name(giver),
                name(receiver),
                name(giver),
                was(giver),
                held(giver),
                name(receiver),
                was(receiver),
                held(receiver),
            ));
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
            let now = app.world().resource::<SocietyClock>().now_days;
            let hour = app.world().resource::<SocietyClock>().hour();
            let started = app
                .world()
                .resource::<bevy::ecs::message::Messages<ActivityStarted>>();
            for message in started.iter_current_update_messages() {
                ledger.started(now, hour, &message.activity);
                if message.activity == "sleep" {
                    sleep_ledger.began[hour.floor() as usize % 24] += 1;
                }
            }
            for message in &ended_now {
                ledger.ended(now, &message.activity, message.completed);
                if message.activity == "sleep" {
                    let hour = app.world().resource::<SocietyClock>().hour();
                    if message.completed {
                        sleep_ledger.woke[hour.floor() as usize % 24] += 1;
                    } else {
                        sleep_ledger.interrupted += 1;
                    }
                }
            }
        }
        {
            for message in &ended_now {
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
        floor_ledger.record(&app, &world, tick > 0);
        sleep_ledger.record(&app, &world, tick > 0);
        let mut row = snapshot_row(&app, tick as f64, &world, planted);
        row.shares = shares;
        // The first row is the pre-society state: `size_person_state` has not run, so `Needs` is empty
        // and every need reads 0.000. Recording it made "all four needs cross below 0.35" true of the
        // mean on the strength of one row at tick 0, while no individual ever came close.
        if tick > 0 {
            peak_reach = peak_reach.max(row.reach);
            peak_known_by = peak_known_by.max(row.known_by);
            rows.push(row);
        }
        // **What is behind each snub?** At the tick a `snub` starts, the memory that sets the snubber's
        // grudge (the one with the largest `max(0, -valence) * recency`) is classified by kind. `snub`'s
        // only consideration is that grudge, so this is the direct answer to "what is the town reacting to".
        {
            let now = app.world().resource::<SocietyClock>().now_days;
            let started = app
                .world()
                .resource::<bevy::ecs::message::Messages<ActivityStarted>>();
            for message in started.iter_current_update_messages() {
                let (true, Some(target)) = (message.activity == "snub", message.target) else {
                    continue;
                };
                let Some(memories) = app.world().get::<Memories>(message.person) else {
                    continue;
                };
                let best = memories
                    .entries
                    .iter()
                    .filter(|m| m.other == Some(target))
                    .map(|m| {
                        let hours = ((now - f64::from(m.day)) * 24.0).max(0.0) as f32;
                        (
                            (-m.valence).max(0.0) * bevy_society::model::recall::recency(hours),
                            m,
                        )
                    })
                    .filter(|(weight, _)| *weight > 0.0)
                    .max_by(|a, b| a.0.total_cmp(&b.0));
                let label = match best {
                    None => "none".to_owned(),
                    Some((_, m)) => match &m.kind {
                        MemoryKind::Interaction {
                            id,
                            accepted,
                            initiated,
                        } => format!(
                            "interaction {} {} {}",
                            id.0,
                            if *initiated { "initiated" } else { "received" },
                            if *accepted { "accepted" } else { "refused" }
                        ),
                        MemoryKind::Witnessed { .. } => "witnessed".to_owned(),
                        MemoryKind::Suffered { .. } => "suffered".to_owned(),
                        MemoryKind::HeardRumor(_) => "rumor".to_owned(),
                        MemoryKind::Talked => "talked".to_owned(),
                        MemoryKind::Custom(_) => "custom".to_owned(),
                    },
                };
                *grudge_sources.entry(label).or_insert(0) += 1;
            }
        }
        drain_narrative(
            &mut app,
            &world,
            tracked,
            &resolved_now,
            &ended_now,
            &mut lines,
        );
    }

    let out = PathBuf::from(&args.out);
    if let Err(err) = std::fs::create_dir_all(&out) {
        eprintln!("evidence: cannot create {}: {err}", out.display());
        std::process::exit(1);
    }
    write_narrative(&mut app, tracked, &lines, &out);
    {
        let path = out.join("partners.txt");
        let mut text = format!(
            "exchanges: {all_exchanges}\nwith the initiator's previous partner: {repeat_exchanges} ({:.1}%)\n",
            100.0 * repeat_exchanges as f32 / all_exchanges.max(1) as f32
        );
        for (name, (all, repeat)) in &repeat_by_interaction {
            text.push_str(&format!("  {name}: {repeat} of {all}\n"));
        }
        if let Err(err) = std::fs::write(&path, text) {
            eprintln!("evidence: cannot write {}: {err}", path.display());
        }
    }
    if !share_log.is_empty() {
        let path = out.join("shares.txt");
        if let Err(err) = std::fs::write(&path, share_log.join("\n") + "\n") {
            eprintln!("evidence: cannot write {}: {err}", path.display());
        }
    }
    write_why(&mut app, tracked, &out);
    {
        let total: usize = grudge_sources.values().sum();
        let mut text = format!("snubs started: {total}\n");
        for (label, n) in &grudge_sources {
            let _ = writeln!(
                text,
                "{n:>6}  {:>5.1}%  {label}",
                100.0 * *n as f32 / total.max(1) as f32
            );
        }
        if let Err(err) = std::fs::write(out.join("grudge_sources.txt"), text) {
            eprintln!("evidence: cannot write grudge_sources.txt: {err}");
        }
    }
    ledger.write(&out);
    floor_ledger.write(&out);
    sleep_ledger.write(&out);
    write_chronotypes(&app, &world, &out);
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
            shares,
            label_changes,
            reciprocity: reciprocity_of(app.world(), &world),
            approaches: (accepted_approaches, refused_approaches),
            victims: (insult_deeds, disliked_insulter),
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

/// What each activity did, per game-day, and the hour of day it was chosen at.
#[derive(Default)]
struct Ledger {
    /// `(game-day, activity) -> (started, completed, interrupted)`.
    days: BTreeMap<(u32, String), (u32, u32, u32)>,
    /// `(activity, hour of day) -> chosen`.
    hours: BTreeMap<(String, u32), u32>,
}

impl Ledger {
    fn started(&mut self, now: f64, hour: f32, activity: &str) {
        self.days
            .entry((now.floor() as u32 + 1, activity.to_owned()))
            .or_default()
            .0 += 1;
        *self
            .hours
            .entry((activity.to_owned(), hour.floor() as u32 % 24))
            .or_default() += 1;
    }

    fn ended(&mut self, now: f64, activity: &str, completed: bool) {
        let entry = self
            .days
            .entry((now.floor() as u32 + 1, activity.to_owned()))
            .or_default();
        if completed {
            entry.1 += 1;
        } else {
            entry.2 += 1;
        }
    }

    /// `activities.txt`: a table per activity of `day  started completed interrupted`, then the hour
    /// histogram of when each was chosen.
    fn write(&self, out: &Path) {
        let mut names: Vec<&str> = self.days.keys().map(|(_, name)| name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        let mut text = String::new();
        for name in &names {
            let _ = writeln!(text, "{name}: day started completed interrupted");
            for ((day, activity), (started, completed, interrupted)) in &self.days {
                if activity == name {
                    let _ = writeln!(
                        text,
                        "  {day:>3} {started:>6} {completed:>6} {interrupted:>6}"
                    );
                }
            }
        }
        let _ = writeln!(
            text,
            "\nhour of day each activity was chosen at (all days):"
        );
        for name in &names {
            let mut row = format!("{name:>10}:");
            for hour in 0..24u32 {
                let _ = write!(
                    row,
                    " {:>5}",
                    self.hours
                        .get(&(name.to_string(), hour))
                        .copied()
                        .unwrap_or(0)
                );
            }
            let _ = writeln!(text, "{row}");
        }
        if let Err(err) = std::fs::write(out.join("activities.txt"), text) {
            eprintln!("evidence: cannot write activities.txt: {err}");
        }
    }
}

/// **Who is at the floor on `food`, and what are they doing.** A share of person-ticks under [`FLOOR`] says how much of
/// the day goes hungry; this says whose fault it is (asleep, walking, busy with something else, or at the table) and
/// whether the household had a meal in hand. A spell is one stretch with `food` under [`HUNGRY`], the `eat` curve's
/// midpoint: the hours it lasts are what a hunger that takes precedence would shorten.
struct FloorLedger {
    /// The `food` slot of `Needs`, `None` in a pack without one.
    food: Option<usize>,
    /// The first good's holding a person needs to count as holding a meal (`eat` consumes a quarter loaf).
    meal: f32,
    /// `(what the person is doing, holds a meal) -> person-ticks at the floor`, from game-day 2 on.
    doing: BTreeMap<(String, bool), u64>,
    /// Person-ticks counted (every villager, every tick, from game-day 2 on) and how many were at the floor.
    ticks: (u64, u64),
    /// When each villager's current spell began, in game-days.
    since: BTreeMap<Entity, f64>,
    /// Finished spells, in game-hours.
    spells: Vec<f32>,
}

/// `food` under this starts a spell of hunger: `eat`'s `Logistic(-10, 0.35)` is half way to its ceiling here.
const HUNGRY: f32 = 0.35;

impl FloorLedger {
    fn new(defs: &SocietyDefs) -> Self {
        Self {
            food: defs.need_id("food").map(|id| usize::from(id.0)),
            meal: 0.25,
            doing: BTreeMap::new(),
            ticks: (0, 0),
            since: BTreeMap::new(),
            spells: Vec::new(),
        }
    }

    fn record(&mut self, app: &App, world: &World, live: bool) {
        let (Some(slot), true) = (self.food, live) else {
            return;
        };
        let ecs = app.world();
        let now = ecs.resource::<SocietyClock>().now_days;
        let defs = ecs.resource::<SocietyDefs>();
        for &entity in &world.villagers {
            let Some(level) = ecs
                .get::<Needs>(entity)
                .and_then(|needs| needs.0.get(slot).copied())
            else {
                continue;
            };
            if level < HUNGRY {
                self.since.entry(entity).or_insert(now);
            } else if let Some(began) = self.since.remove(&entity) {
                self.spells.push(((now - began) * 24.0) as f32);
            }
            if now < 2.0 {
                continue;
            }
            self.ticks.0 += 1;
            if level >= FLOOR {
                continue;
            }
            self.ticks.1 += 1;
            let label = ecs
                .get::<Doing>(entity)
                .map_or_else(|| "nothing".to_owned(), |doing| doing.label(defs));
            let holds = ecs
                .get::<bevy_society::Stock>(entity)
                .is_some_and(|stock| stock.get(GoodId(0)) >= self.meal);
            *self.doing.entry((label, holds)).or_default() += 1;
        }
    }

    fn write(&self, out: &Path) {
        let (total, floor) = self.ticks;
        let mut text = format!(
            "person-ticks from game-day 2: {total}\nat the floor on food (< {FLOOR}): {floor} ({:.1}%)\n\n",
            100.0 * floor as f64 / total.max(1) as f64
        );
        let _ = writeln!(
            text,
            "what the person at the floor was doing, and whether the household held a meal ({} of the first good):",
            self.meal
        );
        let mut rows: Vec<(&(String, bool), &u64)> = self.doing.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for ((label, holds), n) in rows {
            let _ = writeln!(
                text,
                "{n:>9}  {:>5.1}%  {label}  [{}]",
                100.0 * *n as f64 / floor.max(1) as f64,
                if *holds { "holds a meal" } else { "no meal" }
            );
        }
        let mut spells = self.spells.clone();
        spells.sort_by(f32::total_cmp);
        let at = |q: f32| {
            spells
                .get(((spells.len() as f32 - 1.0) * q).round().max(0.0) as usize)
                .copied()
                .unwrap_or(0.0)
        };
        let mean = spells.iter().sum::<f32>() / spells.len().max(1) as f32;
        let _ = writeln!(
            text,
            "\nspells of hunger (food under {HUNGRY} until back over it), game-hours: {} finished, mean {mean:.2}, median {:.2}, p90 {:.2}, max {:.2}",
            spells.len(),
            at(0.5),
            at(0.9),
            at(1.0)
        );
        if let Err(err) = std::fs::write(out.join("floor.txt"), text) {
            eprintln!("evidence: cannot write floor.txt: {err}");
        }
    }
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

/// A need below this level is "at the floor": nothing is left of it.
const FLOOR: f32 = 0.05;

/// One row of the time series.
struct Row {
    tick: f64,
    needs: Vec<f32>,
    /// How many villagers have need *k* below [`FLOOR`], per need: the count of people the need has left at the
    /// bottom. The mean hides a village in which some people are starving and the rest are fine.
    floor: Vec<u32>,
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
    /// The first good's store across the village: `(min, mean, max, standard deviation)`. Zeros when the
    /// definitions declare no goods.
    store: (f32, f32, f32, f32),
    /// The first good's price.
    price: f32,
    /// Accepted `share`s so far, cumulative.
    shares: usize,
}

/// The starter pack, plus whatever the village scenario adds, and (`provisioning`) the overlay whose `bake` and
/// `eat` answer to the actor's own shelf.
fn society_defs(
    scenario: &str,
    provisioning: bool,
    urgent: Option<Option<f32>>,
    sleepers_available: bool,
) -> Result<SocietyDefs, String> {
    let mut file = bevy_society::DefsFile::starter().map_err(|e| e.to_string())?;
    if provisioning {
        let overlay =
            bevy_society::DefsFile::from_ron(include_str!("assets/provisioning.society.ron"))
                .map_err(|e| e.to_string())?;
        for activity in overlay.activities {
            let slot = file.activities.iter_mut().find(|a| a.id == activity.id);
            match slot {
                Some(slot) => *slot = activity,
                None => return Err(format!("the starter pack has no '{}'", activity.id)),
            }
        }
    }
    if scenario.starts_with("village") {
        let additions =
            bevy_society::DefsFile::from_ron(include_str!("assets/village.society.ron"))
                .map_err(|e| e.to_string())?;
        file.merge(additions).map_err(|e| e.to_string())?;
    }
    // `--urgent BELOW` puts `eat` (and, on the provisioning overlay, `bake`) in the rank-1 bucket while `food` is
    // under BELOW; `--no-urgent` takes every activity out of it, so the overlay can be read with and without.
    match urgent {
        Some(Some(below)) => {
            for activity in &mut file.activities {
                if activity.id == "eat" || (provisioning && activity.id == "bake") {
                    activity.urgent = Some(bevy_society::UrgentDef {
                        need: "food".to_owned(),
                        below,
                        rank: 1,
                    });
                }
            }
        }
        Some(None) => {
            for activity in &mut file.activities {
                activity.urgent = None;
            }
        }
        None => {}
    }
    // `--sleepers-available` puts every `unavailable` activity back on the list of things a person can be approached
    // during, so a run can be read with and without the rule that nobody greets a sleeper.
    if sleepers_available {
        for activity in &mut file.activities {
            activity.unavailable = false;
        }
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
/// `(mutual friend pairs, one-way friend pairs, mean |aff(a->b) - aff(b->a)|)` over every pair.
///
/// The crate holds each direction of a tie separately, so a friendship can be one-sided. Measured before
/// the dyad balance step: **50%** of directed friend ties were unreciprocated and the two directions of a
/// pair disagreed by **0.145**, against a friend band 0.05 wide.
fn reciprocity_of(ecs: &EcsWorld, world: &World) -> (usize, usize, f32) {
    let mut mutual = 0usize;
    let mut one_way = 0usize;
    let (mut gap_sum, mut pairs) = (0.0_f64, 0usize);
    let people = &world.villagers;
    for (i, &a) in people.iter().enumerate() {
        for &b in &people[i + 1..] {
            let (Some(ta), Some(tb)) = (
                ecs.get::<Ties>(a).and_then(|t| t.get(b)),
                ecs.get::<Ties>(b).and_then(|t| t.get(a)),
            ) else {
                continue;
            };
            gap_sum += f64::from((ta.affinity - tb.affinity).abs());
            pairs += 1;
            let (fa, fb) = (ta.label == TieLabel::Friend, tb.label == TieLabel::Friend);
            if fa && fb {
                mutual += 1
            } else if fa || fb {
                one_way += 1
            }
        }
    }
    (mutual, one_way, (gap_sum / pairs.max(1) as f64) as f32)
}

fn snapshot_row(
    app: &App,
    tick: f64,
    world: &World,
    planted: Option<bevy_society::RumorId>,
) -> Row {
    let world_ref = app.world();
    let living = world.villagers.len().max(1) as f32;
    let (mut needs_sum, mut need_count) = (vec![0.0_f32; 4], 0usize);
    let mut floor = vec![0_u32; 4];
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
                if *level < FLOOR
                    && let Some(count) = floor.get_mut(slot)
                {
                    *count += 1;
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
        floor,
        pleasure: p / n,
        arousal: a / n,
        dominance: d / n,
        opinion_variance: variance,
        reach,
        known_by,
        // Directed ties, the unit `tests/friendship_bound.rs` counts. Halving them assumed every tie is mutual;
        // a one-way rivalry (the common case after a snub) rounded down to nothing.
        friends,
        rivals,
        doing,
        store: store_spread(world_ref, &world.villagers),
        price: world_ref
            .get_resource::<bevy_society::Prices>()
            .map_or(0.0, |prices| prices.get(GoodId(0))),
        shares: 0,
    }
}

/// `(min, mean, max, standard deviation)` of the first good's store over the villagers.
///
/// The standard deviation is the one number that says whether stores are level: a village that shares
/// should end with a smaller spread than one that does not, and the mean is what stays put.
fn store_spread(ecs: &EcsWorld, villagers: &[Entity]) -> (f32, f32, f32, f32) {
    let held: Vec<f32> = villagers
        .iter()
        .filter_map(|&e| ecs.get::<bevy_society::Stock>(e))
        .map(|stock| stock.get(GoodId(0)))
        .collect();
    if held.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let n = held.len() as f32;
    let mean = held.iter().sum::<f32>() / n;
    let sd = (held.iter().map(|h| (h - mean).powi(2)).sum::<f32>() / n).sqrt();
    let min = held.iter().copied().fold(f32::MAX, f32::min);
    let max = held.iter().copied().fold(f32::MIN, f32::max);
    (min, mean, max, sd)
}

/// Reads the messages a tracked person produced this tick into `lines`.
fn drain_narrative(
    app: &mut App,
    world: &World,
    tracked: Option<Entity>,
    resolved_now: &[InteractionResolved],
    ended_now: &[bevy_society::ActivityEnded],
    lines: &mut Vec<String>,
) {
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
        for message in ended_now.iter().filter(|m| m.person == tracked) {
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
        for message in resolved_now {
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
        "tick,need0,need1,need2,need3,floor0,floor1,floor2,floor3,pleasure,arousal,dominance,opinion_variance,rumor_reach,friends,rivals,bread_min,bread_mean,bread_max,bread_sd,price,shares,doing\n",
    );
    for row in rows {
        let needs: Vec<String> = (0..4)
            .map(|i| format!("{:.4}", row.needs.get(i).copied().unwrap_or(0.0)))
            .collect();
        let floor: Vec<String> = (0..4)
            .map(|i| row.floor.get(i).copied().unwrap_or(0).to_string())
            .collect();
        let doing: Vec<String> = row
            .doing
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect();
        let _ = writeln!(
            csv,
            "{},{},{},{:.4},{:.4},{:.4},{:.5},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{},\"{}\"",
            row.tick,
            needs.join(","),
            floor.join(","),
            row.pleasure,
            row.arousal,
            row.dominance,
            row.opinion_variance,
            row.reach,
            row.friends,
            row.rivals,
            row.store.0,
            row.store.1,
            row.store.2,
            row.store.3,
            row.price,
            row.shares,
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
    /// Accepted `share`s, each one unit of a good changing hands.
    shares: usize,
    /// Every `TieLabelChanged`, in either direction.
    label_changes: usize,
    /// `(mutual friends, one-way friends, mean |aff(a->b) - aff(b->a)|)` over every pair.
    reciprocity: (usize, usize, f32),
    /// `(accepted, refused)` approaches.
    approaches: (usize, usize),
    /// `(accepted insults, of those where the victim's affinity for its insulter is negative afterwards)`.
    ///
    /// **The observable this crate could not answer before 2026-10-27-03.** `patient_reaction` moved trust and
    /// no affinity, so a victim's liking for the person who had just insulted them was whatever mere exposure
    /// had made it: measured on this scenario at seed 7, **+0.034**, positive. A count of victims holding a
    /// negative tie says in one number whether being hurt costs the relationship.
    victims: (usize, usize),
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
        shares,
        label_changes,
        reciprocity,
        approaches,
        victims,
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
    let spread = store_spread(world_ref, &world.villagers);
    let json = format!(
        "{{\n  \"ticks\": {},\n  \"people\": {},\n  \"rumors\": {},\n  \"rumor_known_by\": {},\n  \"rumor_reach\": {:.3},\n  \"friends\": {},\n  \"rivals\": {},\n  \"completed_endings\": {},\n  \"interrupted_endings\": {},\n  \"became_rival\": {},\n  \"left_rival\": {},\n  \"accepted_compliments\": {},\n  \"label_changes\": {},\n  \"mutual_friends\": {},\n  \"one_way_friends\": {},\n  \"affinity_gap\": {:.3},\n  \"refusals\": {},\n  \"acceptances\": {},\n  \"shares\": {},\n  \"insults\": {},\n  \"victims_who_dislike_their_insulter\": {},\n  \"bread_mean\": {:.3},\n  \"bread_sd\": {:.3},\n  \"bread_max\": {:.3},\n  \"price\": {:.3},\n  \"game_days\": {:.2}\n}}\n",
        clock.tick,
        living,
        book.rumors.len(),
        knowers,
        peak_reach,
        friends,
        rivals,
        endings.0,
        endings.1,
        rival_transitions.0,
        rival_transitions.1,
        accepted_compliments,
        label_changes,
        reciprocity.0,
        reciprocity.1,
        reciprocity.2,
        approaches.1,
        approaches.0,
        shares,
        victims.0,
        victims.1,
        spread.1,
        spread.3,
        spread.2,
        world_ref
            .get_resource::<bevy_society::Prices>()
            .map_or(0.0, |prices| prices.get(GoodId(0))),
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
    /// Overrides `Tuning::rival_band`, so the band a rivalry is held through can be swept without
    /// editing the crate.
    rival_band: Option<f32>,
    /// Overrides `Tuning::dyad_balance_per_day`, so a run can be read with ties pulled toward reciprocity.
    dyad_balance: Option<f32>,
    /// `Tuning::witness_dilution`, so the exponent can be swept without editing the crate.
    witness_dilution: Option<f32>,
    /// `Tuning::negativity_ratio`, so how much more an injury moves its victim than a benefit of the same
    /// size can be swept without editing the crate. `0.0` switches the harm half off at the patient.
    negativity_ratio: Option<f32>,
    /// `Tuning::rejection_affinity_penalty`, so what a refused approach costs the approacher can be swept.
    rejection_penalty: Option<f32>,
    /// Overrides `Tuning::positivity_offset`, so the same scenario can be re-read with a different
    /// resting mood.
    positivity_offset: Option<f32>,
    /// Overrides the scenario's `seconds_per_day`, so the same scenario can be re-read at a different
    /// watch speed. The **observable** should not move: `seconds_per_day` is how fast a host watches a day,
    /// not how much of it a person spends walking.
    seconds_per_day: Option<f32>,
    /// Overrides `Tuning::stall_timeout_hours`, in game hours.
    stall_timeout: Option<f32>,
    /// Overrides `Tuning::failed_place_memory_hours`, in game hours.
    failed_place_memory: Option<f32>,
    /// Overrides `Tuning::friend_capacity`.
    friend_capacity: Option<f32>,
    /// Overrides `Tuning::retaliation_refractory_hours`, in game hours.
    retaliation_refractory: Option<f32>,
    /// Overrides `Tuning::forgiveness`.
    forgiveness: Option<f32>,
    /// Runs for this many game-days instead of the scenario's `ticks:`, so a long run is one flag rather
    /// than an arithmetic exercise. The friendship bound needs thirty days.
    days: Option<f32>,
    /// Replaces the starter pack's `bake` and `eat` with `assets/provisioning.society.ron`'s, which read the
    /// actor's own shelf (`Input::Stock`), so the same scenario can be read with households that provision themselves.
    provisioning: bool,
    /// `--urgent BELOW` / `--no-urgent`: override the pack's `urgent` rank (see `UrgentDef`); `None` leaves it.
    urgent: Option<Option<f32>>,
    /// `--chronotype-sd H`: gives every villager a `Chronotype` drawn from a normal of standard deviation `H` hours
    /// (`Chronotype::sample`), from a stream of its own so every other draw of the run is the one it had without
    /// the flag. `None` (the default) leaves everybody on the village's clock.
    chronotype_sd: Option<f32>,
    /// `--sleepers-available`: clears `ActivityDef::unavailable` on every activity (a sleeper can be approached again).
    sleepers_available: bool,
    /// `--track N`: the narrative follows the N-th villager in spawn order (0, the first, by default), so a lark's day
    /// and an owl's can be read from one run.
    track: Option<usize>,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            scenario: "village-evening".to_owned(),
            out: "docs/loop/evidence".to_owned(),
            seed: None,
            people: None,
            seconds_per_day: None,
            stall_timeout: None,
            failed_place_memory: None,
            friend_capacity: None,
            retaliation_refractory: None,
            forgiveness: None,
            positivity_offset: None,
            friend_affinity: None,
            label_hysteresis: None,
            friend_band: None,
            rival_band: None,
            dyad_balance: None,
            witness_dilution: None,
            negativity_ratio: None,
            rejection_penalty: None,
            days: None,
            provisioning: false,
            urgent: None,
            chronotype_sd: None,
            sleepers_available: false,
            track: None,
        };
        let mut argv = std::env::args().skip(1);
        while let Some(flag) = argv.next() {
            match flag.as_str() {
                "--scenario" => args.scenario = argv.next().unwrap_or_default(),
                "--out" => args.out = argv.next().unwrap_or_default(),
                "--seed" => args.seed = argv.next().and_then(|v| v.parse().ok()),
                "--chronotype-sd" => args.chronotype_sd = argv.next().and_then(|v| v.parse().ok()),
                "--sleepers-available" => args.sleepers_available = true,
                "--track" => args.track = argv.next().and_then(|v| v.parse().ok()),
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
                "--friend-capacity" => {
                    args.friend_capacity = argv.next().and_then(|v| v.parse().ok())
                }
                "--friend-band" => args.friend_band = argv.next().and_then(|v| v.parse().ok()),
                "--rival-band" => args.rival_band = argv.next().and_then(|v| v.parse().ok()),
                "--dyad-balance" => args.dyad_balance = argv.next().and_then(|v| v.parse().ok()),
                "--witness-dilution" => {
                    args.witness_dilution = argv.next().and_then(|v| v.parse().ok())
                }
                "--negativity-ratio" => {
                    args.negativity_ratio = argv.next().and_then(|v| v.parse().ok())
                }
                "--rejection-penalty" => {
                    args.rejection_penalty = argv.next().and_then(|v| v.parse().ok())
                }
                "--friend-affinity" => {
                    args.friend_affinity = argv.next().and_then(|v| v.parse().ok())
                }
                "--days" => args.days = argv.next().and_then(|v| v.parse().ok()),
                "--provisioning" => args.provisioning = true,
                "--urgent" => args.urgent = argv.next().and_then(|v| v.parse().ok()).map(Some),
                "--no-urgent" => args.urgent = Some(None),
                "--stall-timeout" => args.stall_timeout = argv.next().and_then(|v| v.parse().ok()),
                "--retaliation-refractory" => {
                    args.retaliation_refractory = argv.next().and_then(|v| v.parse().ok())
                }
                "--forgiveness" => args.forgiveness = argv.next().and_then(|v| v.parse().ok()),
                "--failed-place-memory" => {
                    args.failed_place_memory = argv.next().and_then(|v| v.parse().ok())
                }
                other => {
                    eprintln!("evidence: unknown argument {other}");
                    std::process::exit(1);
                }
            }
        }
        args
    }
}

/// Gives every villager a body clock from a stream of its own (`seed ^ salt`), so personalities, ties and every
/// other draw of the run are the ones it has without `--chronotype-sd`.
fn give_body_clocks(app: &mut App, world: &World, sd: f32) {
    let seed = app.world().resource::<SocietySettings>().seed;
    let mut rng = bevy_emerge_rng::seeded(seed ^ 0xC1C0_C10C);
    for &entity in &world.villagers {
        let chronotype = Chronotype::sample(&mut rng, sd);
        app.world_mut().entity_mut(entity).insert(chronotype);
    }
}

/// `chronotypes.txt`: each villager's offset, in hours, in spawn order.
fn write_chronotypes(app: &App, world: &World, out: &Path) {
    let mut text = String::from("name offset_hours\n");
    for &entity in &world.villagers {
        let name = app
            .world()
            .get::<Name>(entity)
            .map_or("someone", |name| name.as_str());
        let offset = app.world().get::<Chronotype>(entity).map_or(0.0, |c| c.0);
        let _ = writeln!(text, "{name} {offset:+.2}");
    }
    if let Err(err) = std::fs::write(out.join("chronotypes.txt"), text) {
        eprintln!("evidence: cannot write chronotypes.txt: {err}");
    }
}

/// **When the village sleeps.** Every villager every tick from game-day 1 on: how many were asleep, by clock hour
/// of the day; the clock hour each sleep began and ended at; how many ended interrupted; and how many exchanges
/// were aimed at somebody who was asleep when it resolved.
#[derive(Default)]
struct SleepLedger {
    /// Villager-ticks asleep (arrived, doing `sleep`) and villager-ticks counted, by clock hour.
    asleep: [u64; 24],
    counted: [u64; 24],
    /// Sleeps started and completed, by the clock hour of the event.
    began: [u32; 24],
    woke: [u32; 24],
    interrupted: u32,
    /// Exchanges whose target was asleep when the exchange resolved, and all exchanges.
    slept_on: u64,
    exchanges: u64,
    /// The same, per interaction: `(aimed at a sleeper, all)`.
    by_interaction: BTreeMap<String, (u64, u64)>,
}

impl SleepLedger {
    fn record(&mut self, app: &App, world: &World, live: bool) {
        if !live {
            return;
        }
        let ecs = app.world();
        let hour = ecs.resource::<SocietyClock>().hour().floor() as usize % 24;
        let defs = ecs.resource::<SocietyDefs>();
        for &entity in &world.villagers {
            self.counted[hour] += 1;
            if ecs.get::<Doing>(entity).is_some_and(|doing| {
                !doing.is_travelling() && defs.activity_name(doing.activity) == "sleep"
            }) {
                self.asleep[hour] += 1;
            }
        }
    }

    /// One resolved exchange, and whether its target was asleep.
    fn exchange(&mut self, interaction: &str, slept: bool) {
        self.exchanges += 1;
        let entry = self.by_interaction.entry(interaction.to_owned()).or_default();
        entry.1 += 1;
        if slept {
            self.slept_on += 1;
            entry.0 += 1;
        }
    }

    /// `sleep.txt`: a table of `hour asleep% began woke`, then the totals.
    fn write(&self, out: &Path) {
        let mut text = String::from("hour asleep_pct began woke\n");
        for hour in 0..24 {
            let share = 100.0 * self.asleep[hour] as f64 / self.counted[hour].max(1) as f64;
            let _ = writeln!(
                text,
                "{hour:02} {share:6.1} {:5} {:5}",
                self.began[hour], self.woke[hour]
            );
        }
        let began: u32 = self.began.iter().sum();
        let woke: u32 = self.woke.iter().sum();
        let _ = writeln!(
            text,
            "\nsleeps started {began}, completed {woke}, interrupted {} ({:.1}% of those that ended)",
            self.interrupted,
            100.0 * f64::from(self.interrupted) / f64::from((woke + self.interrupted).max(1))
        );
        let _ = writeln!(
            text,
            "exchanges aimed at somebody asleep: {} of {} ({:.2}%)",
            self.slept_on,
            self.exchanges,
            100.0 * self.slept_on as f64 / self.exchanges.max(1) as f64
        );
        for (name, (slept, all)) in &self.by_interaction {
            let _ = writeln!(text, "  {name}: {slept} of {all}");
        }
        // The widest hour: how much of the village sleeps at the peak and at the trough, and how many hours are
        // neither (a step has none; a spread has several).
        let shares: Vec<f64> = (0..24)
            .map(|hour| 100.0 * self.asleep[hour] as f64 / self.counted[hour].max(1) as f64)
            .collect();
        let mixed = shares.iter().filter(|s| **s > 10.0 && **s < 90.0).count();
        let _ = writeln!(
            text,
            "asleep share at the peak {:.1}%, at the trough {:.1}%; clock hours with between 10% and 90% asleep: {mixed}",
            shares.iter().copied().fold(0.0, f64::max),
            shares.iter().copied().fold(100.0, f64::min)
        );
        if let Err(err) = std::fs::write(out.join("sleep.txt"), text) {
            eprintln!("evidence: cannot write sleep.txt: {err}");
        }
    }
}

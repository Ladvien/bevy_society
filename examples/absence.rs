//! **The absence harness** (issue #35). A party of two leaves a six-person town for three days and
//! comes back; this writes what the people who stayed felt.
//!
//! ```text
//! cargo run --release --example absence -- --seed 7 --out docs/loop/runs/<run>/evidence/after
//! ```
//!
//! - `series.csv` — one row per six game-minutes: the distress each holder feels toward each away
//!   person, the holder's pleasure, and the reunion joy.
//! - `narrative.txt` — the absence as readable lines for one holder.
//! - `summary.json` — the figures a log entry quotes.
//!
//! `tools/plot_absence.py` turns the series into `absence.png`.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Duration;

use bevy::MinimalPlugins;
use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_society::{
    Away, EmotionKind, Emotions, Mood, PersonReturned, SocietyClock, SocietyDefs, SocietyPlugins,
    SocietySettings, TieLabel, Ties,
};

const TICKS_PER_DAY: f64 = 64.0;

fn main() {
    let mut seed = 7u64;
    let mut out = PathBuf::from("absence-evidence");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => seed = args.next().and_then(|v| v.parse().ok()).unwrap_or(seed),
            "--out" => out = args.next().map(PathBuf::from).unwrap_or(out),
            other => eprintln!("ignoring {other}"),
        }
    }
    std::fs::create_dir_all(&out).expect("the output directory is writable");

    let defs = SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.2)],
            activities: [(
                id: "wander",
                target: Wander(1.0),
                duration_hours: 1.0,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
            )],
        )"#,
    )
    .expect("the definitions parse");
    let settings = SocietySettings {
        tick_secs: 1.0 / TICKS_PER_DAY as f32,
        seed,
        ..SocietySettings::default()
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / TICKS_PER_DAY,
        )))
        .insert_resource(settings)
        .insert_resource(defs);
    #[cfg(feature = "dialogue")]
    app.add_plugins(
        SocietyPlugins
            .build()
            .disable::<bevy_society::DialoguePlugin>(),
    );
    #[cfg(not(feature = "dialogue"))]
    app.add_plugins(SocietyPlugins);
    app.finish();
    app.cleanup();

    let names = ["Ada", "Bram", "Cole", "Dena", "Edda", "Finn"];
    let people: Vec<Entity> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            app.world_mut()
                .spawn((
                    bevy_society::Person,
                    Name::new((*name).to_owned()),
                    Transform::from_xyz(i as f32 * 1.5, 0.0, 0.0),
                ))
                .id()
        })
        .collect();
    app.update();
    app.update();
    // Ada and Bram are the party. Cole is Ada's closest friend, Dena a fond acquaintance of Bram's,
    // Edda a friend of Ada's at 0.3, Finn knows neither.
    let tie = |app: &mut App, a: Entity, b: Entity, level: f32| {
        for (from, to) in [(a, b), (b, a)] {
            let mut ties = app.world_mut().get_mut::<Ties>(from).expect("ties");
            let t = ties.entry_with(to, level);
            t.affinity = level;
            t.familiarity_peak = 0.9;
            t.label = TieLabel::Friend;
        }
    };
    tie(&mut app, people[2], people[0], 0.95);
    tie(&mut app, people[3], people[1], 0.6);
    tie(&mut app, people[4], people[0], 0.3);
    let party = [people[0], people[1]];
    let holders = [people[2], people[3], people[4], people[5]];

    let mut series = String::from("day,holder,away,distress,joy,pleasure\n");
    let mut narrative = String::new();
    let start = app.world().resource::<SocietyClock>().now_days;
    let leave_at = start + 0.5;
    let return_at = leave_at + 3.0;
    let end = return_at + 1.5;
    let mut left = false;
    let mut back = false;
    let mut returned_days = None;
    let mut peak = [0.0f32; 4];
    let mut last_hour = -1i64;
    loop {
        let now = app.world().resource::<SocietyClock>().now_days;
        if now >= end {
            break;
        }
        if !left && now >= leave_at {
            for &p in &party {
                app.world_mut().entity_mut(p).insert(Away);
            }
            left = true;
            let _ = writeln!(
                narrative,
                "day {:.2}  Ada and Bram leave town for three days",
                now - start
            );
        }
        if left && !back && now >= return_at {
            for &p in &party {
                app.world_mut().entity_mut(p).remove::<Away>();
            }
            back = true;
        }
        app.update();
        let returns: Vec<PersonReturned> = app
            .world_mut()
            .resource_mut::<Messages<PersonReturned>>()
            .drain()
            .collect();
        for ret in returns {
            returned_days = Some(ret.days_away);
            let who = app
                .world()
                .get::<Name>(ret.person)
                .map_or("?", |n| n.as_str())
                .to_owned();
            let _ = writeln!(
                narrative,
                "day {:.2}  {who} is back after {:.2} days",
                now - start,
                ret.days_away
            );
        }
        let hour = ((now - start) * 24.0 / 0.5) as i64;
        let sample = hour != last_hour;
        last_hour = hour;
        for (h, &holder) in holders.iter().enumerate() {
            let emotions = app.world().get::<Emotions>(holder).expect("emotions");
            let mood = app.world().get::<Mood>(holder).expect("mood");
            for (a, &away) in party.iter().enumerate() {
                let find = |kind| {
                    emotions
                        .0
                        .iter()
                        .find(|e| e.kind == kind && e.toward == Some(away))
                        .map_or(0.0, |e| e.intensity)
                };
                let distress = find(EmotionKind::Distress);
                let joy = find(EmotionKind::Joy);
                peak[h] = peak[h].max(distress);
                if sample {
                    let _ = writeln!(
                        series,
                        "{:.4},{},{},{distress:.4},{joy:.4},{:.4}",
                        now - start,
                        names[2 + h],
                        names[a],
                        mood.pleasure
                    );
                }
                if h == 0 && a == 0 && sample && (left && now < return_at + 1.0) {
                    let hours = (now - start) * 24.0;
                    if (hours % 12.0) < 0.45 {
                        let _ = writeln!(
                            narrative,
                            "day {:.2}  Cole misses Ada: distress {distress:.2}, joy {joy:.2}, pleasure {:.2}",
                            now - start,
                            mood.pleasure
                        );
                    }
                }
            }
        }
    }
    std::fs::write(out.join("series.csv"), series).expect("write series");
    std::fs::write(out.join("narrative.txt"), narrative).expect("write narrative");
    let summary = format!(
        "{{\n  \"seed\": {seed},\n  \"days_away_reported\": {},\n  \"peak_distress\": {{\"Cole (start 0.975)\": {:.4}, \"Dena (start 0.8)\": {:.4}, \"Edda (start 0.65)\": {:.4}, \"Finn (no tie)\": {:.4}}}\n}}\n",
        returned_days.map_or("null".to_owned(), |d| format!("{d:.4}")),
        peak[0],
        peak[1],
        peak[2],
        peak[3]
    );
    std::fs::write(out.join("summary.json"), summary).expect("write summary");
}

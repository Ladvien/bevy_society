//! THROWAWAY measurement: how long does a rival label last, and what releases it?
//!
//! Deleted after the iteration records its numbers. Not a permanent test.

mod common;

use bevy::prelude::*;
use bevy_society::{PersonIndex, SocietySettings, TieLabel, TieLabelChanged, Ties};

fn village(seed: u64) -> App {
    let mut app = common::village::village(common::village::village_defs(), seed, 24);
    let settings = SocietySettings {
        seed,
        seconds_per_day: 64.0,
        start_hour: 15.0,
        tick_secs: 1.0 / 64.0,
        tuning: bevy_society::Tuning {
            travel_timeout_hours: Some(4.0),
            ..Default::default()
        },
        ..SocietySettings::default()
    };
    app.insert_resource(settings);
    app
}

#[test]
fn measure_rival_spells() {
    let seeds = [7u64];
    let ticks = 12000;
    let band: f32 = std::env::var("RIVAL_BAND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    for seed in seeds {
        let mut app = village(seed);
        // (from,to) -> (start tick, affinity at start)
        let mut open: std::collections::BTreeMap<(Entity, Entity), (u64, f32)> =
            std::collections::BTreeMap::new();
        let mut spells: Vec<(u64, u64, f32, f32)> = Vec::new();
        let mut transitions_over: Vec<f32> = Vec::new();
        let mut by_familiarity = 0usize;
        for _ in 0..ticks {
            app.update();
            let tick = app.world().resource::<bevy_society::SocietyClock>().tick;
            let msgs: Vec<_> = app
                .world_mut()
                .resource_mut::<Messages<TieLabelChanged>>()
                .drain()
                .collect();
            // affinity snapshot for margins
            let index = app.world().resource::<PersonIndex>().sorted.clone();
            let aff = |a: Entity, b: Entity| -> (f32, f32) {
                app.world()
                    .get::<Ties>(a)
                    .and_then(|t| t.get(b))
                    .map_or((0.0, 0.0), |tie| (tie.affinity, tie.familiarity_peak))
            };
            for m in msgs {
                if m.new == TieLabel::Rival {
                    let (a, f, _) = (aff(m.from, m.to).0, aff(m.from, m.to).1, 0);
                    open.insert((m.from, m.to), (tick, a));
                    let _ = f;
                } else if m.old == TieLabel::Rival {
                    if let Some((start, a0)) = open.remove(&(m.from, m.to)) {
                        let (a1, f1) = aff(m.from, m.to);
                        spells.push((start, tick, a0, a1));
                        transitions_over.push(a1);
                        if f1 < 0.1 {
                            by_familiarity += 1;
                        }
                    }
                }
            }
            let _ = index;
        }
        let n = spells.len();
        let mut durs: Vec<u64> = spells.iter().map(|(s, e, _, _)| e - s).collect();
        durs.sort_unstable();
        let med = durs.get(n / 2).copied().unwrap_or(0);
        let mean = if n > 0 {
            durs.iter().sum::<u64>() as f64 / n as f64
        } else {
            0.0
        };
        // ticks per game-day: 64 s/day / (1/64 s) = 4096
        let tpd = 4096.0;
        let mut margins: Vec<f32> = transitions_over.iter().map(|a| a + 0.3).collect();
        margins.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "seed {seed}: rival spells {n}, median {med} ticks ({:.3} game-days), mean {:.2} game-days",
            med as f64 / tpd,
            mean / tpd
        );
        println!(
            "  margin above the -0.30 bar at release: median {:.4}, p90 {:.4}, max {:.4}; released with familiarity < 0.1: {by_familiarity}",
            margins.get(n / 2).copied().unwrap_or(0.0),
            margins.get(n * 9 / 10).copied().unwrap_or(0.0),
            margins.last().copied().unwrap_or(0.0)
        );
    }
}

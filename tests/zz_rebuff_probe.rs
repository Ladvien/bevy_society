//! Throwaway probe (not committed): the label_hysteresis fixture at several rebuff memories.
mod common;

use bevy::prelude::*;
use bevy_society::{PersonIndex, SocietyClock, SocietySettings, TieLabel, TieLabelChanged, Ties};

fn label_sequence(app: &mut App, ticks: usize) -> Vec<(TieLabel, TieLabel)> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        app.update();
        let _ = app.world().resource::<SocietyClock>().tick;
        for m in app.world_mut().resource_mut::<Messages<TieLabelChanged>>().drain() {
            out.push((m.old, m.new));
        }
    }
    out
}

fn village(seed: u64, band: f32, memory: Option<f32>) -> App {
    let mut app = common::village::village(common::village::village_defs(), seed, 24);
    let mut settings = SocietySettings {
        seed,
        seconds_per_day: 64.0,
        start_hour: 15.0,
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = Some(4.0);
    settings.tuning.label_hysteresis = band;
    settings.tuning.rebuff_memory_hours = memory;
    app.insert_resource(settings);
    app
}

fn counts(app: &App) -> (u32, u32) {
    let (mut f, mut r) = (0u32, 0u32);
    for &(_, e) in &app.world().resource::<PersonIndex>().sorted {
        if let Some(t) = app.world().get::<Ties>(e) {
            for tie in &t.0 {
                match tie.label {
                    TieLabel::Friend => f += 1,
                    TieLabel::Rival => r += 1,
                    _ => {}
                }
            }
        }
    }
    (f / 2, r / 2)
}

#[test]
fn probe() {
    let band = bevy_society::Tuning::default().label_hysteresis;
    let memories: Vec<Option<f32>> = vec![None, Some(3.0), Some(6.0), Some(9.0), Some(12.0), Some(18.0), Some(24.0)];
    let rows: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = memories
            .iter()
            .map(|&m| {
                s.spawn(move || {
                    let mut with = village(7, band, m);
                    let mut without = village(7, 0.0, m);
                    let cw = label_sequence(&mut with, 6400);
                    let cwo = label_sequence(&mut without, 6400);
                    let dem = |c: &Vec<(TieLabel, TieLabel)>| {
                        c.iter().filter(|(o, n)| *o == TieLabel::Acquaintance && *n == TieLabel::Stranger).count()
                    };
                    let (dw, dwo) = (dem(&cw), dem(&cwo));
                    format!(
                        "memory {:?}: demotions with {dw} without {dwo} ratio {:.4}; messages {} vs {}; counts with {:?} without {:?}",
                        m,
                        dw as f32 / dwo.max(1) as f32,
                        cw.len(),
                        cwo.len(),
                        counts(&with),
                        counts(&without)
                    )
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for r in rows {
        println!("{r}");
    }
}

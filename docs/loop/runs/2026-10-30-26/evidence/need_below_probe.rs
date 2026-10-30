//! Throwaway: NeedBelow firings per person in a 12-person village over 3 game days.
mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{DefsFile, Needs, PersonId, SocietyDefs, TriggerFired};
use common::village::village;

#[test]
fn probe() {
    let env = |k: &str, d: f32| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(d);
    let below = env("PROBE_BELOW", 0.7);
    let seed = env("PROBE_SEED", 7.0) as u64;
    let people = env("PROBE_PEOPLE", 12.0) as usize;
    let days = env("PROBE_DAYS", 3.0) as usize;
    let mut file = DefsFile::starter().unwrap();
    file.merge(DefsFile::from_ron(include_str!("../examples/assets/village.society.ron")).unwrap())
        .unwrap();
    file.merge(
        DefsFile::from_ron(
            &format!(r#"(triggers: [(id: "starving", on: NeedBelow(need: "food", below: {below}), then: "")])"#),
        )
        .unwrap(),
    )
    .unwrap();
    let defs = SocietyDefs::compile(file).unwrap();
    let food = defs.need_id("food").unwrap().0 as usize;
    let mut app = village(defs, seed, people);
    let per_day = 256usize;
    let mut fired = vec![0u32; people];
    let mut per_day_fired = vec![0u32; days];
    let mut oracle = vec![0u32; people];
    let mut armed = vec![true; people];
    let mut max_in_tick = 0usize;
    let mut lo = vec![f32::MAX; people];
    let mut hi = vec![f32::MIN; people];
    let mut ticks_with_fire = 0usize;
    for u in 0..(days * per_day) {
        app.update();
        let n: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<TriggerFired>>()
            .drain()
            .collect();
        if !n.is_empty() {
            ticks_with_fire += 1;
        }
        max_in_tick = max_in_tick.max(n.len());
        for f in &n {
            if let Some(PersonId(p)) = f.actor {
                fired[p as usize] += 1;
                per_day_fired[u / per_day] += 1;
            }
        }
        let mut q = app.world_mut().query::<(&PersonId, &Needs)>();
        let rows: Vec<(u32, f32)> = q.iter(app.world()).filter(|(_, n)| !n.0.is_empty()).map(|(i, n)| (i.0, n.level(food))).collect();
        for (p, level) in &rows { let p = *p as usize; lo[p] = lo[p].min(*level); hi[p] = hi[p].max(*level); }
        for (p, level) in rows {
            let p = p as usize;
            if level < below && armed[p] {
                armed[p] = false;
                oracle[p] += 1;
            } else if level > below + 0.05 {
                armed[p] = true;
            }
        }
    }
    println!("PROBE people={people} days={days} below={below} seed={seed} ticks={}", days * per_day);
    println!("PROBE per-day firings: {per_day_fired:?}");
    println!("PROBE total firings: {}", fired.iter().sum::<u32>());
    println!("PROBE total independent crossings (oracle): {}", oracle.iter().sum::<u32>());
    println!("PROBE ticks with a firing: {ticks_with_fire}, most firings in one tick: {max_in_tick}");
    for p in 0..people {
        println!("PROBE person {p:02}: fired {:3}  crossings {:2}  food {:.2}..{:.2}", fired[p], oracle[p], lo[p], hi[p]);
    }
}

//! Throwaway: NeedBelow firings per person in a 12-person village over 3 game days.
mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{DefsFile, Needs, PersonId, SocietyDefs, TriggerFired};
use common::village::village;

#[test]
fn probe() {
    let mut file = DefsFile::starter().unwrap();
    file.merge(DefsFile::from_ron(include_str!("../examples/assets/village.society.ron")).unwrap())
        .unwrap();
    file.merge(
        DefsFile::from_ron(
            r#"(triggers: [(id: "starving", on: NeedBelow(need: "food", below: 0.25), then: "")])"#,
        )
        .unwrap(),
    )
    .unwrap();
    let defs = SocietyDefs::compile(file).unwrap();
    let food = defs.need_id("food").unwrap().0 as usize;
    let people = 12usize;
    let mut app = village(defs, 7, people);
    let per_day = 256usize;
    let mut fired = vec![0u32; people];
    let mut per_day_fired = vec![0u32; 3];
    let mut oracle = vec![0u32; people];
    let mut armed = vec![true; people];
    let mut max_in_tick = 0usize;
    let mut ticks_with_fire = 0usize;
    for u in 0..(3 * per_day) {
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
        let rows: Vec<(u32, f32)> = q.iter(app.world()).map(|(i, n)| (i.0, n.level(food))).collect();
        for (p, level) in rows {
            let p = p as usize;
            if level < 0.25 && armed[p] {
                armed[p] = false;
                oracle[p] += 1;
            } else if level > 0.30 {
                armed[p] = true;
            }
        }
    }
    println!("PROBE people={people} days=3 ticks={}", 3 * per_day);
    println!("PROBE per-day firings: {per_day_fired:?}");
    println!("PROBE total firings: {}", fired.iter().sum::<u32>());
    println!("PROBE total independent crossings (oracle): {}", oracle.iter().sum::<u32>());
    println!("PROBE ticks with a firing: {ticks_with_fire}, most firings in one tick: {max_in_tick}");
    for p in 0..people {
        println!("PROBE person {p:02}: fired {:3}  crossings {:2}", fired[p], oracle[p]);
    }
}

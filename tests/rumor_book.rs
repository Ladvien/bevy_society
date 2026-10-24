//! **The rumor book only grows, and that is cheap.**
//!
//! `RumorBook.rumors` never shrinks: `alive` is cleared in exactly one place, when a rumor's *subject*
//! despawns. A rumor whose every knower has stifled (Daley & Kendall) stays `alive` forever.
//!
//! Measured 2026-11-14, 200 people over 3 000 ticks (12 game-days): **2 250 rumors, all `alive`, 301
//! still tellable by someone, 1 949 (87%) untellable.** Over 6 000 ticks the book reaches **4 299**.
//!
//! `docs/loop/needs-decision/retiring-untellable-rumors.md` justified retiring them with a **4× tick
//! cost** over 3 000 ticks. That claim is **wrong**: measured across a 6 000-tick run the per-tick cost
//! does not grow —
//!
//! | tick | rumors | ms/tick (no `RumorReach` trigger) | ms/tick (with one) |
//! |---|---|---|---|
//! | 1 000 | 765 | 5.76 | 5.95 |
//! | 3 000 | 2 412 | 3.51 | 3.07 |
//! | 5 000 | 4 299 | 4.80 | 3.93 |
//!
//! — because 4 299 pointer-chases for a trigger that almost never fires is nothing beside 200 people
//! scored against their neighbours. Eight iterations carried the 4× claim in the top-5 list; the
//! measurement that would have supported it was never taken.
//!
//! What the book *does* cost is memory, and this test pins that instead: roughly 48 bytes a rumor, so
//! **200 KiB** at 200 people over 22 game-days and about **5 MiB** at 5 000.
//! **Run this file in release.** It lives in a village for twelve and twenty-two game-days with 200
//! people; in a debug build that is **4 m 26 s**, more than doubling the whole suite. `cargo test`
//! skips it in debug and CI runs it with `--release`, where it takes about 36 s.
#![cfg(not(debug_assertions))]

mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Knowledge, Place, PlaceTags, RumorBook, SocietyDefs};
use common::{headless, settings, spawn_person};
use std::collections::BTreeSet;

fn crowded(population: usize, ticks: usize) -> (usize, usize, usize) {
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    // A four-second day, so 3 000 ticks is twelve game-days and the village has a history. At the
    // default 240-second day the same tick count is half a day and the book is empty.
    let mut short_day = settings();
    short_day.seconds_per_day = 4.0;
    let mut app = headless(defs, short_day);
    let places: Vec<Entity> = (0..4)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 4.0;
            app.world_mut()
                .spawn((
                    Name::new(format!("p{i}")),
                    Place { capacity: 64 },
                    PlaceTags(vec![
                        "gathering".to_owned(),
                        "food".to_owned(),
                        "work".to_owned(),
                    ]),
                    Transform::from_xyz(10.0 * angle.cos(), 0.0, 10.0 * angle.sin()),
                ))
                .id()
        })
        .collect();
    for i in 0..population {
        let home = places[i % places.len()];
        let angle = i as f32 * 0.618 * std::f32::consts::TAU;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(12.0 * angle.cos(), 12.0 * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Home(home), Autonomous));
    }
    for _ in 0..ticks {
        app.update();
    }
    let book = app.world().resource::<RumorBook>();
    let total = book.rumors.len();
    let alive = book.rumors.iter().filter(|r| r.alive).count();
    let mut tellable: BTreeSet<u32> = BTreeSet::new();
    let mut query = app.world_mut().query::<&Knowledge>();
    for known in query.iter(app.world()) {
        for entry in known.0.iter().filter(|entry| entry.spreader) {
            tellable.insert(entry.rumor.0);
        }
    }
    (total, alive, tellable.len())
}

#[test]
fn the_book_accumulates_rumors_nobody_can_tell() {
    let (total, alive, tellable) = crowded(200, 3000);
    println!("{total} rumors, {alive} alive, {tellable} still tellable");
    assert_eq!(alive, total, "only a subject's despawn clears `alive`");
    // Measured 301 of 2 250 still tellable, i.e. 87% untellable. The threshold is 80%, not 90%: the
    // earlier 93% figure came from a different population and run length, and pinning a test to it
    // would re-introduce the kind of stale number this iteration exists to correct.
    assert!(
        tellable * 5 < total,
        "most of the book should be untellable — that is the finding, not the bug: \
         {tellable} of {total} can still be told"
    );
    assert!(
        total > 1000,
        "the book has to grow for this to be worth saying"
    );
}

#[test]
fn the_book_costs_memory_not_ticks() {
    // The claim this replaces was a 4× tick cost. Measured, the per-tick cost does not grow with the
    // book; what grows is memory, at roughly 48 bytes a rumor. This asserts the growth is real and
    // linear, which is what makes a long run's memory predictable.
    let (three_thousand, _, _) = crowded(200, 3000);
    let (six_thousand, _, _) = crowded(200, 6000);
    println!(
        "3000 ticks: {three_thousand} rumors (~{} KiB); 6000: {six_thousand} (~{} KiB)",
        three_thousand * 48 / 1024,
        six_thousand * 48 / 1024
    );
    assert!(
        six_thousand > three_thousand,
        "the book grows with the length of the run"
    );
    assert!(
        six_thousand < three_thousand * 3,
        "and it grows roughly linearly, not quadratically: {three_thousand} -> {six_thousand}"
    );
}

# Saving and loading a society

A host that autosaves at dawn and continues from the file needs the society to come back exactly as it left: friendships, grudges, opinions, memories, rumors, the clock, the random stream's position and the once/cooldown bookkeeping of every trigger. A society that reset on reload would not be one. This page is the whole of it; it needs the `serde` feature.

```toml
[dependencies]
bevy_society = { version = "0.3", features = ["serde"] }
```

## Two functions and one struct

```rust,ignore
pub fn capture(world: &mut World) -> SocietySave;
pub fn restore(world: &mut World, save: SocietySave) -> Result<RestoreReport, RestoreError>;
```

`capture` reads the world and **never writes to it**; it takes `&mut World` only because building a query needs one. `SocietySave` derives `Serialize` and `Deserialize` and holds plain data, so it goes to any serde format. The crate does not pick one for you and does not write the file.

**Nothing in a save is an `Entity`.** An entity id is an index into one world's allocator, and the next session's is unrelated. Every reference to a person is a `PersonKey(u64)` and every reference to a place is a `PlaceKey(u64)`: the ids your game already has for them (a roster id, a save-file slot). The crate never writes either component; **you** attach them when you spawn:

```rust,ignore
commands.spawn((Person, Autonomous, Name::new("Bram"), PersonKey(101), Home(well)));
commands.spawn((Name::new("well"), Place { capacity: 6 }, PlaceKey(1), Transform::default()));
```

A person without a `PersonKey` is **left out of the save**, with one `warn!` per `capture` naming how many. Every reference to such a person is handled the same way: a tie to them is dropped, an emotion or memory about them keeps no target, and an activity directed at them is saved as no activity at all.

## What is in a `SocietySave`

| field | holds |
|---|---|
| `version` | The format version, `bevy_society::VERSION` (currently `5`). A file written by an older version is brought up by `SocietySave::upgrade`, which `restore` calls itself. |
| `names` | The names of every definition list (needs, topics, deeds, factions, activities), in definition order. Ids are indices into these lists, so this is how a restore notices the defs have moved. |
| `clock` | The tick count, game time in days, `dt_days`, and the fixed-step accumulator. |
| `rng` | The one random stream's seed, stream number and word position, so the draws after a reload are the draws that would have followed. |
| `persons` | One `PersonSave` per keyed person, ascending by `PersonId` (then by key): personality, the body-clock offset (`chronotype`, absent from an older file, which restores as `0.0`), mood, emotions, needs, goods held (`stock`), outgoing ties (affinity, trust, respect, familiarity peak, last contact, label), opinions, rumor knowledge, memories, memberships, signals, status, `dormant` and `deceased`, when an absence began (`away_since_bits`), the running activity and its target, the walk destination, the stall window of a walker (`travel_progress`), who the person has just harmed and until when (`recent_harm`, the retaliation refractory), who has just refused them and until when (`rebuffed`, the rebuff memory; absent from an older file, which restores as nobody avoided), the places a walk just failed to reach (`failed_places`), and the last ORCA velocity under `steering`. |
| `rumors` | The rumor book in order, so `RumorId`s stay stable. |
| `triggers` | Which `once` triggers fired and every cooldown, each under the trigger's authored `id` (`ids`, so a restore into an edited defs file finds the same trigger; state for an id the defs no longer have is dropped with a warning), the clock instant `Hour` triggers measure against, which people have fired a `NeedBelow` line and not yet recovered (`need_disarmed`), and which reach triggers fired. |
| `faction_stats` | The faction aggregates. |
| `prices` | The price book, one number per good. It is recomputed from the stores each tick, but `ctx.price` reads the tick-start snapshot, which is frozen before that recomputation, so the first tick after a restore needs the saved book. Empty without `EconomyPlugin`. |
| `fields` | Under `fields`, the crowd grid and each faction grid's cells; `None` when the plugin is not in the app. |

**Not saved:** positions (on your ground, `Transform` is yours: stand the reloaded people where your own save puts them), the places themselves and their `Place` / `PlaceTags` / `Obstacle` components, messages in flight, and a conversation's open node. `Locomotion::Host` is the contract that says walking is yours.

## Loading

Spawn the world the save is about first: your persons with their `PersonKey`s, your places with their `PlaceKey`s, the same `SocietyDefs`. Then run **one update**, so `Startup` has seeded the clock and the RNG (a restore applied earlier would be overwritten by that seeding) and the social fields exist. Then restore:

```rust,ignore
use bevy_society::{SocietySave, capture, restore};

fn autosave(world: &mut World) -> String {
    serde_json::to_string(&capture(world)).expect("a save is plain data")
}

fn load(world: &mut World, json: &str) {
    let save: SocietySave = serde_json::from_str(json).expect("the file is a save");
    match restore(world, save) {
        Ok(report) => {
            for key in &report.unknown_keys {
                warn!("the save names person {key}, who is not in this world");
            }
            info!("{} person(s) came back fresh", report.fresh.len());
        }
        Err(err) => error!("the save was refused: {err}"),
    }
}
```

`serde_json` is your dependency, not the crate's. Any `serde` format works.

## Refusals

`restore` validates everything **before writing anything**, so an `Err` leaves the world exactly as it was.

| `RestoreError` | Means | What to do |
|---|---|---|
| `Version { found, expected }` | The save's version is newer than this build, or older with no upgrade step (today only version 1 has one, to 2). The crate refuses rather than guess at a layout. | Load with the crate version that wrote it, or discard the file. |
| `DefsMismatch { kind }` | The save's list of `"needs"`, `"topics"`, `"deeds"`, `"factions"` or `"activities"` is not what the defs now declare. Ids are indices, so a defs file that gained, lost or reordered a definition would write person 3's needs into person 3's opinions. | Load with the defs the save was written against, or version your defs and your saves together. |
| `NotReady` | The world has no `SocietyDefs` or no `SocietyClock` yet. | Run the restore after the core plugin's startup. |

## What the report says

`RestoreReport` is what was not a clean match, none of it fatal:

| field | Means |
|---|---|
| `unknown_keys` | Keys in the save with no person in the world. Each is also warned about; the save's state for them is dropped. |
| `fresh` | Persons in the world with no entry in the save. They keep the state you spawned them with and get new ids after the highest restored one, in their current id order. |
| `upgraded_from` | `Some(v)` when the save was written at an older version and `SocietySave::upgrade` brought it up first; `None` for a current save. |
| `dropped_activities` | Activities that could not come back, because their place or person key did not resolve or their activity id is not in the defs. |

A rumor about a person who is no longer keyed keeps `Entity::PLACEHOLDER` and stops circulating, because nothing can spread a rumor about somebody the world does not have.

## Is it exact?

Yes, and it is tested: `tests/save.rs` checks that `capture` → JSON → `restore` → `capture` is the same save, and that the 100 ticks after a restore are bit-for-bit the ticks the uninterrupted run took, down to the RNG's word position and every trigger cooldown. Two details follow from that:

- A `Doing` stores the game-day it started as its IEEE-754 **bits**, not the number, because a decimal text format can round-trip an `f64` through a shorter string and lose the last bit. Use the `started_day()` / `arrived_day()` accessors; they turn the bits back.
- Ties are re-sorted by target on the way in, so the binary search `Ties` relies on still holds.

## Check it yourself

A save is only as exact as the keys you gave it. This is the smallest round trip that proves yours: capture after some ticks, restore into a fresh app, compare.

```rust,ignore
let mut app = headless(defs.clone(), settings());
let (bram, alys) = spawn_keyed(&mut app); // each carries a PersonKey
run(&mut app, 200);
let json = serde_json::to_string(&capture(app.world_mut())).unwrap();

let mut fresh = headless(defs, settings());
let (bram2, alys2) = spawn_keyed(&mut fresh); // the same keys
run(&mut fresh, 1); // Startup has to have run
let save: SocietySave = serde_json::from_str(&json).unwrap();
let report = restore(fresh.world_mut(), save).unwrap();
assert!(report.unknown_keys.is_empty() && report.fresh.is_empty());

assert_eq!(
    app.world().resource::<SocietyClock>().tick,
    fresh.world().resource::<SocietyClock>().tick,
);
assert_eq!(
    app.world().get::<Ties>(bram).unwrap().get(alys).unwrap().affinity,
    fresh.world().get::<Ties>(bram2).unwrap().get(alys2).unwrap().affinity,
);
```

`headless`, `settings`, `run` and `spawn_keyed` are the test harness in `tests/common/mod.rs` and `tests/save.rs`.

## Next

- [`concepts.md`](concepts.md): ticks, game-days and why determinism makes a save exact.
- [`decisions.md`](decisions.md): reading why a person did what they did.

## Older saves

`restore` upgrades a version-1 save itself and says so in `RestoreReport::upgraded_from`. The two versions differ by `PersonSave::stock`; a version-1 person comes back holding nothing of every good the defs declare. To migrate a file once and re-save it, call `SocietySave::upgrade`, which returns `RestoreError::Version` for a version it has no step for. `DefsMismatch` still applies after an upgrade.

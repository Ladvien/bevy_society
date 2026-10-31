# In an editor

Six functions exist so an editor (or a test harness) can drive a society without a window: run it forward, report what changed, put it back, dry-run one trigger, check a script before saving it, and find out who depends on a building. Each works on a `&mut World` between updates, never from inside a system.

## Run a society forward: `fast_forward`

```rust,no_run
# use bevy::prelude::*;
# use bevy_society::{FastForward, fast_forward};
# fn f(world: &mut World) -> Result<FastForward, bevy_society::FastForwardError> {
let run: FastForward = fast_forward(world, 1.0)?; // one game-day
println!("{} ticks, day {} to {}", run.ticks, run.from_days, run.to_days);
# Ok(())
# }
```

`fast_forward(world, days)` runs `days` game-days of society ticks in one call, driving the same `FixedMain` schedule a live app does and rotating the message buffers between steps, so the society it leaves is the one the host would have after the same number of ticks — same seed, same inputs, bit for bit. The tick count is `ceil(days * seconds_per_day / tick_secs)`, and `0.0` runs none.

It refuses, rather than guessing: `NoSociety` (no `SocietyClock`/`SocietySettings`), `NotStarted` (no `Startup` yet — update the app once), `BadDays` (negative, not finite, or so large the tick count overflows a `u64`), `HostClock` (`ClockSource::Host` belongs to the host), and `NoTimeStep` (no fixed clock, or a zero timestep). A host's own `Update` systems do not run, and the app's real-time clocks are left as the last frame left them.

## Report and reset: `preview_forward`

`preview_forward(world, days)` is `capture` + `fast_forward` + `capture` + `restore`, with the persons' `Transform`s put back too. It answers "what does a day do to this village?" and then puts the village back:

| field | what it is |
|---|---|
| `Preview::run` | the `FastForward` that happened |
| `Preview::persons` | per person, in `PersonKey` order: `from`/`to` plane position, `doing_before`/`doing_after`, and every tie whose existence or affinity moved |
| `Preview::dropped_activities` | activities the reset could not put back; those people restart idle |

**Every person needs a `PersonKey`** (the host's own id for them). A society with an unkeyed person is refused with `PreviewError::Unkeyed(n)` **before anything runs**, because a reset could not bring them back. `PreviewError::Run` also touches nothing; `PreviewError::Reset` means the run happened and the save could not be written back, so the world is left advanced. What a `SocietySave` does not hold — everything the host owns — is not reset.

## Dry-run a trigger: `test_trigger`

`test_trigger(world, "watch_stops_theft", Some(actor), Some(target))` evaluates one authored trigger against the society the **last** tick froze, as if its event had just happened, and reports `TriggerTest`:

- `selected` — the actor passes the trigger's `factions`/`persons` filter;
- `in_area` — the actor stands inside its `area`;
- `condition` — whether the `if` script held (`None` when there is no `if`, or an earlier check already refused it);
- `would_fire` — selected, in area, `if` held, `then` ran cleanly. `once`/cooldown bookkeeping is **not** consulted: this asks about the script, not the history;
- `effects` — each write the `then` queued, in order, as text such as `Adjust { person: 0, other: 1, field: "trust", delta: 0.2 }`.

Nothing changes: no state is spent, nothing is queued, no message is written, and the event the trigger listens for need not have happened. Run a `fast_forward` first, then test the trigger, to see how an edited script behaves on the society it will run in. `TriggerTestError::NoSnapshot` says no tick has run yet.

## Who is in a place, and who is in a box

- `place_dependents(world, place)` returns `PlaceDependent { person, name, role }` for everyone whose `Home` or `BoundPlaces` entry is that place, in `PersonId` order, one row per role (`"home"`, `"work"`). An editor refuses to delete a building someone lives or works at, and **names them**.
- `persons_in_rect(world, min, max)` returns the entities whose society-plane position lies inside the rectangle, in `PersonId` order: the selection a bulk edit or a trigger's `area` binds to.

Both take `&World` and never change it.

## The rest of an editor's needs

| what | where |
|---|---|
| places and persons as RON, spawned and captured | [`authoring.md`](authoring.md#placing-persons) — `PlacementFile`, `Placed` |
| edited definitions into a running society | [`scripting.md`](scripting.md#editing-a-running-society) — `reload_defs` |
| script text checked before it is saved | [`scripting.md`](scripting.md#checking-text-before-it-is-saved) — `check_script` |
| which conversation rows dangle | [`dialogue.md`](dialogue.md#checking-the-two-files-agree) — `dialogue_problems` |
| a trigger bound to an area, faction or person | [`authoring.md`](authoring.md#triggers) — `area`, `factions`, `persons` |
| which triggers fired | [`scripting.md`](scripting.md) — the `TriggerFired` message |
| save and restore | [`persistence.md`](persistence.md) |

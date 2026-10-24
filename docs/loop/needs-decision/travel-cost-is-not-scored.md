# Should every activity weigh the walk to reach it?

## The problem, measured

`Signal::Distance` is only penalised where a definition asks for it. At `village-evening`'s settings — 64
s a day, walking `1.4 / 64` m a tick, a village 8.2 m across — a crossing walk is **375 ticks**, and:

| activity | duration, ticks | gates on `Distance`? |
|---|---|---|
| `snub` | **17** | no |
| `compliment` | **34** | no |
| `chat` | **51** | yes |
| `gossip` | **51** | yes |
| `play` | **85** | yes |
| `eat` | **128** | no |
| `relax` | **256** | no |
| `work` | 683 | no |
| `sleep` | 1 365 | no |

**Five activities are shorter than the walk that reaches them, and three of those five are the only ones
that consider distance at all.** The consequence, measured over 17 game-days of `village-evening`:

| | share of all person-ticks |
|---|---|
| **travelling** | **62.5%** |
| doing an activity | 37.5% |
| of which `sleep` | **14.7%** — against 33.3% that one 8-hour sleep a day allows |

A villager spends most of its life walking to things that are over before it arrives.

## Why it is a decision

The fix is a **definitions-file** change: which activities score `Distance`, and with what curve. That is
the file a host ships and edits, and the crate's rule is that persons, needs and activities are data. Adding
a consideration to `eat` or `relax` changes what a host's own pack does the moment it upgrades.

## Options

### 1. Every non-`Person` activity scores `Distance`

`eat`, `relax` and `work` gain `(input: Distance, curve: Linear(-1.0, 1.0))`.

- **For:** the decision then reflects what a use costs, which is the missing term.
- **Against:** it is a schema-visible change to the shipped pack, and `work` has a `Hours(8, 17)` gate that
  already limits when it can be chosen.

### 2. The decision scores travel cost for every activity, from the crate

A new consideration computed from the target's distance and the activity's duration — effectively
`duration / (duration + walk)` — available to any definition that wants it.

- **For:** the term exists once instead of being copied into each activity, and a host gets it by naming it.
- **Against:** a new `Signal` variant is a definitions-file schema change, and the crate gains a
  consideration it currently derives from data alone.

### 3. The shipped pack's short activities are lengthened

`snub` 0.1 → 1.0 h, `compliment` 0.2 → 1.0, `chat` and `gossip` 0.3 → 1.0.

- **For:** no new consideration. An activity that lasts longer than the walk is worth the walk.
- **Against:** it changes what a greeting *is*. A 1-hour `snub` is not a snub.

## Recommendation

**Option 2**, in a later iteration. It puts the missing term in one place, it is opt-in by name so no
existing definition changes behaviour, and it composes with option 1 if a host wants it everywhere.

Measured while exploring: adding `Distance` to `eat`, `relax` and `compliment` takes travelling from
**62.5% to 50.5%** and the total need mean from 0.526 to 0.513. Twelve points of walking for 1.3% of need —
a fair trade, and a small one. The bigger lever is the **decision**, not the pack: a person should prefer a
near place over a far one for the same activity, and today `Place` targets offer one candidate per matching
place with no distance in the score.

## What ships now

| item | state |
|---|---|
| `tests/travel_cost.rs` | **failing, ignored** — pins the eight short activities and the five blind to distance |
| `tests/travel.rs` | failing, ignored — the `Doing`-while-travelling labelling |
| the harness | reports `travelling:<activity>` separately from `<activity>` |
| the measurement | `docs/loop/runs/2026-11-21-01/` |

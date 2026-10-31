# Plan - `Tuning::rebuff_memory_hours`: a refusal is remembered

Ranked candidates (the issue list is empty; parked branches are older, ignored): (1) **re-approach after a refusal** (top item of the last log entry; ranked item 2, "behaviour that reads as wrong when you watch it"): 29 % of refused approaches are followed within an hour by another affiliative approach to the same person against 42 % after an accepted one; picked. (2) Satiation after *accepted* approaches (34 % of exchanges repeat the partner; six compliments in forty minutes): a different mechanism, left for the next pass. (3) The walk home (`travel-time-in-two-clocks.md`): needs a decision about two clocks, not a measured defect to fix. (4) Re-record the README clips: needs a display, this host has none. (1) beats (2) because it has a source that states the direction (Maner et al. 2007) and a measured gap against its own control row; (2) has neither.

## Design

| piece | change |
|---|---|
| `src/core.rs` `Tuning` | `rebuff_memory_hours: Option<f32>`. Doc: the mechanism, Maner et al. 2007 DOI, "design default" for the interval, `None` is the old curve, saved. Default decided by the sweep (3 / 6 / 12 / 24 h, below); a protected test that fails at the chosen default makes it `None` and the pass says so. |
| `src/behavior.rs` | `pub(crate) struct Rebuffed(Vec<(Entity, f64)>)`, beside `FailedPlaces` and `RecentHarm`. Written in `exchange`'s refusal branch (not `refused_by_rule`), only when the interaction's deed is absent or `harm >= 0`. `DecisionQueries` gains `rebuffed`. `decide_activities` drops `DoingTarget::Person(other)` in the memory for a non-harmful activity, after `targets_for` (RNG draws unchanged). |
| `src/save.rs` | `PersonSave::rebuffed: Vec<(u64, u64)>` (`#[serde(default)]`, `VERSION` unchanged: an older file restores with nothing remembered, the state it was saved without); `PersonQuery` gains `rebuffed`; capture and restore beside `recent_harm`. |
| `examples/evidence.rs` | `ReapproachLedger` (written, `reapproach.txt`), flags `--rebuff-memory H` and `--no-rebuff`. Also a `partners` count of distinct partners per person-day and the refusal rate in `summary.json` if not already there. |
| tests | `tests/rebuff.rs` (below). Existing protected pins are not edited. |
| docs | `docs/research/rebuff.md` (new, indexed in `docs/research/README.md`), `docs/guide/decisions.md` (a line on what a refusal changes), `docs/guide/persistence.md` (the saved list), `CHANGELOG.md`. |

## Tests (`tests/rebuff.rs`)

1. A person refused by their only neighbour does not choose that neighbour for a chat again until the memory has run out, and does after (behaviour at the boundary, with the clock stepped past `until`).
2. With two neighbours, a refusal from one sends the next approach to the other (Maner: reconnection is with new sources).
3. A refused `insult` writes no rebuff (harm stays under the retaliation refractory), and an accepted approach writes none.
4. `rebuff_memory_hours: None` is the old behaviour: the same refused person is chosen again at once (the control that the first test would otherwise pass by accident).
5. Feature `serde`: a save mid-memory restores into a world that still avoids the person; an older save (no `rebuffed`) loads.
6. Determinism: two runs of the same seed hash equal with the field on (the repo's existing hash helper).

## Evidence

Same seeds, `before` = this branch with `--no-rebuff` byte-equal to the `/tmp/bs71/evidence-base` binary built from HEAD (checked with `cmp` on `summary.json` and `narrative.txt`), `after` = the chosen default. Village seeds 7 / 42 / 11 / 99 (17 game-days), towns of 60 at seeds 7 and 99 (30 game-days), a 40-person town at seed 3 (20 game-days; fresh eyes). Per run: `reapproach.txt` (the table in `research.md`), refusal share of approaches, distinct partners, friend / rival ties, insults, mood. A sweep over 3 / 6 / 12 / 24 h at seed 7. The narrative for the owl (`--track 16`) before and after, for the refused-gossip run. `perf.md`: the benchmark at 1 000 and 5 000 people, ABBA order, CPU seconds.

## Out of scope

Satiation after accepted approaches; a per-interaction (rather than per-person) memory; making the refused side remember *why*; any change to the acceptance draw.

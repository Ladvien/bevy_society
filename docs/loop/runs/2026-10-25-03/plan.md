# Plan: a friendship is maintained, so a band that only holds when nothing happened

## Pick, and why it beats the runner-up

**Pick:** make the friend bar hysteretic — but **only while no event touched the pair**, so a witnessed
insult still ends the friendship and continuous drift no longer does.

**Why:** the measured problem is not "friendships end too often". It is that **a third of the endings
have no interaction behind them** (56 of 504 at seed 7, 52 of 507 at seed 42) and are announced to the
host as relationship events, while the label lives in a **0.05-wide sliver** (bar 0.30, ceiling 0.35) on
a variable that swings **0.072** at a time. A friendship held for a median of four game-hours is not a
relationship; it is a threshold being grazed.

**Runner-up:** "raise the rate of friendly interaction" (item 2 on the list). It changes what a host's
village is, needs a three-axis measurement, and would not address the endings at all — they are drift,
not scarcity.

**Why not just the previous iteration's rejected option:** last iteration rejected a band on the friend
bar because it moved rivals 42 → 48. That measurement could not separate a band that holds
unconditionally from one that holds only when nothing happened. This iteration measures the split.

## Design

### Hold the friend bar, but only against drift

`TieLabel::classify_from` already takes the current label. Extend the friend case:

```
Friend is kept while       affinity >= friend_affinity - label_hysteresis
and                        no event has touched the pair within label_grace_days
```

**"No event" is the tie's own `last_contact_day`**, which every exchange already stamps
(`Ties::entry`/`expose_toward`), so no new state is needed: an exchange with the person is exactly the
"something happened" signal. If the pair talked within `label_grace_days`, the single bar applies and a
friendship can end as it does today. If they did not, the band holds it.

That is the shape social penetration theory describes — a friendship is **maintained**, and its absence
of maintenance is what should end it — and it is measurable: the drift endings have no exchange behind
them, the event endings do.

### The two new `Tuning` fields

| field | meaning | default |
|---|---|---|
| `label_hysteresis` | exists; last iteration applied it to the stranger bar only | unchanged at 0.02 |
| `friend_band` | how far below `friend_affinity` a **quiet** friendship is held | swept in `eval.md` |
| `friend_grace_days` | how long without contact the band applies | swept in `eval.md`, chosen from the measured distribution of "hours from insult to ending" (median 0.4, so a grace window of hours, not days) |

Both are **design defaults chosen by measurement**. Neither has a source paper; the *mechanism* does
(Altman & Taylor 1973, staged and maintained relationships, corpus doc `10.33790_jmhsb1100188`).

### The constraint that must not be broken

The band must be **narrower than the sliver it guards** where possible, and it must not raise the friend
count much: `affinity_ceiling` (0.35) exists because an unbounded affinity made a 60-person town a
clique (155 friends, 0 rivals). A band that holds every friendship forever is the same failure by
another route. The sweep reports friends and rivals on three seeds and the choice is made against them.

### Touched

| file | change |
|---|---|
| `src/relationships.rs` | `classify_from` gains the grace-window case for `Friend`; `Tie::refresh_label` passes `last_contact_day`; unit tests |
| `src/core.rs` | `Tuning::friend_band`, `Tuning::friend_grace_days`, comments and defaults |
| `tests/label_hysteresis.rs` | the existing no-movement test must still hold; new cases for the band and the grace window |
| `tests/tuning_covers_rates.rs` | the two new fields are read |
| `docs/research/relationships.md`, `CHANGELOG.md`, `docs/loop/log.md`, this run record | |

### Tests

1. **Unit.** A `Friend` at `friend_affinity - friend_band/2` with no contact for longer than
   `friend_grace_days` stays a `Friend`; the same tie with contact yesterday is an `Acquaintance`.
2. **Unit.** An insult ends the friendship even if the pair have been quiet: the event path is a single
   bar, and a tested one.
3. **The metric that matters**: median held friendship duration rises; count of drift-driven endings
   falls to near zero; count of insult-driven endings does not.
4. **Friends and rivals stay within a few of baseline** on three seeds.
5. **Determinism**: same seed twice, same label sequence.

## Measurement plan

`village-evening` at its own clock, seeds 7 / 42 / 11, 67 200 ticks (16.4 game-days), through
`examples/evidence.rs` with a new `--friend-band` and `--friend-grace` flag:

- held friendship duration (median, p10, p90), before and after;
- endings attributed to insult / exchange / nothing, before and after;
- friends, rivals, `label_changes`, rumor reach, interrupted endings — the four numbers that must not
  move;
- the affinity histogram, so the sliver can be seen shrinking or not.

Plots: (a) held-duration distributions before/after; (b) endings by cause, before/after; (c) friends and
rivals across the sweep. Narrative: the tracked villager's day, and one friendship's affinity trace.

## Not doing

- **Stages** (casual / close / best as separate categories). That is a schema change to `TieLabel`, a
  message type a host consumes, so it belongs in `needs-decision/`. It is also the honest long-term
  answer and this band is the stopgap that makes it unnecessary for now.
- **Remembering a label across a period of no contact as a separate scalar.** The grace window uses
  `last_contact_day`, which already exists; adding state would need saving, and a save format change is
  a bigger decision than this iteration should make.

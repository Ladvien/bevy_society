# Plan: a label is a latitude, not a line

## Pick, and why it beats the runner-up

**Pick:** give a tie's label hysteresis — a bar to *enter* a category and a lower bar to *leave* it — so
`TieLabelChanged` reports relationship events instead of threshold noise.

**Runner-up:** item 1 on the log's list, "raise the rate of friendly interaction". Rejected for this
iteration because it is authored content that changes what a host's village *is*, it needs a three-axis
measurement before it can even be chosen, and the previous iteration already measured that the
weight-based version of the same idea does not work. This pick is a defect with a number attached: a
message a host is meant to act on fires 1 782 times in four game-days, 58% of them from a value that had
moved by a median of 0.00018 in the tick that did it.

## Design

### Two bars for two directions

`TieLabel::classify_with_trust` keeps its signature and its meaning (the *promotion* bars: what it takes
to be a Friend or a Rival). A tie that is **already** in a category leaves it only once the value has
fallen past a **lower** bar:

```
promote:  affinity >= friend_affinity                     -> Friend
demote:   affinity <  friend_affinity - label_hysteresis  -> not Friend
```

The same for Rival, and for `Stranger`: a tie already `Stranger` needs familiarity
`stranger_familiarity + label_hysteresis` to become an Acquaintance, and an Acquaintance needs it below
`stranger_familiarity - label_hysteresis` (floored at 0) to fall back.

**Where it lives.** A new `TieLabel::classify_from(current, affinity, familiarity, trust, tuning)`, which
is `classify_with_trust` when `current` is not the category being tested. `Tie::refresh_label` calls it
with its own `self.label`, so the state is the tie, not a new resource. `classify` and
`classify_with_trust` stay as the *promotion* test, and their documentation says so — `classify(0.35, …)`
remains a Friend, which is what every existing test and the exposure-equilibrium printout ask.

### The new `Tuning` field

`label_hysteresis: f32` — **design default, chosen by measurement**, documented as such, because no
paper gives a number (Sherif & Hovland 1961 supplies the mechanism: a latitude of acceptance, not a
line; see `research.md`).

Default: whatever the sweep in `eval.md` picks, stated there with its table. `0.0` restores the current
behaviour exactly, which is the fallback if the sweep finds nothing worth having.

Why affinity-scaled rather than a separate field per label: one number is one thing a host can tune, and
`friend_affinity` is 0.30 with the ceiling at 0.35, so the *band* has to be a fraction of that range, not
an absolute familiarity. A second field for the stranger bar would be a second number with the same
justification and half the evidence; if the sweep shows the stranger end needs its own width, that becomes
a `needs-decision` item rather than a shipped guess.

### Touched

| file | change |
|---|---|
| `src/relationships.rs` | `TieLabel::classify_from`, `Tie::refresh_label` uses it, unit tests |
| `src/core.rs` | `Tuning::label_hysteresis` with its comment and default |
| `tests/tuning_covers_rates.rs` | the new field is a named rate and is read |
| `tests/label_hysteresis.rs` | **new**: churn falls, counts do not move, determinism |
| `docs/research/relationships.md` | the mechanism and its citation |
| `CHANGELOG.md`, `docs/loop/log.md`, this run record | |

### Tests

1. **Unit.** A Friend whose affinity dips to `friend_affinity - ε` stays a Friend; the same tie built
   fresh at that affinity is an Acquaintance. Symmetric for Rival and Stranger.
2. **`label_hysteresis = 0.0` is byte-identical** to the shipped behaviour: same label sequence on a
   4200-tick village run.
3. **The message count falls** on `village-evening` at seed 7: `TieLabelChanged` events drop by at least
   half, and at least half of what remains is accompanied by a one-tick affinity step above the band.
4. **The counts do not move**: friends and rivals at the end of a 4 200-tick run are within ±3 of the
   baseline at the same seed. Hysteresis that changes how many friends exist is not hysteresis.
5. **Determinism.** Two runs at the same seed produce the same label sequence and the same final counts.

## Measurement plan

- Baseline: `village-evening` seeds 7, 42, 11, 4 200 ticks, at the scenario's own settings.
- Sweep `label_hysteresis` over 0.00 / 0.02 / 0.04 / 0.08 and report for each: label changes, the
  fraction caused by a sub-band step, friends, rivals.
- Plots: (a) a tie's affinity against its label over time for the pair that changes most, before and
  after, so the band is visible as two lines rather than one; (b) label changes per game-day across the
  sweep; (c) the friend/rival counts across the sweep.
- Narrative: `examples/evidence.rs`'s day file, read for whether a "friend" now stays a friend.

## Not doing

- **Assimilation** (the band moving toward repeated experience). It is the other half of social judgment
  theory and it would make the bars themselves state. Recorded in `research.md`, not built.
- **A separate width for the stranger band.** Would be a second unsourced number. If the sweep says the
  low end needs it, it becomes a decision.
- **Debouncing the message.** Changes when the host hears about a change, not whether there was one.

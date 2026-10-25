# Plan: let a dyad resolve toward reciprocity

## Pick, and why it beats the runner-up

**Pick:** give the **dyad** the balance step it has always been missing. Each tick, pull `aff(A→B)` and
`aff(B→A)` a little toward each other, at a named rate.

**Why:** Heider's balance theory says "in a dyadic relation, balance basically means reciprocity: feeling
positive about a person who doesn't reciprocate this feeling is unbalanced" (corpus `10.18564_jasss.5323`).
The crate implements the **triadic** half of the same theory (`src/model/balance.rs`, "a friend of a friend
is pulled toward friendship") and not the dyadic half. Measured consequence: **50% of directed friend ties
are unreciprocated**, the two directions disagree by **0.145** on a scale whose friend band is 0.05, and the
decision model aims approaches using the actor's tie while the acceptance model decides with the target's —
so **38.9% of all approaches are refused**, 45-48% when aimed at an acquaintance.

**Runner-up:** tuning `accept.base` down, or adding `Trust`/`Familiarity` considerations to `compliment`.
Both treat the symptom: they change what a person does with an asymmetric tie instead of making the tie
mutual. They also move a number a host has already tuned.

**Why not last iteration's item 1** (`last_exchange_day` on `Tie`): it needs a `SocietySave::VERSION` bump
and is recorded; this needs no new state at all.

## Design

### The dyadic balance step

In `src/model/balance.rs`, beside `triad_delta`:

```rust
/// Pull two directed affinities toward their mean: `(b − a) · rate · dt` added to `a`.
pub fn dyad_delta(rate_per_day: f32, dt_days: f32, affinity_ab: f32, affinity_ba: f32) -> f32
```

Applied in a new system in `SocietySystems::Relate`, iterating `PersonIndex::sorted` and each tie once per
**unordered** pair (`a < b` by `Entity::to_bits`, so a pair is handled once and in a fixed order).

**Both directions move**, which is what balance theory says: the pair converges on their mean rather than one
being dragged. Convergence is `(1 − rate·dt)` per tick toward the mean, so it never overshoots and both
directions meet.

### The new `Tuning` field

`dyad_balance_per_day: f32` — **design default, chosen by measurement**, no source paper gives a rate,
because Heider's statement is qualitative. `0.0` disables the step exactly.

It must be **slower than `affinity_relax_per_day`'s effect on the labels** is fast: a reciprocity step strong
enough to flatten the graph destroys the labels (every tie tends to the population mean and `Friend`/
`Rival` both become rare). The sweep reports friend/rival counts and the label-message count, and the choice
is made against them.

### The one behaviour change to watch

If actors like people who like them back, then `compliment`'s `Affinity` gate fires when the *relationship is
mutual* — and the refusal rate should fall without touching the acceptance model at all. That is the
downstream evidence the change is behavioural rather than cosmetic, and it is measured.

### Touched

| file | change |
|---|---|
| `src/model/balance.rs` | `dyad_delta`, unit test (mean-seeking, never overshoots, symmetric) |
| `src/relationships.rs` | `dyad_balance` system in `Relate`, after `refresh_labels` |
| `src/core.rs` | `Tuning::dyad_balance_per_day` with its comment and default |
| `tests/reciprocity.rs` | **new**: symmetry rises, counts do not explode, refusal rate falls, determinism |
| `tests/tuning_covers_rates.rs` | the field is a named rate and is read |
| `docs/research/relationships.md`, `CHANGELOG.md`, `docs/loop/log.md`, this run record | |

### Tests

1. **Unit.** `dyad_delta` is antisymmetric (applying it to both directions moves them together), zero at the
   mean, and cannot overshoot for `rate·dt ≤ 1`.
2. **Village.** Unreciprocated friend share falls by at least a third; mean directional affinity gap narrows.
3. **Counts.** Friends and rivals stay within a sane band — the failure mode is *everyone the same*, which
   shows up as both counts collapsing.
4. **Behaviour downstream.** The refusal rate falls.
5. **Determinism.** Same seed twice, identical state hash.

## Measurement plan

`village-evening` at its own clock, seeds 7 / 42 / 11, 16.4 game-days, via `examples/evidence.rs` with a new
`--dyad-balance` flag and new `summary.json` fields for the reciprocity statistic:

- unreciprocated friend share and mean directional gap;
- refusal rate, and refusal rate by the target's label;
- friends, rivals, label messages, rumor reach, interrupted endings;
- opinion variance, to confirm the dyad step did not disturb the opinion model.

Plots: (a) reciprocity across the sweep; (b) refusal rate by tie label, before/after; (c) the two directed
affinities of one pair over time, before and after, so the convergence is visible.

## Not doing

- **Triad or dyad steps on trust or respect.** Same argument, no measurement yet.
- **Changing `compliment`'s considerations.** That is the definitions file a host ships; the point here is to
  fix the tie, not the pack.
- **A reciprocity threshold** ("ignore gaps below x"). A threshold is a second unsourced number and the
  convergence step already makes small gaps small.

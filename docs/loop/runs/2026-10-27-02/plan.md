# Plan: hold a rivalry through a band, as a friendship is held

Iteration `2026-10-27-02`. One clause in one function, one new `Tuning` field.

## Design

`TieLabel::classify_from` holds a **Friend** while its affinity has dipped no further than `friend_band` below the
friend bar. It has no matching clause for **Rival**, so a rivalry is released the moment affinity rises above
`-rival_affinity`. Measured, that release happens at a margin of **0.0001–0.0002** — one tick of relaxation — and
the median rival spell is **0.14–0.38 game-days** at three seeds.

The change adds the symmetric clause:

```rust
// A rivalry is held the way a friendship is held, and for the reason the pack already cites:
// "bad impressions ... are quicker to form and more resistant to disconfirmation than good ones".
if promoted != Self::Rival
    && current == Self::Rival
    && affinity <= -(tuning.rival_affinity - tuning.rival_band)
    && familiarity >= tuning.rival_familiarity
{
    return Self::Rival;
}
```

`rival_band` is a new `Tuning` field, default chosen by measurement (see Acceptance). It is a **design default**:
Baumeister gives the direction and no number, like `snub`'s curve slope. `0.0` is exactly today's behaviour, so
the field is also the off switch, as `label_hysteresis = 0.0` is for the low end.

## Why this shape and not the alternatives

1. **A wider `label_hysteresis`.** That field guards the `Stranger` boundary (`familiarity`), not affinity, and it
   is consulted only after the `promoted != Stranger` early return — so raising it cannot affect a Rival at all.
   It also already has a measured default whose cost was counted.
2. **Slow negative relaxation** (`affinity_relax_per_day` scaled by sign). Tested in run `2026-12-28-01` and it
   is **backwards**: rivals 42 → 82 while friends fell to 11, because a persisting grudge with a *faster*
   friendship decay leaves no counterweight. It also changes the affinity value itself, which the opinion model
   reads as an influence weight — a coupling across three models. A band changes no value, only the label.
3. **A floor on `rival_affinity`.** A floor is a band that can never be escaped, i.e. a grudge that cannot be
   reconciled. The measurement above shows 149 rival spells at three seeds and the log records reconciliation as a
   property worth keeping; a band releases at a real change and a floor releases never.

## Touched modules

| file | change |
|---|---|
| `src/core.rs` | `Tuning::rival_band`, with the citation, the measured release margin, and the default's provenance |
| `src/relationships.rs` | the rival hold clause in `classify_from`; the doc comment on the bands |
| `tests/label_hysteresis.rs` | `the_rival_bar_keeps_a_single_line` becomes a band test — **it currently asserts the defect** |
| `docs/research/relationships.md` | the hold/release table gains the rival row |
| `docs/loop/scenarios.yaml` | `village-evening`'s observable: rival spells, not just rival counts |
| `CHANGELOG.md` | one entry |

## Tests

1. **`a_rivalry_is_held_through_a_shallow_rise`** — a rival at `-(rival_affinity) + rival_band/2` stays a Rival;
   at `-(rival_affinity - rival_band) - 0.01` it becomes an Acquaintance. Fails before the change (both are
   Acquaintance), passes after.
2. **`a_zero_band_is_the_old_single_line`** already exists and iterates every current/affinity/familiarity
   combination for `label_hysteresis`; it gains `rival_band: 0.0` and so pins that the band is the only difference.
3. **`the_rival_bar_keeps_a_single_line` is rewritten**, not deleted: it named the asymmetry as intended
   behaviour and its own name is now false. The replacement asserts the band and says why the old assertion was
   removed.
4. **`a_rivalry_can_still_be_reconciled`** — a rival that climbs well past the band is released, so the change
   cannot be "make grudges permanent".
5. **The village at seed 7** (`a_held_rivalry_lasts_longer_in_the_village`): rival spells measured through
   `TieLabelChanged`, median spell strictly longer than with `rival_band: 0.0`, and `left_rival > 0`. This is the
   scenario test; the mechanism tests above would all pass on a village nobody could watch.

Not added: no test pinning the shipped default to a number. The default is a design default chosen by
measurement, and a test on the constant would only restate the source line.

## Acceptance

- **The band's value comes from a sweep**, not from taste: `rival_band` in 0.0 / 0.05 / 0.10 / 0.15 at three
  seeds, recording rival pairs, friend pairs, median rival spell, and `left_rival`. The band that lengthens spells
  without moving the friend count by more than the seed-to-seed spread is the one shipped.
- `cargo test` and `cargo test --no-default-features` green; `clippy --all-targets --all-features -D warnings`;
  `fmt --check`; `doc --no-deps --all-features -D warnings`.
- `perf.md`: 100 / 1 000 persons before and after. The clause is one comparison inside a function that already
  runs two; no measurable cost expected, recorded not claimed.
- Determinism: no draw added, no iteration order touched; `tests/determinism.rs` unchanged and green.
- **No save-format change.** `TieLabel` already round-trips; `rival_band` is tuning, and `Tuning` is not
  serialized into a save (checked in `src/save.rs`).

## Risks

- **The village turns hostile in aggregate.** The measurement says rivals are 0–2 today; holding them could take
  that to a number a host dislikes. The sweep records friend and rival pairs together at every setting, and the
  acceptance bar is that friends do not fall by more than the seed spread.
- **A band above `2 · rival_affinity` would make a rivalry unreleasable** — the clamp is `±affinity_ceiling`
  (0.35) and `rival_affinity` is 0.30, so any band ≥ 0.60 pins every rival forever. The field's doc states the
  bound and the sweep does not go near it.
- **`stranger_familiarity` interaction.** A band test at familiarity 0 exercises a path `classify_from` guards
  with `.max(0.0)`; the new clause requires `familiarity >= rival_familiarity` (0.2) explicitly, so it cannot
  hold a tie the tie does not have.

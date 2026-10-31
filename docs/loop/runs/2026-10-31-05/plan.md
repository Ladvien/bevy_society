# Plan - kin (2026-10-31-05)

## Design

One fact on the tie (`Tie::kin = r`), four readers, one authored form. Everything else (what a relative is *worth*) is a `Tuning` field or a pack curve. Details and sources: `docs/research/kinship.md`.

| piece | where | what |
|---|---|---|
| `Kin` enum, `Kin::relatedness()` | `src/relationships.rs` | nine relations with a published `r` (0.5 / 0.25 / 0.125); re-exported flat |
| `Tie::kin: f32` | `src/relationships.rs` | `r`, default 0; `Tie::new` sets 0; `Ties::trim_keeping` never removes `kin > 0` |
| `Input::Kinship` / `Signal::Kinship` | `src/defs.rs`, `src/behavior.rs` | `(2r).clamp(0,1)` from the target's tie; person-only (`check_input`) |
| `Tuning::kin_premium` (1.0) | `src/core.rs`, `src/behavior.rs` | `score *= 1 + premium * 2r` for an activity whose interaction gives goods; precomputed `ActivitySpec::gives` |
| `absence::closeness` | `src/absence.rs` | a kin tie is close whatever the label or familiarity |
| `TieSpec::kin: Option<Kin>` | `src/placement.rs` | authored; spawn mirrors the reverse tie, familiarity 1.0; validation: unequal `r` in two directions, kin to oneself; capture writes it back |
| `ctx.kin(a, b)` | `src/snapshot.rs`, `src/script.rs` | `TieView::kin`, 0 with no tie |
| `TieSave::kin` (`serde(default)`) | `src/save.rs` | saved and restored |
| harness `--households N`, `--kin-premium P`, `kin.txt` | `examples/evidence.rs` | N-person households of full siblings; share-to-kin share, bread variance within and across households |

## Why a multiplier on the score and not an acceptance bonus

Curry et al. measure the **giver's** willingness. The acceptance draw is the recipient's. The score is where the giver chooses, and `crowding_penalty` is the precedent for a price that belongs to the target and not to a consideration, so a pack written before `Kinship` existed gets the premium without a definitions change. `Input::Kinship` stays available for every other activity (visit, chat, gossip) a pack wants to tilt.

## Tuning

- `kin_premium: f32 = 1.0`, `>= 0`. Design default; direction and dependence on `r` from the three sources. `0.0` is the previous release (a society with no kin ties is byte-identical at any value). Doc comment states it; `tuning_covers_rates` and the README Tuning table updated if they list fields.

## Assumed decision (written to `needs-decision/kin-is-a-property-of-a-tie.md`)

Options: (a) `kin: f32` on each directed `Tie`, mirrored at spawn (picked: lookup is free in the scoring loop, a save carries it per tie, trim can see it); (b) a symmetric pair registry resource (correct shape for a dyad, but a lookup per scored candidate and a new save container; it is the shape a later pairing model needs); (c) leave kin to hosts (the status quo). The premium is on by default at `1.0` because it touches only societies that author kin (none exist before this change); the starter pack is unchanged.

## Tests

`tests/kin.rs` (new): `Input::Kinship` reads `2r` and 0 for a non-relative and for no tie; a giver picks the sibling over an equally needy stranger with the premium and splits with `kin_premium = 0` (many seeds); the premium scales with `r` (sibling over cousin); a kin tie survives a trim that drops a stranger's; a relative grieves with no label and no familiarity and a stranger with the same tie does not; placement mirrors, validates and round-trips; a save round-trips and a save without the field loads kin 0; `ctx.kin`; a household society run twice is identical (determinism); a society with no kin is unchanged by `kin_premium` (state hash equal at 0 and 1). Unit tests: `Kin::relatedness` values, `Tie::new` kin 0.

## Evidence (before / after at the same seed)

- `before`: `--households 4 --kin-premium 0` (relatives exist, nobody favours them) = the previous release's behaviour for that society.
- `after`: `--households 4` (premium 1.0).
- Four villages (seeds 7 / 42 / 11 / 99, 24 people, 17 game-days), two towns (60 people), one fresh-eyes run (40 people, seed 3, households of 3), and a premium sweep 0 / 0.5 / 1 / 2 / 4 at seed 7.
- No-kin run at seed 7: `cmp` against the build before the change (`summary.json`, `narrative.txt`, `series.csv`).
- Measure: share of `share` exchanges that go to a relative, against chance; bread variance within and across households; friend / rival / label counts, insults, interrupted endings, the need floors; perf at 100 / 1,000 / 5,000 people before and after.

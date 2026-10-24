# Plan: grudge is the worst offence, not the count

## Why this over the runner-up

Runner-up was the `wander` ceiling: `wander` has no considerations, so it always scores 1.000 and its
`tier_cutoff` floor discards a starving person's `eat` at 0.173. Tested by giving it an `Hour` curve —
`snub` went **50.3% → 53.3%**, i.e. slightly *worse*. Not the lever.

The grudge is, at 24 points of snubbing and a visibly wrong narrative.

## Design

`src/model/recall.rs`: `grudge` takes the **maximum** rather than the sum.

```rust
memories
    .into_iter()
    .map(|(valence, r)| (-valence).max(0.0) * r)
    .fold(0.0_f32, f32::max)
    .clamp(0.0, 1.0)
```

`src/behavior.rs`: a memory's magnitude uses the deed's `harm` **only when the deed fired** (on accept);
a refusal costs the interaction's own `0.3`.

## Measured

| | seed 11 | seed 42 |
|---|---|---|
| `snub` share, sum | **50.3%** | **48.4%** |
| `snub` share, max | **26.0%** | **32.6%** |
| longest single-activity run | 10 → **6** | 11 → **8** |
| top activity | `snub` → **`compliment`** | `snub` → **`gossip`** |

Village unchanged: travelling 58.9% → 59.0%, need mean 0.545 → 0.542 (seed 42: 59.3% / 0.550).

**Honest limit:** seed 11 also showed friends:rivals 4:9 → 6:5 and rumor reach 0.167 → 0.333, and I nearly
shipped that as the headline. Seed 42 shows reach **0.167 either way** and friends 5 → 6. The doubling was
seed-specific; the snub reduction is not. Only the snub reduction is claimed.

## Touched

- `src/model/recall.rs` — the formula and its doc.
- `src/mind.rs` — `Memories::grudge` doc; the unit test that pinned the sum now pins the maximum.
- `src/behavior.rs` — the refusal magnitude.
- `tests/snub.rs` (new) — written red first at 0.700.
- `docs/research/memory.md`, `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. `memory_capacity` already exists; the point is that the grudge no longer depends on it.

## Tests

| Test | What it proves |
|---|---|
| `model::recall::tests::grudge_counts_only_negative_memories_and_clamps` | two 0.3 slights are a 0.3 grievance, not 0.6; positives contribute nothing; clamped at 1 |
| `mind::tests::grudge_is_the_worst_offence_not_the_count_of_them` | four trivial slights no longer equal one betrayal; the deeper offence still dominates |
| `tests/snub.rs::a_refused_snub_does_not_cost_more_than_the_refusal` | **red at 0.700**, green at 0.300: a refused insult writes the refusal's weight, not the deed's |

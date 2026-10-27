# Plan: the merge carries every list it says it carries

Iteration `2026-10-26-13`. One change, in one function, plus the three tests that failed before it.

The iteration's *first* candidate — diluting the affinity and respect a witnessed deed writes — was built,
measured, and reverted; `eval.md` is its record and `review.md` says why the merge took its place.

## Design

`DefsFile::merge` carried **nine of eleven** lists. `skills` and `goods` were missing, and `compile` resolves
`consumes` and `produces` by name, so the omission never surfaced at the merge: it surfaced as
`DefsError::UnknownId { kind: "good" }` **against whichever activity used the good**, in a pack that plainly
declares it. A host splitting its definitions across two files got an error naming the wrong file.

The fix is the same pair of calls the other nine lists already make:

```rust
check_disjoint("skill", &self.skills, &other.skills)?;
check_disjoint("good", &self.goods, &other.goods)?;
self.skills.extend(other.skills);
self.goods.extend(other.goods);
```

`check_disjoint` is what makes a duplicate id a `DefsError::Duplicate`, which is the answer the other eleven
lists already give and what `docs/guide/authoring.md` already promised — the guide said "merging rejects a
duplicate id, in any list", and that sentence was false when it was written.

No new field, no new authored data, no schema change, no per-tick code, no draw.

## What this is not

- **Not a change to what a merge means.** A good named in both files is a duplicate, exactly as an activity,
  need, topic, deed, faction, trigger or conversation named in both files is. The receiver is unchanged.
- **Not a change to the shipped packs.** `examples/assets/village.society.ron` declares no `goods:` and no
  `skills:`, so the merge every example performs produces a byte-identical `SocietyDefs`. That is why there is
  no before/after scenario table for this change: it has no behavioural surface in the crate's own packs, and
  the tests are the evidence.
- **Not a fix for the `UnknownId` message.** `warn_unmade_goods` iterates declared goods, so a `consumes:` on a
  good no pack declares still fails at compile. That path is correct; it is also the message this bug used to
  produce, which is why the diagnosis was slow. Recorded, not changed.

## Touched modules

| file | change |
|---|---|
| `src/defs.rs` | `merge` carries `skills` and `goods`; a comment records the eleven/two split and what it cost |
| `tests/defs.rs` | three tests, each failing against the old `merge` |
| `docs/guide/authoring.md` | the list `merge` carries, named, with the upgrade note |
| `CHANGELOG.md` | one `Fixed` entry with the 0.3.0 upgrade note |
| `docs/loop/needs-decision/good-is-outweighed-by-bad.md` | correction of a premise the reverted candidate measured |

## Tests

1. **`a_merged_pack_contributes_its_goods`** — the second pack declares a good; the merged file compiles and
   the good resolves. Fails before (the good is dropped), passes after.
2. **`a_good_defined_in_both_packs_is_a_duplicate`** — fails before with no duplicate error at all, because the
   good was dropped rather than compared.
3. **`a_good_only_the_merged_pack_consumes_resolves`** — the consuming activity lives in the first pack and the
   good in the second, which is the shape a host hits and the shape that produced the misleading error.
4. **The eight prior tests in `tests/defs.rs` are unchanged** and pass, which is the check that the eleven
   lists that already merged were not disturbed.
5. **`skills`** is carried by the same two calls; no test, because no shipped pack declares a skill and a
   fixture would restate the two lines above it. Stated rather than silently omitted.

Not added, per the brief: no test on the *order* of the merged lists, which is incidental, and none on the
`Extend` itself.

## Acceptance

- `cargo test` and `cargo test --no-default-features` green; `clippy --all-targets --all-features -D warnings`;
  `fmt --check`; `doc --no-deps --all-features -D warnings`.
- The three new tests fail against `d825bdd`'s `src/defs.rs` and pass against this one — verified in both
  directions, recorded in `review.md`.
- `perf.md`: cost at 100 / 1 000 persons, before and after, recorded not claimed. The change runs once at
  compile time and cannot be visible.

## Risks

- **A host relies on the silent drop.** Nothing could: a dropped good is unreachable, and the packs that would
  notice already fail to compile. The changelog's upgrade note covers a split pack that was *working* only
  because the second file's goods were unused.
- **`check_disjoint`'s error text names a `kind` string.** `"good"` and `"skill"` must match the words
  `compile` uses in `UnknownId`, or a duplicate reports a kind the host cannot look up. It is the same literal
  the other nine calls use and the tests assert on the resulting error, not on the string.

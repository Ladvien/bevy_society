# Handoff

Written 2026-10-26. The loop brief, the invariants and the directory layout are in the brief itself; this file is
only what is **in flight, verified and unfinished**, so the next iteration can start without re-deriving it.

## Where the repo is

`main` at `6c35f15` (iteration `2026-10-25-09`, "a village with no room"). Working tree has iteration
**`2026-10-25-10`** uncommitted — see below.

The crate has **no model changes in flight**: every experiment from the last two iterations was either shipped or
reverted byte-for-byte. `git status` is the source of truth.

## Iteration 2026-10-25-10, uncommitted: a full place costs something

**The change.** `Tuning::crowding_penalty` (**default 0.6**) plus one multiply in `src/behavior.rs`'s per-target
scoring loop: a `Place(tag)` / `Bound` target whose place has **no free seat** keeps `1 - penalty` of its score.
`capacity: 0` stays a **prohibition** (never offered). `used: u32` is back on `PlaceView` and is what the price reads.

**Why.** Target expansion filtered on `used < capacity`, so a full place produced no candidate and its activity was
impossible. Measured, a 30-seat village given 48 people took `work` from 10.3% to 3.6% of a life and rival density
from 7.8% to 20.1% of pairs.

**Verified numbers** (five seeds, 24/36/48 people, before → after):

| | 24 | 36 | 48 |
|---|---|---|---|
| rival density | 7.83% → **2.90%** | 23.08% → **6.10%** | 20.11% → **3.94%** |
| `work` share | 10.4% → **12.9%** | 5.8% → **11.3%** | 3.7% → **9.6%** |
| interrupted endings | 318 → 352 | 711 → 669 | 1 325 → 1 278 |

Evidence: `runs/2026-10-25-10/evidence/{before,after}/*/summary.json` and
`runs/2026-10-25-10/evidence/plots/capacity-price.png`. Research, plan, eval, perf and review are all written in
`runs/2026-10-25-10/`.

**Two designs were rejected by measurement before this one shipped** (the proportional price, and removing the veto
with no price). Both are in the eval with their numbers — do not re-try either without reading it.

### What is left before this iteration can be committed

1. **The suite has not been seen green on the final tree.** `tests/crowded_place.rs` (4 tests) and the `crowding_tests`
   unit tests in `src/behavior.rs` (4 tests) were run individually and pass; `tests/bound_places.rs` (7) and
   `tests/label_hysteresis.rs` (8) pass after their fixes. The **whole** suite was running when this handoff was
   written. Run:

   ```
   ./tools/big-cargo.sh test --release --no-fail-fast --all-features
   ./tools/big-cargo.sh --no-sync test --release --no-fail-fast --no-default-features
   ./tools/big-cargo.sh --no-sync clippy --all-targets --all-features -- -D warnings
   cargo +1.98.1 fmt --check
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
   ```

2. **`docs/loop/log.md` has the entry but `Commit COMMIT`** still needs the real hash.

3. **`docs/guide/authoring.md` is not updated.** It has no `capacity` section at all; the honest place for the new
   rule is a short paragraph beside `Activities` → `target`. `getting-started.md` *is* updated.

4. **The GIFs were not re-recorded and do not need to be** — none of the seven clips fills a place. But that claim was
   not checked frame by frame this iteration; it was inferred from the capture's 24 people in 30 seats. Worth one
   look at `docs/media/goods.gif` frame 239 before committing if you want it verified rather than reasoned.

5. **`docs/loop/needs-decision/a-village-with-no-room.md` is now partly resolved** and says nothing about it. Its
   option 2 ("`capacity` as a soft limit with a cost") is what shipped; options 1 and 3 are still open. Add a
   "Resolved in part" section, as `over-subscribed-evening.md` does.

## The top five remaining items (from the log, unchanged)

1. **The price model follows net stock and counts units, not heads.** `consumed` is what `Stock::take` *removed*, so
   `produced − consumed` is net stock and **the price rises when bread piles up** while a real shortage is invisible.
   The sign was flipped in an earlier iteration to make a test pass. Deferred three times. Needs unmet demand counted
   (a new `Flow.unmet`), scaling by population, its own research (excess-demand price adjustment) and its own
   measurement. Evidence: `runs/2026-10-25-08/evidence/plots/spread.png`, bottom left; `docs/research/economics.md`
   "Open contradictions".
2. **`--people` scales people and not the village.** `examples/evidence.rs` builds the same four places (30 seats) at
   every `--people`, which has now produced **two** false results — a "rivalry scaling law" and a "population
   effect" — that reached the log before being caught. Either scale `capacity` with the population or refuse the
   flag above the seat count.
3. **A deed's hostility still grows linearly with the crowd.** `src/reputation.rs` divides `trust` by
   `witnesses ^ Tuning::witness_dilution` and applies `affinity`, `respect` and the memory's `importance` at full
   magnitude per witness. Latané 1981 (*American Psychologist* 36(4):343-356, DOI `10.1037/0003-066X.36.4.343`,
   quoted in `docs/research/rumor-and-reputation.md`) is the law. **It was built, tested, and reverted** — any
   positive exponent empties the 24-person village's rivalries because that village's crowd is already 12.6 witnesses
   per deed. Numbers in `runs/2026-10-25-09/eval.md`. Needs item 2 settled first, or a different lever.
4. **Interrupted arrivals are a travel-time constant**, not a capacity one: they rise with population and now also
   rise 11% at 24 people. `travel_timeout_hours` against `seconds_per_day`; see
   `needs-decision/travel-time-in-two-clocks.md`.
5. **The refusal rate is `accept.base` 0.50** — mean P 0.656, affinity term +0.058. Still the most host-visible
   unmeasured number in the crate.

## Things that cost time and should not be rediscovered

- **`big` was unusable on 2026-10-26**: load average 21-30 from other users, 4 GB free, and two `cargo test` runs
  died with `Connection to 192.168.1.110 closed by remote host` mid-compile. `tools/big-cargo.sh` is correct and
  documented in `docs/DEMOS.md`; retry it, but expect to fall back to the laptop.
- **A cargo lock can outlive a killed run.** A stale `target/release/.cargo-lock` made a later `cargo test` wait
  silently with an empty log for ten minutes. If a build produces no output, check for a running `rustc` before
  assuming a hang.
- **Two test idioms in this repo are traps** and both were hit: a **pinned count** (`assert!(x <= 1)`) that encodes
  one trajectory, and **integer division in an assertion** (`n / 20` truncates, so 6 passes a bar of 6). Assert the
  rate, and print the counts.
- **The 1-second-game-day fixture** used by the decision tests makes a 0.5-hour activity last one tick, so a place is
  never occupied when the next decision runs. Any test about occupancy needs `duration_hours` long enough that two
  people overlap — six hours works.
- **Deciding is gated on `doing.get(person).is_ok()`**, so villagers decide only when idle, and identical villagers
  re-decide in lockstep. A fixture that wants a real queue needs staggered decisions (two needs with different
  decays is enough).
- **Papers downloaded and awaiting conversion**: Perc et al. 2017, `10.1016/j.physrep.2017.05.004`. Latané 1981 and
  Akerlof & Kranton 2000 have **no open-access copy**; Latané's abstract is quoted verbatim in the research note and
  its DOI is verified against Crossref.

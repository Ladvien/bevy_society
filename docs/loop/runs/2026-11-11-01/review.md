# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does a `Destination` now mean something?

Claim: a person with a `Destination` moves toward it, with or without the `steering` feature, and a
`Place` activity can therefore finish and pay its `satisfies`.

**Evidence.** `tests/destination.rs`, written **red first** at `moved 0.00 m from the origin; arrived
false`, now `moved 6.72 m; arrived true`. And in the 24-person village:

| activity | target | completed before | after |
|---|---|---|---|
| eat | `Place` | 14/62 (23%) | **63/74 (85%)** |
| relax | `Place` | 5/23 (22%) | **41/43 (95%)** |
| sleep | `Home` | 24/79 (30%) | **97/103 (94%)** |
| work | `Place` | 6/64 (9%) | **68/95 (72%)** |
| `Place`/`Home` arrivals | — | **26** | **212** |

**Critique.** The tests run in the *default* feature set, where `steer_movers` also exists, so they
prove the `Without<SocialMover>` guard works for a person with no mover. They do not by themselves
prove the no-steering build, because that is a compile-time configuration and a `#[cfg]`-free test
cannot switch it — `cargo test --no-default-features` running the same two tests green is the
mitigation, and it is the configuration the bug lived in.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the plot

**Critique.** `evidence/walk.png` shows the mean of the four needs: flat at zero before, oscillating
between 0.05 and 0.20 after. That is the shape a society with travel should have, and it is the first
time this crate has produced it.

But the plot alone would let me overclaim. The numbers that matter are the per-need zero fractions
after tick 200:

| need | before | after |
|---|---|---|
| social | 39.3% | **2.0%** |
| rest | 43.6% | **0.8%** |
| food | 71.2% | **12.8%** |
| fun | 89.2% | **35.3%** |

`fun` at 35% is still bad, and I have not fixed it: `relax` is its only source and it is gated to
17:00–23:00. Saying "needs are fixed" would be false. `fun` is the top remaining item with a number.

**Critique of a temptation.** The previous iteration added a *travel-reach* warning, and the first
thing I checked here was whether the timeout explains this. It does not: 55 game-hours is 12.8 m of
walking and the village's longest walk is 8.2 m. Two different defects; the warning stays, and
`research.md` says so explicitly so a later reader does not conflate them.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: is the shipped configuration still right?

**Critique.** The fix adds a `Tuning` field, which a host will see. It is 1.4 m/s — the same number
`SocialMover::speed` already defaulted to — so a host switching between steering and not sees no change
in pace. That is deliberate and is in the field's doc comment.

**Critique.** `Destination`'s doc comment was the *evidence* for this bug: it promised movement that
did not exist. It now names `walk_toward_destination` and `Tuning::walk_speed`. A crate whose comments
are load-bearing needs them corrected in the same commit as the behaviour, or the next reader is misled
the same way.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 26 → 212 arrivals; `Place` completions 23% → 85% |
| Zoom | GREAT — needs stop flatlining; `fun` still 35% at zero, reported as such |
| Fresh eyes | GREAT — same pace as steering, and the wrong comment is fixed at the source |

## Top 5 remaining

1. **`fun` at 35% of ticks at zero** — `relax` is its only source and is gated 17:00–23:00. Either a
   second source or a wider window; both are definitions, not code.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
   `needs-decision/retiring-untellable-rumors.md`.
3. **The two-clocks decision** — warned about, measured, and no longer the cause of starvation, but the
   API question (which clock a timeout lives in) is still open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`work` at 72%** — the lowest completion of the three fixed activities; worth a look after `fun`.

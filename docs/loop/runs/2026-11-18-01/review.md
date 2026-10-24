# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does every person's need cycle?

Claim: the observable's first sentence is false for three of four needs, and the reason is measurement.

**Evidence.** Per person, 64 s/day, 80 000 ticks:

| need | lowest any person | cycles for all 8? |
|---|---|---|
| social | **0.39** | **no** |
| rest | **0.50** | **no** |
| food | 0.00 | **yes** |
| fun | 0.12 | 6 of 8 |

`tests/need_cycle.rs`, two tests, both green.

**Critique.** The test asserts `rest` **never** cycles, which will fail the day someone tightens the rates.
That is deliberate — it is the assertion that forces a re-read of the observable — but it means the test
encodes a design default as a fact. The comment says so.

**Verdict: GREAT.**

## Pass 2 — Zoom: the plot

**Critique.** `evidence/tick-zero.png` puts the old and corrected series side by side. The left panel's
"every need touches 0.00 once" is visibly a **single vertical line at tick 0** — the artefact is not subtle
once plotted, and it survived two iterations because nobody plotted `need-cycle` at this resolution.

The corrected panel shows `food` (green) as the only need reaching low, sawtoothing 0.22 → 1.00 once a day,
with `social`, `rest` and `fun` staying in a 0.5–1.0 band. That is what the rates predict.

**Critique of my own reasoning.** I first explained the discrepancy as "the mean of out-of-phase sawtooths
dips below any individual", which is **wrong** — a mean's trough is never below its lowest member. Writing
that down and then checking the arithmetic is what pointed at the row. The wrong explanation is in the log
because it is instructive: a plausible statistical story nearly stopped the search.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: third time for this class

**Critique.** This is the third measurement artefact of the same shape in four iterations:

| | artefact | consequence |
|---|---|---|
| 2026-11-06 | `RumorId(0)` in the series, then in the summary | "rumor reach flat-lines" for three iterations |
| 2026-11-14 | a `4×` tick cost that was never measured | a breaking change nearly justified |
| 2026-11-18 | a row at tick 0 | a `measured` status that was false |

Each was a harness reading something that was not what it claimed. The pattern is worth naming: **every
number this loop publishes comes from `examples/evidence.rs`, and that file has been wrong more often than
the library it measures.** A host reading the crate is unaffected; a reader of the loop's conclusions has
been misled three times.

**Critique.** The mitigation is what has accumulated: `tests/perception.rs` compares against a computed
reference, `tests/scenario_spec.rs` pins the spec contract, `tests/rumor_book.rs` pins the book, and now
`tests/need_cycle.rs` asserts the pre-update state has no needs. Each artefact ended with a test that
fails if it returns.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — per person, only `food` cycles; the observable was three-quarters false |
| Zoom | GREAT — the artefact is one visible vertical line; my first explanation of it was wrong |
| Fresh eyes | GREAT — third artefact of the same class, named, and each now has a test |

## Top 5 remaining

1. **`need-cycle`'s observable is a design question** — three needs stay above 0.5 at these rates. Tighten
   the rates, or ask only that needs recover? A `needs-decision` candidate.
2. **`opinion-converges-but-not-entirely` is not implemented in the harness** — it has a test but
   `--scenario` rejects the name.
3. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip is checked against a third
   configuration.
4. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured.

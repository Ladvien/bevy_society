# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the crate now name a configuration it can prove wrong?

Claim: a host whose timeout cannot reach its places is told at startup, with the number.

**Evidence.** `tests/unreachable.rs` (written red first — the warning did not exist), and the line it
produces:

```
a travel_timeout_hours of 40 at seconds_per_day 4 allows 427 ticks, which at 1.4 m/s is only 9.3
metres of walking — less than 10 m. People whose activity is further away will always time out …
```

Backed by direct measurement of the two clocks: 8.2 m takes **308 ticks**, 13 m takes **527**, and a
40-hour timeout at a four-second day allows **427**.

**Critique.** The threshold is 10 m and the warning speaks for the *default* speed. A host that sets
`SocialMover::speed` itself, or whose village is 15 m wide, gets no warning — the check cannot see
entities in `finish`. That is documented on the function rather than hidden, and it is the honest limit:
the crate can prove a timeout is certainly too short, not that it is certainly long enough. Calling it
"unreachable places" in the first draft was wrong and the name is now `warn_short_travel_reach`.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the plot closely

**Critique — and the plot is where this iteration refuses to overclaim.** `evidence/timeout.png`:

| | before (40 h) | after (55 h) |
|---|---|---|
| `fun` at zero | 55% | **35%** |
| `fun` mean | 0.049 | 0.057 |
| `food` at zero | 28% | 22% |
| `social` at zero | 4% | **11%** |

`fun`'s floor clearly rises and `rest` is visibly better. But **all four needs still hug zero**, and
`social` got *worse*. The temptation is to report "travel fixed", and the plot says that would be false:
travel was a cause, not the cause. The completion table says the rest — 78 of 108 `eat`s interrupted
against 0 of 152 socials — and that asymmetry is the next iteration's item.

**Critique of my own detour.** I spent several probes measuring a nine-person tavern where **nobody
arrived at all**, and concluded capacity was starving them. The real cause was that my probe never
inserted `SocialMover`, so nothing walked. That is a near-miss: had I written it up, the log would blame
`Place::capacity` for a bug in my test app. What caught it was noticing the single-person probe *had*
arrived (tick 308) with a mover and the nine-person one had not without.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: is the corrected note honest?

**Critique.** `need-cycle`'s blocker note said, in the shipped file, that a person "sleeps back to back:
29 completed 4-hour sleeps in 20 000 ticks" *with the shipped rates*. It does not. Re-measured against
the starter pack: `food` and `fun` both cycle [0.000, 1.000]; `sleep` fires **10 times in 10.1 days**
because it is gated `Hours(22, 6)` and ranked 1.

The note is replaced, and the old one is kept under `superseded:` so a reader can see the mistake
rather than a silently edited file. The scenario stays `pending`, correctly: its observable asks all
four needs to cycle and two do not.

**Critique.** A stranger now reads four things in `scenarios.yaml`: what the observable is, that two of
four needs meet it, why the other two do not, and what was previously believed. That is what the file is
for.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — the crate names a timeout it can prove too short, with the arithmetic |
| Zoom | GREAT — reported a marginal win as marginal; caught my own mover-less probe before it shipped a wrong conclusion |
| Fresh eyes | GREAT — the corrected blocker note keeps the old one visible and the scenario honestly `pending` |

## Top 5 remaining

1. **`Place`/`Person` completion asymmetry** — 78 of 108 `eat`s interrupted against 0 of 152 socials,
   with the timeout now ruled out as the first cause. This is what starves `food`, `fun` and `rest`.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
   `needs-decision/retiring-untellable-rumors.md`.
3. **The two-clocks decision** — now has a warning and measured numbers, but the API question (which
   clock a timeout lives in) is still open.
4. **The village's rumor origin** — reaches 1/24; an arbitrary villager cannot be the town crier.
5. **`need-cycle`'s remaining two needs** — `social` and `rest` never cross 0.35 at the shipped rates;
   whether that is correct behaviour or too-generous rates is unmeasured.

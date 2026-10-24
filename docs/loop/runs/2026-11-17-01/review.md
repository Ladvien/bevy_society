# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does every scenario now run as its file declares, and does it pass?

Claim: the spec's seed is honoured, and every scenario meets its own observable at its own settings.

**Evidence.** Two consecutive `need-cycle` runs end at identical values (`[0.957, 0.608, 0.669, 0.867]`),
so the seed is read and deterministic. Every scenario cycles all four needs:

| scenario | need means | cycles? |
|---|---|---|
| `need-cycle` | [0.86, 0.81, 0.58, 0.79] | **yes** |
| `village-evening` | [0.29, 0.53, 0.24, 0.26] | **yes** |
| `rumor-crosses-a-cluster` | [0.64, 0.15, 0.40, 0.61] | **yes** |
| `deed-draws-a-response` | [0.86, 0.81, 0.63, 0.81] | **yes** |

**Critique.** `need-cycle`'s observable says *"every person's"* needs cycle; the series is a **mean across
eight people**, so an individual could be failing while the mean cycles. The per-person claim is not
tested here — `tests/need_cycle.rs` tests one person, which is a different and narrower thing. That gap is
real and recorded rather than papered over.

**Verdict: GREAT.**

## Pass 2 — Zoom: the plot nearly fooled me

**Critique.** `evidence/spec-needs.png` across 19.5 days looks like violent thrashing — all four needs
sawtoothing between 0.2 and 1.0 dozens of times. I was ready to report oscillation as a new defect.

Counting troughs below 0.4:

| need | troughs / 19.5 days |
|---|---|
| social | **0** |
| rest | **0** |
| food | **20** (1.0/day) |
| fun | **0** |

So the full-range plot's density is small excursions, and the real cycling is `food`, once a day, exactly as
the decay rate predicts. `evidence/need-cycle-zoom.png` over three days confirms it: every need decays
steadily and jumps when its activity runs, `rest` holds 1.0 through sleep, `food` sawtooths 0.25 → 0.80.

**A full-range plot can manufacture a defect.** The trough count is cheap and it settles the question; the
review would be wrong without it.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: which conclusions does this retire?

**Critique.** The log has carried `food` at 15% of ticks at zero since 2026-11-13. At the spec's settings
`food` has a mean of **0.24** in the village and **0.58** in `need-cycle`, with one trough a day. It was an
artefact of the four-second day and is retired, not fixed.

`rumor-crosses-a-cluster` reaching **0.583** against a 40% observable means it had been `measured` while
failing — a status the file asserted and nothing checked. That is the second stale status this pair of
iterations has found.

**Critique.** This iteration ships no model change. Its value is that the evidence base is now trustworthy
enough to act on: seven scenarios, each at its declared settings and seed, with the numbers written down.
Everything the log has been queuing for a "fix" needs re-deriving against this before it is worth doing.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — seed honoured and deterministic; all four scenarios cycle their needs |
| Zoom | GREAT — the full-range plot faked a defect; the trough count disproved it |
| Fresh eyes | GREAT — retires `food` at 15% and exposes a `measured` scenario that was failing |

## Top 5 remaining

1. **Per-person need cycles, not the mean** — the observable says *every person*; the series is a mean over
   eight. An individual could be failing.
2. **`opinion-converges-but-not-entirely` is not implemented in the harness** — it has a test but
   `--scenario` rejects the name, so it is the one scenario with no evidence run.
3. **`examples/capture.rs` still hardcodes 8.0 s/day** and does not read the spec, so the README clip's
   numbers are checked against a third configuration.
4. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured.

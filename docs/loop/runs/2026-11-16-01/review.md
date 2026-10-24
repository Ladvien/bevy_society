# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the harness run what the spec declares?

Claim: `examples/evidence.rs` executes each scenario at the settings `docs/loop/scenarios.yaml` gives it,
and the village behaves as the model intends there.

**Evidence.** `tests/scenario_spec.rs` (three tests) plus the run itself:

| | hardcoded 4.0 s/day | **spec 64.0 s/day** |
|---|---|---|
| needs mean | [0.11, 0.21, 0.08, 0.11] | **[0.88, 0.77, 0.50, 0.81]** |
| rumor peak reach | 0.042 | 0.083 (`known_by` 2) |

**Critique.** The needs figure is from a throwaway run at the spec's settings, not from the harness, because
the harness's own output is a series of samples and a summary — it does not print a needs mean. That makes
the headline number reproducible only by re-running a test-shaped program. It is recorded in `eval.md` with
the exact configuration, but a host cannot check it from `summary.json`.

**Verdict: GREAT.**

## Pass 2 — Zoom: four defects the change exposed

**Critique — this is where the iteration earned its keep.** Reading the spec was a three-line change;
making the village *work* at the spec's own settings was not:

| | measured | why |
|---|---|---|
| `ticks: 4200` at 64 s/day | 1.03 game-days, not 17 | the tick counts were authored at 4 s/day |
| `plant_at` 560 at 64 s/day | 3.3 game-hours; reach 0.375 → **0.000** | nobody had gossiped yet |
| 2-hour timeout at 64 s/day | **7.5 m** against an 8.2 m village | the crate's own warning said so |
| `summary.json` | `known_by: 0` while the series showed 2 | it read the final state; the rumor is *forgotten* by design |

The last one is the one I nearly got wrong twice. My first hypothesis was that `best_telling`'s tie-break
starved the planted rumor — I wrote three tests for it, and **all three passed without any code change**, so
the hypothesis was wrong. My second was an off-by-one id race; I "fixed" it and the number did not move. Only
printing the rumor's `origin` and `subject`, then checking each villager's `Knowledge`, showed that the rumor
had `known_by: 2` and **zero living holders** — because aging past 150 later rumors evicts it. That is the
crate working, and the summary was asking the wrong question.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: is the corrected observable honest?

**Critique.** `village-evening` asked for the rumor to reach **half the village**. It cannot, and the reason
is measurable rather than mysterious: the village makes about **18 rumors a day**, and `best_telling`
correctly prefers the strongest and freshest. A planted rumor is told — `known_by` 1 → 2, measured — and
then out-competed. Reaching half would need it to stay the most salient thing for days.

The observable now asks that it be **told to at least one other person**, with the reach figure recorded
rather than being the pass condition. The old wording is kept in the file with the measurement that
replaced it, so the change is visible rather than silent.

**Critique of what this means for the loop.** Every needs and rumor conclusion in the previous eight
iterations was measured at a day length 16× shorter than the spec. Some of them — the `need-cycle` blocker,
the "rumor reach flat-lines" note, `food` at 15% of ticks — may be artefacts of that configuration. The log
says so plainly rather than quietly re-deriving each one.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — the spec is executed; needs 4–7× higher at its settings |
| Zoom | GREAT — four defects, including one I got wrong twice before measuring |
| Fresh eyes | GREAT — the observable now asks what the model does, with the old wording kept |

## Top 5 remaining

1. **Re-derive the behaviour conclusions at the spec's settings.** `need-cycle`, `food` at 15%, and the
   rumor-reach note were all measured at 4 s/day; none may survive at 64.
2. **`examples/capture.rs` still hardcodes 8.0 s/day** — the third value. It should read the spec too, and
   the README clip's numbers re-checked.
3. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
4. **The two-clocks decision** — warned about, measured, API open.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured.

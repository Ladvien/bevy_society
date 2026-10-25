# Review

## Pass 1 — Behavior: does the component now say something true?

**GREAT.** `Doing::label` returns `travelling:sleep` while a person walks to bed and `sleep` once it arrives,
verified tick by tick in `tests/doing_label.rs`. Deleting the prefix fails exactly the walking test and no other,
which is the check that the test is measuring the behaviour and not the plumbing.

## Pass 2 — Zoom: read the tally, not the summary

**GREAT**, with one correction made during the pass. My first parse of the harness's `doing` field read
`travelling:work=15 wander=1 work=8` as a single token and reported `compliment` at 34.7% of all person-ticks — a
village whose most common activity is complimenting people. Re-parsing the format gave the true distribution:

| state | share |
|---|---|
| sleep | 36.8% |
| work | 22.9% |
| eat | 13.3% |
| compliment | 8.7% |

Sleep at 36.8% against the 33.3% one 8-hour sleep a day implies is the defect this iteration is about, and it is
the same 3.5-point gap the decision file recorded. The distribution otherwise reads like a village.

I also checked the pathologies the loop names: no need clips at 0 or 1 anywhere in 67 199 rows, opinion variance
stays in 0.003–0.053, and mood occupies ±0.12 rather than pinning at a rail. Travel is 1.0% of person-ticks.

## Pass 3 — Fresh eyes: can a host explain it?

**GREAT.** One sentence: *`activity` is what the person chose; `label` is what it is doing now, and it says
`travelling:` while it walks.*

The strongest evidence that this belongs on the type: the crate's **own evidence harness** had already invented
this exact convention privately. Two independent readers converging on one workaround means the workaround
belongs on the component.

The byte-identical `series.csv` is the proof that no existing consumer changed behaviour — the helper is an
extraction, not a redesign.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT** (after fixing my own misparse). Pass 3 **GREAT**.

What did **not** ship, deliberately: option 2 of the decision file (a reserved `travelling` activity plus a target
field) is a schema change to a component a host reads, and stays a host decision. `tests/travel.rs` keeps its
`#[ignore]` because it asserts the stronger claim — that a walker is being *paid* — which is the clock decision.

# Eval

## Before / after per defect

Each of the four defects was found by measuring the one before, so each has its own evidence.

### 1. `snub` gated on low affinity

| | village | cluster of strangers |
|---|---|---|
| `snub` person-ticks over the run | **0** | 2583 |

Curve evaluated: `Logistic(-8, 0.3)` scores 0.92 at affinity 0.0 and 0.08 at 0.6.

### 2. Script considerations could not see the target

| | before | after |
|---|---|---|
| `Script("ctx.grudge(actor, target)")` on a Person option, with a grudge of 0.9 | idle (`ActivityId(0)`) | `snub → Person(B)` |

### 3. `patient_reaction` moved no affinity

Reading the code: `Appraise` + `tie.trust`, no affinity write. Confirmed against the SPEC, which
specifies exactly that, so the gap is in the model rather than the implementation.

### 4. Witness approval used the norm's sign, not the act's

Witness with opinion +0.9 on `kindness`, watching an `insult` (harm −0.4):

| | affinity toward actor | respect |
|---|---|---|
| before | **+0.138** | +0.090 |
| after | negative (the witness disapproves) | negative |

## Scenario outcomes

| scenario | before friends | after friends | before rivals | after rivals |
|---|---|---|---|---|
| `village-evening` | 171 | 79 | 0 | 0 |
| `rumor-crosses-a-cluster` | 19 | 0 | 11 | **30** |
| `need-cycle` | — | 5 | — | **21** |

Synthetic crowd of 12 strangers: **119 of 132 ties** end below −0.4 affinity, labelled `Rival`.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 16 binaries ok |
| `cargo test --no-default-features` | 16 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## `tests/behavior.rs`

16 tests, up from 14: `a_script_consideration_can_score_a_person_target` and
`a_script_consideration_reads_no_grudge_for_a_stranger`. The second exists so the first cannot
pass by binding the wrong person.

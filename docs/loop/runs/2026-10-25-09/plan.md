# Plan: one dilution for every effect of witnessing

## Pick

**Pick:** make `Tuning::witness_dilution` divide **every** per-witness consequence of a deed — affinity, respect and
the memory's importance — not only trust.

**Why it beats the runner-up** (the price model, last iteration's item 1): a price that rests at its floor is a
cosmetic defect in a number; a village where a **tenth of all pairs hate each other** and that fraction triples when
the host adds twelve people is a behaviour a host cannot ship. It is also a one-line inconsistency rather than a
model, and the evidence for it is already measured.

## Design

`witness_reaction` already computes `seen = witnesses ^ Tuning::witness_dilution`. Three writes ignore it.

| effect | before | after |
|---|---|---|
| `trust` | `+ trust_gain/harm / seen` | unchanged |
| `affinity` | `+ 0.15 * approval` | `+ 0.15 * approval / seen` |
| `respect` | `+ 0.10 * approval` | `+ 0.10 * approval / seen` |
| `Memory.importance` | `|harm|` | `|harm| / seen` |
| `Memory.valence` | `approval * |harm|` | unchanged — it is already `|harm|`-scaled and the memory is a *record*, not an effect |
| `Emotion` (Admiration/Reproach) | `approval` | unchanged — a feeling is not a ledger entry, and Latané's law is about *impact on behaviour*, not whether one felt anything |
| `NormViolated` (`approval < -0.5`) | unchanged | unchanged — a witness that objects still objects |

**Why not dilute the emotion or the norm signal.** The crate's own note on `witness_dilution` says the dilution
"applies to the deed's *magnitude*, never to its direction or to whether it is right to disapprove". An emotion's
intensity feeding mood contagion is the same kind of thing as the norm judgement: a bystander in a crowd is not
*less disapproving*, they are less **moved** by one incident. That distinction is what stops a crowd of witnesses
from being emotionally inert — a village where nothing ever registers would be a worse failure than a hostile one.

**Why the exponent stays 0.5.** Latané's power law for the number of sources, and the value the crate already
ships. No new field.

### Touched

| file | change |
|---|---|
| `src/reputation.rs` | divide the three undiluted effects by `seen`; cite Latané beside the formula |
| `src/core.rs` | `witness_dilution`'s doc: Latané 1981 as the source of the *exponent*, Sommerfeld for transmission |
| `tests/witness_dilution.rs` | one test per effect, at three crowd sizes |
| `examples/evidence.rs` | a `--people` scale sweep is the evidence; no harness change |

### Tests

1. **Affinity per witness falls with the crowd.** One insult seen by 0, 3 and 9 bystanders: the affinity each
   witness loses must fall, and the *total* affinity lost must still rise (more witnesses, less each).
2. **Respect and memory importance fall the same way**, and the `NormViolated` count does not.
3. **The total hostility of a deed is sublinear in the crowd** — the property the fix exists for.
4. The existing trust test still passes unchanged.
5. **The village keeps its rivalries**: at 24 people the 17 rival pairs and 69 violations must survive, within noise.
6. Determinism: the model draws nothing, so two runs are identical.

### Measurement

Nine seeds × populations 24 / 36 / 48, rival density per population before and after, plus the four regression
numbers at 24. Target: density flat in population, and the 24-person village's rival count level.

## Not doing

- **Diluting the emotion or the norm threshold** (above).
- **The price model**, still item 1 in the log, still needs unmet demand counted.
- **A second good**, still item 3.

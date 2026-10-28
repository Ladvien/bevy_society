# 2026-10-28-10 — what writes the grudge behind a snub?

Open issues: #24, #27 (release; skipped). No bug issue. Item: last pass's top remaining item.

Regenerate: `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --out DIR` and `--seed 7 --people 60 --days 30` for the town. Output `DIR/grudge_sources.txt`. At each `snub` start, the snubber's memory about the target with the largest `max(0,-valence)*recency` (exactly what `Memories::grudge` takes the max of) is classified. Interaction ids in the starter pack: 0 chat, 1 gossip, 2 compliment, 3 insult.

| source of the grudge | s7 | s42 | s11 | town60 |
|---|---|---|---|---|
| insult **received and accepted** | 72.9% | 73.0% | 72.8% | 55.5% |
| own insult attempt **refused** (initiated) | 10.3% | 9.5% | 9.3% | 18.7% |
| refused chat / gossip / compliment (either side) | 17.1% | 16.4% | 17.0% | 25.0% |
| rumor / witnessed deed | 0.8% | 1.5% | 1.3% | 0.8% |
| snubs measured | 739 | 727 | 777 | 3 781 |

(Committed copies: `evidence/*/grudge_sources.txt`, `summary.json`.)

## Finding

Three quarters of all snubs at 24 people, and over half at 60, are an answer to an insult the snubber **received**. The rivalry engine is insult-for-insult retaliation after all; the 2026-10-28-09 trial missed it because it cleared only the **actor's** grievance, and the actor then gets a fresh `insult received` memory from the victim's reply, so the exchange ping-pongs. 10-19% of snubs rest on the snubber's own *refused insult*, a self-feeding loop (failed attempt leaves a bad memory about the same target, which scores the next attempt). Rumors and witnessed deeds are under 2%: reputation does not drive the hostility, direct exchange does.

Verdict on the previous pass's hypothesis: its target was right (retaliation) and its remedy wrong (settling one side). Nothing shipped here.

## Next lead

Break the ping-pong at the **reply**, not the grievance: the victim of a just-received insult answering within hours is the chain. Candidate: a per-pair retaliation refractory period (design default, swept) or letting a refused insult not write a grudge-bearing memory about its target; measure insult count and rivals before/after at the same seeds.

Verdicts: not run (nothing shipped). Dead-code delta: zero.

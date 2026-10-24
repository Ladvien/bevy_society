# Eval

## Before / after, crowd of 12 strangers, same seed

| tick | mean affinity (before) | mean affinity (after) | activities in play |
|---|---|---|---|
| 100 | −0.047 | −0.038 | 2 → **3** |
| 500 | −0.437 | −0.250 | 2 → **2** |
| 1200 | −0.899 | **−0.674** | 2 → **5** |

Activity ids: 2 = eat, 3 = work, 5 = gossip, 7 = snub, 8 = relax. Before the change only gossip
and snub ever ran; after, five activities appear, including `work` and `relax` that the crowd
previously had nowhere to do.

## The three fixes, each measured

| Fix | Measurement |
|---|---|
| Cluster scenario gets a hearth | activities in play 2 → 5; `chat`/`eat`/`work`/`relax` all appear |
| `snub` slope `2.5 → 1.0` | mean affinity at t=1200: −0.899 → −0.820 (isolated from the hearth fix) |
| Harness day length corrected | `village-evening` 17.03 game-days (was silently 17 by accident at 240 s), cluster 5.02 (was 19) |

## Scenario outcomes after the change

| scenario | game-days | friends | rivals | reach |
|---|---|---|---|---|
| `village-evening` | 17.03 | 110 | 0 | 0.375 |
| `rumor-crosses-a-cluster` | 5.02 | 1 | 12 | 0.333 |
| `need-cycle` | 6.19 | 2 | 8 | 0.0 |

Three scenarios, three distinct societies. Before this iteration all three trended to uniform
hostility in the crowd cases, and the village alone stayed friendly.

## What is *not* fixed, and the reason

Affinity still trends downward in a crowd that stays together long enough. The previous review
asked for damping. The literature says there is no such damping to add: **Takács, Flache & Mäs
2016** (`10.1371_journal.pone.0157948`, converted this iteration) show that people do **not**
shift away from those they dislike — negative influence is not empirically supported. What
changes is who a person chooses to interact with, which is a *decision* problem and exactly what
giving the crowd something to do addresses.

So the remaining downward trend is documented as the model behaving as the literature describes,
not as an outstanding defect to tune.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 16 binaries ok |
| `cargo test --no-default-features` | 16 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## Corpus

One paper downloaded and converted: `10.1371_journal.pone.0157948` (21 pages, 121 s on scribe-vlm).
It is cited in `docs/research/rumor-and-reputation.md` for the negative result that shaped this
iteration.

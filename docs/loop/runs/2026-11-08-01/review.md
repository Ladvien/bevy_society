# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the trace explain a decision, and does it leave the society alone?

Claim: a host can now see what a person weighed and by how much, and adding the recorder does not
change what anyone does.

**Evidence.** `tests/trace.rs`:

| assertion | result |
|---|---|
| no-outcome-change | the same seed gives an **identical chosen sequence** with and without the trace |
| the trace names the options | `chose chat (score 0.038, shortfall 0.00); other options: [("play", 0.038)]` |
| fixed-size ring | exactly 8 decisions after 4 000 ticks, newest first |

and `evidence/village/why.txt` for a real village decision: six options, the lowest chosen, 25% below
the best.

**Critique.** The no-outcome-change test is the load-bearing one and it is strong: it compares the
whole sequence of `ActivityStarted` names, not a summary, so any perturbation of the RNG stream would
show. But it is a *single* seed and a two-activity definition — a recorder that only misbehaved with
more activities would slip through. The determinism test covers the seeded path more broadly (24
people, 3 000 ticks, 2/2), which is the mitigation.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the artifacts closely

**Critique — and this is where the iteration earned its keep.** The first `why.txt` I generated said:

```
tick 4147, margin -0.33
```

**A negative margin**, from a quantity I had documented as "how much better the chosen option was than
the next best, as a fraction of the chosen score". It cannot be negative. Running it on the village —
not reasoning about it — is what found it.

The cause is real and instructive: the crate uses Dill's **weighted draw** over the viable tier, so the
chosen option is frequently *not* the top scorer (that village line chose the sixth-best of six). A
measure built by comparing against "the next one in a sorted list" is comparing against the wrong thing
entirely. Replaced with `shortfall` against the best score, and four unit tests now pin it.

Second thing the trace exposed on its first use: **`name: None`**, rendering as "Why someone did that".
`decide_activities` is at Bevy's 16-parameter limit, so resolving `Name` there would be a seventeenth.
Solved by `Decision::describe(&[(Entity, String)])` — the reader supplies names, the trace falls back to
"someone" for an entity that has since despawned, which is honest and is why the comment says so.

**Critique of the numbers.** `1.000 snub / 1.000 relax / 1.000 wander / 0.971 eat / 0.912 chat /
0.752 compliment` — several activities saturate at exactly 1.0. That is the multiplicative score with
every consideration near its best, not a bug in the trace, but it does mean the *ordering* among the
1.000s is decided by `rank` and then by the draw, which a host reading the file cannot see. Noted for a
later iteration; the trace reports the rank tier it saw, not the tier rule.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: could a host developer explain this without reading the code?

**Critique.** The output is one sentence and a table:

```
tick 4147: someone chose compliment (0.752), 25% below snub (1.000)
```

and

```
 1.000  snub
 1.000  relax
 ...
 0.752  compliment   <- chosen
```

A host can read that aloud. It answers "why did Bram do that?" with "because a weighted draw picked
the sixth-best option, a quarter below the best", which is a *reason*, not a restatement of the
behaviour.

**Critique.** The one thing a stranger will ask is "why is a lower-scoring option ever chosen?" — and
the trace does not say *tier_cutoff* or *weighted draw* on its face. `Decision::describe` says "a
weighted draw picked this over a higher scorer" in the long form, which is the answer; the tabular file
does not. A host who prints the file alone gets the numbers without the rule.

**Verdict: GREAT** for explainability; the tabular form still leans on the sentence above it.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — identical outcome with and without the recorder; the options and their scores are readable |
| Zoom | GREAT — the first output had a negative "margin", which the plot-as-text caught and the semantics fix |
| Fresh eyes | GREAT — a host can say why, in a sentence, from data the crate now produces |

## Top 5 remaining

1. **Back-to-back activity loop is NOT a crate bug** — last iteration's blocker was my test's definition
   (starter `sleep` is `Hours(22, 6)`, ranked 1, once per day). `need-cycle`'s observable still is not
   met by the shipped rates; the real question is the rate balance, not the loop.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
   `needs-decision/retiring-untellable-rumors.md`.
3. **The two-clocks decision** — still the only cause of need starvation in a real layout.
4. **The village's rumor origin** — reaches 1/24; an arbitrary villager cannot be the town crier.
5. **`Field` / `Gradient` wiring** — unchanged.

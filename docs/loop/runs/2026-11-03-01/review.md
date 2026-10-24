# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the documentation now do what it says?

Claim: a host who copies the crate's quick start gets a society that advances.

**Evidence.** `evidence/before-tick-count.txt` — `BEFORE_TICKS=0`. `evidence/after-tick-count.txt` —
`AFTER_TICKS=37`. Same program, same 600 updates, same seed; only the documentation differs. And the
guide: `evidence/before-guide-panic.txt` shows the panic a host hit at `src/dialogue.rs:86`; the guide
now shows the disable and says the plugin panics.

**Critique.** A tick count is a weak behavioural claim on its own — 37 ticks is a very young society.
But it is the *right* claim here: the defect was that the number was zero, and nothing in the crate
ran at all. The stronger claim, that the snippet is *correct*, is carried by a different check: the
doctest compiles and runs, which is what caught my own first repair naming a `TimeUpdateStrategy` that
Bevy 0.20 does not re-export in its prelude.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the artifacts closely

**Critique — and this pass is the reason the iteration is worth more than its diff.** Reading the
plot the brief requires turned up three things I was not looking for:

1. **`need3` pinned at 0.0 on 3 283 of 4 200 ticks (78.2%)**; `need2` on 1 950 (46.4%). The `need-cycle`
   scenario's own `observable` says a need that only falls means no activity satisfies it. That is
   exactly what the village plot shows.
2. **Rumor reach freezes at tick 62 of 4 200.** Planted at tick 200 with a target cluster, it reaches
   0.500 and never changes again — gossip that spreads instantly, then stops. The brief names this
   failure mode ("gossip that never spreads or spreads instantly") and it is live.
3. **19 `ActivityEnded` against 9 `ActivityStarted`** for one tracked person in the fresh-eyes run.
   This one is a crate defect with a located cause: starts are written on arrival
   (`behavior.rs:340`) and for no-travel activities (`:832`), while ends are written for any removed
   `Doing`, including a travel timeout (`:449`). A host subscribing to the pair sees ends with no
   starts.

Also fixed here, found by reading: the narrative printed `20h43` followed by `08h26`, which reads as
time running backwards. It now prints `d3 20h43`. Without the day marker a multi-day narrative is
unreadable, and the narrative is a mandatory gate.

**Critique of the fix itself.** `--people` past 24 indexed `["millers","weavers","watch"][3]` and
panicked. Fixed by wrapping the faction index. Both of these were found by *running* the harness with
the new flags, not by reading it.

**Verdict: GREAT** — the pass found three behavioural defects and one harness defect that would
otherwise have shipped silently.

## Pass 3 — Fresh eyes: a different seed and population

**Critique.** `--scenario village-evening --seed 99 --people 40`
(`evidence/fresh-eyes/`): 17.0 game-days, 372 friend-edges, 0 rivals, rumor reach 0.300. The narrative
reads as a coherent day for a stranger: chat, compliment, gossip, sleep, with times that advance and
names that repeat between people (V28 appears twice, Corin and Sable recur), which is what a small
society should do.

**Critique of what it exposes.** All four needs of the tracked person end at **0.0**, and there are
16-game-hour gaps with no activity at all (`d5 14h37` → `d7 09h33`). So the society is not yet
believable at this population either; the difference from 24 people is that 40 makes the starvation
worse, not better. That is the same finding as pass 2, from a different seed and a different
population — which is what makes it worth the top slot next iteration rather than a footnote.

**Verdict: GREAT** for the documentation change (it works at any seed and population); **NOT YET** for
the society's need satisfaction, recorded as the top remaining item with numbers.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 0 → 37 ticks, guide no longer panics, all asserted by compiled checks |
| Zoom | GREAT — found two need-starvation defects, a frozen rumor, and the start/end asymmetry |
| Fresh eyes | GREAT for the change; NOT YET for need satisfaction, with numbers, seed 99 / 40 people |

## Top 5 remaining

1. **`ActivityStarted` / `ActivityEnded` asymmetry** — ends fire for activities whose start was never
   written (travel timeout). Located at `behavior.rs:340`, `:449`, `:832`. A host listening sees ends
   without starts; gossip and witness systems will inherit it.
2. **`need3` at zero 78% of the time** — no activity the starter pack offers satisfies it often enough.
   `need-cycle` is still `pending` for this reason.
3. **Rumor reach frozen at 0.500 from tick 62** — spreads instantly, then stops. Daley–Kendall
   stifling should make it *decay*, not flat-line.
4. **`Field` / `Gradient` wiring** — unchanged from last iteration, four measured facts recorded.
5. **`warn_unscored_fields` says "1 definitions"** — a plural helper.

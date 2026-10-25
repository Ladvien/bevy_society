# Review: a friendship is held through a shallow dip

Three passes, each with a written critique, before the verdict. Evidence is under `evidence/`.

## Pass 1 — Behaviour: does the change do what the paper and the scenario say?

**GREAT, for the claim actually made.** Altman & Taylor's social-penetration account (via Battle & White
2024, corpus `10.33790_jmhsb1100188`) says a friendship is *maintained*, and that its categories are staged
rather than a line on one scalar. The change makes the loss of a friendship follow a **failure to maintain
of some size** rather than any dip: `friend_band` 0.05, and the measured effect is exactly that.

| seed 7, 16.4 game-days | before | after |
|---|---|---|
| median held friendship | 0.36 game-days | **1.01** |
| friendships that ended | 504 | **242** |
| label messages | 1 953 | **1 546** |
| friends / rivals | 18 / 42 | 22 / **42** |
| `became_rival` / rumor reach / interrupted | 272 / 0.542 / 329 | **272 / 0.542 / 329** |

The four numbers that must not move did not move. `evidence/before/summary.json`,
`evidence/after/summary.json`.

## Pass 2 — Zoom: is the band in the right place, and is the story it tells true?

**GREAT, but only after two wrong versions were measured and thrown away.** This is the pass that did the
work, and both failures are worth recording.

**Wrong version 1: the band guarded the wrong branch.** The first implementation put the friend case inside
the `match current { … }` arm that only runs when the promotion test already failed at the *stranger* bar.
It compiled, it read plausibly, and it changed **exactly nothing** at 0.05 or 0.10 — messages 1 953 at every
width. A guard that can never fire is indistinguishable from a guard that works until you sweep it.

**Wrong version 2: my first "proof" that refusals were not the cause.** I compared the refusal penalty at
0.05 / 0.02 / 0.00 and concluded it was not the mechanism, because endings only fell 504 → 264. That
comparison was run against version 1, so the friend band was inert throughout and the only thing varying was
the penalty. Re-run against the fixed guard, the numbers are the ones in `eval.md`. **A controlled comparison
against a broken arm measures the arm, not the knob.**

**What the zoom did establish.** Endings cluster on affinity **0.25 and 0.30** — one `-0.05` step from the
ceiling and from the bar — and `-0.05` is `rejection_affinity_penalty` at a 38% refusal rate. The friend
shelf is **0.05 wide**; a median tie swings **0.072**. `evidence/plots/friend-band.png` plots endings by cause
before and after: insult 99 → 63, exchange 349 → 136, nothing 56 → 43.

**What the zoom refuses to claim.** The drift endings barely move. The band absorbs shallow dips; it cannot
tell a quiet friendship from one whose owner was refused, because no field on `Tie` records that an exchange
happened — `last_contact_day` is stamped by mere proximity. The plan's grace-window arm was **dropped on this
measurement** rather than shipped.

## Pass 3 — Fresh eyes: different seed, and is it explainable?

**GREAT.** Seed 42 and seed 11 reproduce the direction: endings 507 → 251 and 2 080 → 1 710 messages, friends
18 → 22 / 9 → 11, rivals **42 → 42 / 40 → 40**. The narrative
(`evidence/narratives/`) is the same villager's day and reads identically — a host needs no new concept to
explain it: *friendships last longer now, and a friendship still ends when someone refuses you or insults
you.*

**What a stranger would still complain about, correctly:** a friendship lasting one game-day is short, and
a third of the endings are refusal-driven. Both are upstream of the label and are the first two items below.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**, after two measured rejections. Pass 3 **GREAT**.

## A test deleted, not kept

`the_friend_and_rival_bars_keep_a_single_line` asserted that a tie just under the friend bar is an
`Acquaintance` "band or no band" — the previous iteration's conclusion, which this iteration's measurement
overturned. It is replaced by `a_friendship_is_held_only_within_its_band`, which asserts both halves: held
inside the band, released past it.

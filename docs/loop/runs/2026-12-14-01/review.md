# Review

## Pass 1 — Behavior: does the corrected guide match the loader?

**GREAT**, and the claim is checkable by running it.

| documented form | parses |
|---|---|
| `Tick`, `Hour(h)`, `Interaction("name")` | yes |
| `TieLabel(Friend)` | yes — and `TieLabel(friend)` fails |
| `RumorReach(deed: "steal", fraction: 0.5)` | yes |
| `NormViolated("steal")` | yes — `NormViolated(deed)` fails |
| `NeedBelow(need: "social", below: 0.3)` | yes |
| `Activity("name")`, `DialogueNode(conversation: "greet", node: "n")`, `Custom("name")` | yes |

**10 of 10**, against 6 of 10 before.

## Pass 2 — Zoom: were the errors what I thought?

**GREAT**, and twice my *assumption* was wrong in a way the runs corrected.

I predicted the **prose** was right and the **example** wrong, because `TriggerOn::RumorReach` is declared as a
struct variant. Running it showed the opposite: `RumorReach(deed: ..., fraction: ...)` **parses** and
`RumorReach { deed, fraction }` **fails**. RON accepts parentheses for a struct variant, so the prose was the
error.

Then my test failed on `TieLabel(friend)` and on `Interaction("chat")` — the first a real guide gap
(capitalisation), the second my own test's fault (an interaction the file did not define). Both were caught by
running rather than by reading.

## Pass 3 — Fresh eyes: what would a host actually hit?

**GREAT**, and this is the pass that justifies the iteration. A host's very first move is to copy the block the
guide opens with, or to write one `on:` clause. Before this, four of the ten documented forms were a guaranteed
error, and the error names a byte offset.

The prose trap is subtler and worth more: **two spellings of the same concept appear twenty lines apart** —
`TieLabel(Friend)` in RON and `"friend"` in Rhai — and the test pins both, so a future "cleanup" that unifies
them fails immediately.

The narrative to a host:

> The guide's trigger table now matches what the loader accepts. Struct variants take parentheses, not braces;
> `NormViolated` takes a bare deed name; and the label variants are capitalised in the definitions file but
> lowercase inside a Rhai condition. Every row is compiled by a test, so the table cannot drift.

## Verdict

Three GREAT. Shipped. No model change: the crate was right and the documentation was wrong in four places.

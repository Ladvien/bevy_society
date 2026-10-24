# Research: the README promised something the crate did not do

## Why this iteration is not about the friendship clique

The loop's rule: *"After 5 failed rounds, log the blocker and revert."* Five mechanisms have now failed on that
problem — a `help` activity, a contact exchange, saturating affinity, a trust gate, and saturating all three
accumulators — and the blocker is documented in
`docs/loop/needs-decision/friendship-has-no-diminishing-return.md` with the one untested shape. Further attempts
belong to a fresh look, not to a sixth variation.

So this iteration audited the **public surface**, which no earlier iteration had.

## The finding

`README.md`, "What you get":

> Every formula traces to a paper; **every rate is a named field on `Tuning`**; every default either comes from
> a paper or says it is a design default.

Two literals in non-test code break that promise, and both are things a host would want to change:

| literal | where | what it decides |
|---|---|---|
| `0.4`, `0.3`, `0.2`, `0.1` | `TieLabel::classify` | when a tie becomes a friend, a rival, a stranger |
| `0.05` | `resolve_interactions`, the rejection penalty | affinity lost when an approach is refused |

The `classify` thresholds are documented *in that same function* as "design defaults (no source paper gives
them)" — which is precisely the case the rule exists for. A default nobody can source must at least be visible
in one place and adjustable in one place.

## Why they are consequential, measured

`village-evening`, seed 7, 24 people, 17 game-days:

| | affinity | trust | familiarity |
|---|---|---|---|
| friend-ties (115) | **0.792** | **0.480** | 0.907 |
| other ties (53) | −0.223 | 0.068 | 0.908 |

`friend_familiarity = 0.3` is satisfied by **0.907 against 0.908** — the term discriminates *nothing*, and no
host could discover that without reading the source. Whether that is a bug is the friendship decision's
question; whether it is *visible and adjustable* is this iteration's.

## Design

Five new `Tuning` fields, defaults equal to the literals, plus one for the rejection penalty:

| field | default | source |
|---|---|---|
| `friend_affinity` | 0.4 | design default |
| `friend_familiarity` | 0.3 | design default |
| `rival_affinity` | 0.4 | design default |
| `rival_familiarity` | 0.2 | design default |
| `stranger_familiarity` | 0.1 | design default |
| `rejection_affinity_penalty` | 0.05 | design default |

`TieLabel::classify` gains a `&Tuning` parameter. It had one real caller (`refresh_label`, which already held
`tuning`) and five test assertions, so the change is small.

## Sources

Unchanged. `classify`'s doc already cites mere exposure (Zajonc 1968, DOI `10.1037/h0025848`) and similarity
(Byrne 1961, DOI `10.1037/h0044721`) for the two *mechanisms*, while stating plainly that the four thresholds
have no source. That statement is right; only its visibility was wrong.

## What believable output looks like

Unchanged output — this is a surface change — with a host able to raise `friend_affinity` and see ties demoted,
which is the test.

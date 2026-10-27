# Research: what a victim keeps of the harm done to them

Iteration `2026-10-27-03`. The loop log's #1 was `patient_reaction` moving no affinity: *"a victim keeps
liking their insulter"*. Measured before designing anything, and the premise was understated.

## The measurement that chose it

A village of 24 (`village-evening`'s own layout, seed 7), 12 000 ticks, every accepted `insult` caught as it
reached the victim:

```
accepted insults this tick: 1
  actor->victim aff 0.131 | victim->actor aff 0.034 trust 0.000
of 1 victims: negative-affinity 0 | grudge 1 | negative memory 1 | nonzero aff 1
village: 30 friend edges, 50 rival edges, 24 people
```

**The victim's affinity for the person who just insulted them is `+0.034` — positive, and not far from the
insulter's `+0.131`.** That is the defect stated exactly: after being insulted the victim still *likes* the
insulter, and slightly more than nothing.

The same probe against the host-reported path (`ReportDeed`, one bystander named as witness) shows the two
sides are not merely asymmetric, they are different kinds:

```
BEFORE patient->actor aff 0.0000 trust 0.5000 | witness->actor aff 0.0000
AFTER  patient->actor aff 0.0000 trust 0.4200 | witness->actor aff 0.0000
AFTER  patient mems []  grudge 0.0000
AFTER  witness mems ["Witnessed { deed: DeedId(0), actor: 65v0 } v=-0.000 i=0.400"]
```

`aff 0.0000 → 0.0000`, no memory, `grudge 0.0000`. The witness records a `Witnessed` memory; **the victim
records nothing at all.**

### Why this is the most valuable item, not a cosmetic one

`snub`'s only consideration is `Script("ctx.grudge(actor, target)")`, and `Memories::grudge` reads
`max(0, −valence)` over memories *about* a person. So:

- an exchange-path insult writes `MemoryKind::Interaction` with `valence -0.7`, so it *does* feed the grudge —
  which is why the village probe above shows `grudge 1`;
- a **host-reported** deed writes nothing to the victim, so `ReportDeed` — the surface a host uses for
  everything the society cannot see (a brawl, an affliction, a theft found at dawn) — can never give the
  victim a reason to resent the actor.

And on both paths **affinity is untouched**, so the victim's liking for their insulter is set entirely by
mere exposure. That is the behaviour a host watches and calls wrong.

It beats the runner-up (`examples/benchmark` builds an unsittable home, which makes every perf number in the
repository the cost of a society that cannot sleep) because that one is a measurement hazard in a harness,
while this is a person in the simulation liking the person who hurt them.

## What the literature says

### 1. A benefit and a harm are not symmetric — the crate already cites this

**Baumeister, Bratslavsky, Finkenauer & Vohs 2001**, *Bad Is Stronger Than Good*, DOI
`10.1037/1089-2680.5.4.323`. Corpus doc `10.1037_1089-2680.5.4.323`, verified on disk. Already cited by
`snub` in `assets/starter.society.ron`, and already used by `Tuning::rival_band`.

On reciprocity of affect in dyads (p. 22):

> "Reciprocation patterns were also examined by Wills et al. (1974). Interspouse correlations indicated that
> **negative, displeasurable behaviors were reciprocated to a significant degree**, whereas the reciprocation
> of positive, pleasurable behaviors was weaker and not significant."

And the magnitude asymmetry, quoted for its own number rather than its direction (p. 33):

> "Gottman (1994) has proposed a revealing diagnostic index … positive and good interactions must outnumber the
> negative and bad ones by at least **five to one** … Gottman's index suggests that bad events are on average
> **five times as powerful** as good ones."

This is the number the crate is missing. It already carries `witness_gain` (0.2), `witness_dilution` (0.5) and
`0.15 · approval` for a witness's affinity write — all of them **symmetric in `harm`**: a `+0.6` gift and a
`-0.6` theft move a witness by the same magnitude. A single coefficient of five on the harmful side is what the
paper reports, and nothing in the crate implements it for the person the deed was *done to*.

### 2. Gratitude promotes the relationship; anger does not undo it symmetrically

**Ogolsky, Monk, Rice, Theisen & Maniotes 2017**, *Relationship Maintenance: A Review of Research on Romantic
Relationships*, DOI `10.1111/jftr.12205`. Corpus doc `10.1111_jftr.12205`, verified on disk, p. 10:

> "Gratitude, or being thankful and expressing appreciation, is considered crucial for relationship well-being
> and **may motivate future reciprocity** in relationships"

The crate already appraises `Gratitude`/`Anger` at `|harm|` in `patient_reaction`, which is that mechanism's
emotional half. What it lacks is the *relational* half: the benefit buys liking, and — by the paper above —
the harm costs more than the benefit buys.

### 3. The corpus has no rate for the victim's affinity, and two searches confirm it

`distill_search` for *"victim's liking of the aggressor decreases after being harmed negative affect
reciprocity"* returned a domestic-violence policy paper and an autism-credibility paper at 0.50 and 0.50, both
irrelevant. A second, *"reciprocity of interpersonal attraction: liking is reciprocated, dislike is
reciprocated"*, returned a hookup study at 0.50. A third, for *"negative reciprocity retaliation"*, found
**Gottman's negative-affect reciprocity model** (via a converted 1993 paper) and **Pruitt 2011**, *Conflict,
Escalation and De-escalation of*, DOI `10.1002/9780470672532.wbepp054`, which states the mechanism in one line:

> "in distressed relationships even trivial conflicts tend to escalate, with each partner retaliating in
> response to the other's retaliatory behavior"

Both give the **direction** and neither gives a rate. So the shape comes from Baumeister's 5:1 and the
magnitude is a **design default**, recorded as one.

## The formula

The choice is whether the victim's affinity write is (a) a new rate, (b) the same `0.15 · approval` the witness
uses, or (c) the witness's write scaled by Baumeister's 5.

- A new `Tuning` field for the victim's affinity weight would be a **second** number where the crate already has
  one, and no paper distinguishes a victim's weighting from a bystander's.
- `0.15 · approval` verbatim contradicts the paper's whole subject: the person harmed must be moved *more*.
- Baumeister's number is the one the paper supplies, and it is a ratio, so it composes with the existing write
  rather than competing with it.

So: **the patient's affinity moves by `Tuning::witness_affinity · approval · severity_sign`, where `harmful`
deeds are scaled by `Tuning::negativity_ratio` (5.0, Baumeister/Gottman) and helpful ones are not** — with
`witness_affinity = 0.15` hoisted out of `witness_reaction`'s literal so both sides read one field.

`approval` for the patient is `norm · sign(harm)` — the same judgement `witness_reaction` makes, because a victim
who values honesty judges a theft by the same norm everybody else does, and the deed's own `harm` already carries
the sign. A victim of a deed they approve of (a `help` that rescued them) gains affinity; one of a deed they
disapprove of loses it.

`severity` is `|harm|`, so a `-0.7` theft already costs more than a `-0.4` insult before the ratio is applied:
a witnessed insult is `-0.4`, a theft is `-0.7`, and the ratio takes the victim's write to `-0.15 · 5 = -0.75 ·
|harm|` — three times a witness's, which is the paper's claim in its weakest defensible form.

### And the memory the host-reported path never wrote

`patient_reaction` gains the `Witnessed`-shaped memory the victim is owed. Without it `ReportDeed` gives a
victim no grudge, which is what makes `snub` unreachable after a host-reported injury. The memory is recorded
as `MemoryKind::Interaction` **only on the exchange path** (which already does it in `behavior.rs`) and as a new
`MemoryKind::Suffered { deed, actor }` on the deed path, because those are different facts: one is a
conversation that went badly, the other is something that was *done to you*.

That last part is a **schema change to a public enum**, so it goes to `needs-decision/` rather than being
decided here. The affinity write does not, and ships.

## What believable output looks like

- A victim of an accepted `insult` has **negative** affinity for the insulter within one tick, and holds a
  grudge, so `snub` can fire against them.
- An accepted `help` or `gift` raises the recipient's affinity more than the same deed raises a bystander's.
- The village's friend/rival balance does not lurch: the write is one tick's worth per event, and
  `affinity_ceiling` (0.35) still bounds every tie. A sweep is how that is checked, not an assumption.

## Sources

- Baumeister, R. F., Bratslavsky, E., Finkenauer, C. & Vohs, K. D. (2001). Bad Is Stronger Than Good. *Review of
  General Psychology* 5(4), 323–370. DOI `10.1037/1089-2680.5.4.323`. Corpus doc
  `10.1037_1089-2680.5.4.323`, pp. 1, 22, 33 — checked on disk. Supplies the direction, the "five to one" index
  and the "five times as powerful" magnitude; **already cited by the crate**, in `snub`.
- Ogolsky, B. G., Monk, J. K., Rice, T. M., Theisen, J. C. & Maniotes, C. R. (2017). Relationship Maintenance: A
  Review of Research on Romantic Relationships. DOI `10.1111/jftr.12205`. Corpus doc `10.1111_jftr.12205`, p. 10
  — checked on disk. Gratitude "may motivate future reciprocity"; the emotional half `patient_reaction` already
  implements.
- Pruitt, D. G. (2011). Conflict, Escalation and De-escalation of. DOI `10.1002/9780470672532.wbepp054`. Corpus
  doc `10.1002_9780470672532.wbepp054` — "each partner retaliating in response to the other's retaliatory
  behavior". Direction only; no rate.
- Wills, T. A., Weiss, R. L. & Patterson, G. R. (1974), as reported in Baumeister et al. p. 22 — negative
  behaviours reciprocated "to a significant degree", positive ones "weaker and not significant".
  **Cited through Baumeister, not converted**; recorded as such.
- Gottman, J. M. (1994), as reported in Baumeister et al. p. 33 — the 5:1 index.
  **Cited through Baumeister, not converted**; recorded as such.

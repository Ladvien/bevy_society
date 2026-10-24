# Research: a grudge was the count of offences, not the worst one

## How it started

Orient step: read the narrative for `rumor-crosses-a-cluster` (12 people, seed 11). It read

```
d1 20h17: started compliment
d1 20h29: started snub
d1 20h35: started compliment
d1 20h48: started snub
...
```

and `snub` was **170 of 338** activity starts — half a person's day, including a run of **ten in a row**, six
minutes apart, which is exactly one `snub` duration. `snub` **satisfies no need** (`satisfies: []`), so
nothing pushed back.

## The mechanism

`snub`'s only consideration is `ctx.grudge(actor, target)`. The grudge was

```
grudge = clamp(Σ max(0, −valence) · recency(h), 0, 1)
```

and the memory magnitude was `0.3 + harm.abs()`, which for `insult` (`harm: -0.4`) is **0.7**. Two memories
clamp to 1.0, so `snub`'s `Linear(1.0, 0.0)` turned a saturated grudge into a permanent score of 1.000.

Two things kept it saturated:

1. **The sum.** Any accumulated grievance history reaches 1.0, however small the individual offences.
2. **`memory_capacity`.** Because `Memories` keeps a bounded buffer, how many entries happened to survive
   decided the grudge. Four retained trivial slights equalled one betrayal.

Measured: mean grudge across the cluster sat at **0.061–0.076** while the **maximum was 1.000** throughout,
with 5–8 negative memories about a single person.

## Also found: a refused insult scored as an injury

`magnitude = 0.3 + harm.abs()` used the deed's `harm`, but the **deed only fires on accept**. So a snub the
target *refused* still wrote `-0.7` into the snubber — a memory of an injury that never happened, sized at
0.7, about the very person whose refusal motivated the next one. Fixed to `0.3` when refused (asserted:
`tests/snub.rs`, 0.700 → 0.300).

That fix alone changed no aggregate, because four 0.3 memories still summed to 1.0. It is a correctness fix,
not the lever.

## The literature

**Sommerfeld et al. 2007**, *Gossip as an alternative for direct observation in games of indirect
reciprocity* (PNAS, `10.1073/pnas.0704598104`) frames gossip as reputation information rather than
affiliation — relevant because it is why `snub` carries an `insult` deed and a reputation trace at all.

**Bondü & Richter 2016**, DOI `10.3389/fpsyg.2016.00795` is the pack's own citation for `snub`: rejection
and provocation sensitivity predict reactive aggression — a reaction to how one has been treated, which is
`Memories::grudge`.

The corpus search for "does one strong offence dominate, or do many weak ones accumulate" returned nothing
that settles it. There is no paper here for `max` over `sum`, and the honest thing is to say so: **this is a
design default**, chosen because `sum` makes the value a function of buffer capacity, which is not a
psychological quantity. The mechanism it models — an opinion of a person set by their worst treatment of you
rather than by the count of small ones — is ordinary and needs no citation to be legible; the number does.

## What believable output looks like

A person's day contains a **mix**: work, chat, play, gossip, and the occasional snub, with the worst
treatment being a rare event that does not recur every six minutes. Not `snub, compliment, snub, compliment`
for an evening.

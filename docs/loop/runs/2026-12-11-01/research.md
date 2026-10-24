# Research: five mechanisms, one cause

## Why this

Last iteration concluded the lever was "the threshold and its inputs, not the gain on affinity". This iteration
tested that directly, twice, and both failed — which turned a hypothesis into a structural finding.

## The measured distributions

End of `village-evening`, seed 7, 24 people, 17 game-days, 168 directed ties:

| | affinity | trust | familiarity |
|---|---|---|---|
| friend-ties (115) | **0.792** | **0.480** | 0.907 |
| other ties (53) | −0.223 | 0.068 | 0.908 |

Trust separates friends from the rest by a factor of seven, and familiarity does not separate them at all
(0.907 against 0.908). That is why a `Friend` rule needs a trust term — and why a trust *gate* looked right.

## Two tests, both negative

| test | result |
|---|---|
| `Friend` also requires `trust ≥ 0.6` | **22 friends, 33 rivals — unchanged** |
| contact talk at 3 h + `trust ≥ 0.35` gate | **267 friends, 0 rivals** |

The first changed nothing because 0.6 is above most friends' trust (mean 0.480). The second changed nothing
because `chat` pays **`trust: 0.05` per accepted exchange**, so contact talk at ~8 exchanges a day adds
**+0.4 trust per day** — trust saturates exactly as affinity does.

**A gate on trust is a gate on a quantity the same mechanism saturates.**

## And saturation cannot help either

Saturating bounds a variable at **1.0**. Every threshold in `classify` is at or below 0.4. So saturation changes
how fast 1.0 is approached and never whether 0.4 is crossed.

| configuration | friends | rivals |
|---|---|---|
| saturate affinity alone | 22 | 38 |
| saturate affinity alone **with contact talk** | **260** | **0** |
| saturate **all three** accumulators (affinity, trust, respect) | 23 | 43 |
| saturate all three **with contact talk** | **268** | **0** |

## The structural conclusion

**No tie variable can sit below a threshold for a pair that has had contact.** All four accumulate, and every
exchange supplies all of them, so any *level*-based gate is passed by contact volume.

The remaining lever is a bound **below** the threshold — an asymptote at `K < 0.4`, so that no amount of contact
with one person is enough on its own — which is the one property none of the five failures had.

## Sources

No paper supplies any of these shapes; all were design defaults. The two mechanisms the thresholds rest on —
mere exposure (Zajonc 1968, DOI `10.1037/h0025848`) and similarity (Byrne 1961, DOI `10.1037/h0044721`) — are
cited in `classify`'s own doc and are unchanged.

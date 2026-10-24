# Research: a villager spends 62% of its life walking

## How it surfaced

Last iteration reduced reported-asleep from 56.8% to 36.4% by fitting the travel timeout. That number was
still above the 33.3% one 8-hour sleep a day allows, so this iteration made the evidence harness distinguish
**travelling** from **doing** — a measurement fix, `src/` untouched:

```
travelling:sleep        21.8%
sleep                   14.7%
travelling:eat          12.6%
travelling:work         12.4%
work                     9.1%
...
```

| | share of all person-ticks |
|---|---|
| **travelling** | **62.5%** |
| doing an activity | 37.5% |
| of which `sleep` | **14.7%** — against 33.3% that one 8-hour sleep allows |

`evidence/travelling.png`: red fills most of every column for the whole run.

## The arithmetic

`village-evening` runs at 64 s a day with a 1/64 s tick, so walking is `1.4 / 64 = 0.0219` m a tick and the
village is 8.2 m across. **A crossing walk is 375 ticks.** The starter pack's activities, in ticks:

| activity | ticks | gates on `Distance`? |
|---|---|---|
| `snub` | **17** | no |
| `compliment` | **34** | no |
| `chat` | **51** | yes |
| `gossip` | **51** | yes |
| `play` | **85** | yes |
| `eat` | **128** | no |
| `relax` | **256** | no |
| `work` | 683 | no |
| `sleep` | 1 365 | no |

**Five activities are shorter than the walk that reaches them, and three of those five are the only ones
that consider distance at all.** A person crosses the village for a 128-tick meal and a 256-tick relax,
neither of which knows how far away it was.

## What was measured as a candidate fix

Adding `Distance` to `eat`, `relax` and `compliment`, same seed and run:

| | before | after |
|---|---|---|
| travelling | **62.5%** | **50.5%** |
| total need mean | 0.526 | 0.513 |

Twelve points of walking for 1.3% of need. A fair trade and a small one — and it also showed that
`eat` alone gives 56.3%, so the three together are not additive.

## Why that is not enough, and what is

The pack-level change is a **definitions-file** edit, which is a host's file. The larger lever is the
**decision**: for a `Place` target, `targets_for` offers one candidate per matching place with no distance in
the score, so a person is as likely to pick the far tavern as the near one. Adding a consideration to three
activities treats the symptom; scoring the target's distance treats the cause. That is
`needs-decision/travel-cost-is-not-scored.md`, which recommends a crate-level consideration so the term
exists once and is opt-in by name.

## Literature

No new model. Utility-based choice is `docs/research/needs-and-utility.md`, and `Signal::Distance` is
already one of its inputs — Game AI Pro 3 ch. 13 on response curves over normalised inputs. What the pack
does not do is *use* it where a use is short, and the decision does not price a `Place` by how far it is.

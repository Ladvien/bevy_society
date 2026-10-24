# Research: the travel timeout, fitted to the layout

## Where this starts

The previous iteration found that a person walking to bed is reported asleep, and filed it as a decision
because it changes what `Doing.activity` means. That decision stands. But the *scale* of the problem is a
settings value, and that is not a decision: `travel_timeout_hours` was **20 game-hours** for a village whose
longest walk is **2.2**.

At `seconds_per_day = 64`, one game-hour is **170.7 ticks** and walking speed is **0.0219 m a tick**:

| timeout | ticks | reach |
|---|---|---|
| 20 h | 3 413 | **74.7 m** |
| 4 h | 683 | **14.9 m** |
| village diagonal, 8.2 m | 375 | **2.2 h** |

So the timeout allowed 74.7 m of walking in a village 8.2 m across — **nine times** the longest walk — and
an 8-hour sleep is 1 365 ticks against a 3 413-tick timeout. A person who cannot reach its bed is therefore
reported asleep for **two and a half times the length of the sleep**.

## What fitting it does

Same seed, same spawn, 67 200 ticks:

| | 20 h | **4 h** |
|---|---|---|
| reported asleep | **56.8%** | **36.4%** |
| reported working | 14.8% | **21.5%** |
| needs mean | [0.268, 0.538, 0.217, 0.267] | **[0.709, 0.395, 0.402, 0.599]** |
| **total need mean** | **0.322** | **0.526** |

and the activity mix redistributes the way it should: `sleep` 56.8% → 36.4%, `work` 14.8% → 21.5%, `snub`
4.0% → 8.9%, `compliment` 2.8% → 6.9%, `relax` 4.2% → 5.0%.

Holds at a second seed: seed 42 gives **36.2%** asleep and a need total of **0.511**.

`evidence/timeout-fitted.png` — the 4-hour line stays near the 8-per-day mark while the 20-hour line spikes
to 22, and the need mean sits at 0.50 against 0.30 for the whole run.

**Rest falls** (0.538 → 0.395) and that is correct: people no longer sleep for hours they have not earned.

## Two harness bugs found while measuring

### `contact_radius` was silently ignored

`rumor-crosses-a-cluster` declares `contact_radius: 4.0`. The spec reader mapped only `seconds_per_day`,
`start_hour` and `travel_timeout_hours`, so the value was never applied — right by coincidence, since 4.0
is also the crate default, and wrong for any other number.

### The file was parsed once per tick

`plant_at(scenario)` was called **inside the tick loop** and parses `scenarios.yaml` each time, printing a
warning for every `settings:` key it does not read. A 67 200-tick run emitted **57 000 warnings**. Hoisted
out: zero.

## What the cluster measurement then showed

`rumor-crosses-a-cluster` reaches **0.333** at 12 hours and **0.167** at 20 — both below the 40% its
observable asks for. The **0.583** recorded two iterations ago was taken before `contact_radius` was
applied, so it was a different run; that is now clear rather than mysterious.

The gap is real and is a design question the file names: two groups 40 m apart need a **10.7 game-hour
walk**, so either the gap is smaller, the timeout longer than a day, or a bridge activity exists. The
scenario is `pending`.

## Literature

No new model. `travel_timeout_hours` is documented as a design default, and this iteration fits it to the
layout it is used in — the same discipline the crate asks of every other number.

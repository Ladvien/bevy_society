# Pass 2026-10-31-02 — body clocks: every person sleeps in the same two hours

## Why this item

No open issues (`gh issue list` returns nothing); parked branches (passes 20-40) are older, ignored. Candidates, ranked:

1. **Nobody has a body clock of their own** (ranked item 2, "everyone doing the same thing"). Measured on the shipped pack, village seed 7, 17 game-days (`evidence/before/`, regenerated with the command in `eval.md`): of 450 `sleep` starts, **280 begin in the 22:00 hour and 81 in the 23:00 hour (80 % in two hours)**; every other activity's start histogram has the same shape, a morning spike at 06:00-07:00 when all 24 wake together and an evening ramp into the one bedtime. `sleep` is chosen on `Hour(22, 6)`, one curve for everyone. The crate already knows this is a problem and says so twice: `warn_unreachable_places` ("They all go to bed at the same hour, so the queue behind the disc times out and their sleeps are interrupted") and the comment in `arrive_and_satisfy` ("Everyone sleeps at the same hour, so every resident ..."). 63 of 450 sleeps (14 %) end interrupted. `docs/research/personality-and-mood.md` lists "no circadian effects" as a known gap.
2. Compliment repetition (one person praising the same neighbour nine times in an evening; 34 % of exchanges repeat the previous partner): measured, but pass 3 of 2026-10-30 already tried a need gate and `friendship-has-no-diminishing-return.md` records six failed shapes. No new source, so not this pass.
3. Re-record the README clips: needs a display; logged, not workable here.

Pick 1: it is visible to anyone who watches a village at night, it is the documented cause of interrupted sleeps, it needs no change to any model already pinned (a person with no `Chronotype` is bit-identical to today), and it has a clean literature.

## Sources (home-still, DOIs verified with `paper_search`/`distill_search` today)

- **Roenneberg, Kuehnle, Juda, et al. 2007**, *Epidemiology of the human circadian clock*, Sleep Medicine Reviews 11, DOI `10.1016/j.smrv.2007.07.005`. Chronotype = phase of entrainment, measured by the **mid-point of sleep on free days** (MSF); the Munich ChronoType Questionnaire.
- **Fischer, Lombardi, Marucci-Wellman & Roenneberg 2017**, *Chronotypes in the US*, PLoS ONE 12(6):e0178782, DOI `10.1371/journal.pone.0178782` (read in full today; n = 53 689 diaries, American Time Use Survey). The distribution of mid-sleep is **near-normal** overall and within every age group, "with a slight positive skew"; range almost 10 h (00:00 to 09:53); 25 % earlier than 02:24, 50 % between 02:24 and 04:15, 25 % later than 04:15 (interquartile range 1.85 h, so a normal fit has SD 1.85 / 1.349 = **1.37 h**); standard deviation by age and sex runs 1.57-2.65 h (adults 25-65: 1.98-2.49 h, diary noise included); mean 3:26 ± 2.31 h; sleep duration 8.1 ± 1.5 h and **unrelated to chronotype** (b = -0.0004), so an early type is not a short sleeper.
- **Reiter, Sargent & Roach 2021**, DOI `10.3390/clockssleep3020021` (corpus `10.3390_clockssleep3020021`): MSFsc n = 57 adults, 04:38 ± 1:11 h, range 02:22-07:45 (the extremes sit 2.3 h before and 3.1 h after the mean).
- **Sogawa et al. 2023**, DOI `10.5334/jcr.225` (corpus `10.5334_jcr.225`): Japanese sample n = 362, MSF 4:22 ± 1:36 h, MSW (work days) 3:33 ± 1:06 h.
- **Wittmann, Dinich, Merrow & Roenneberg 2006**, *Social jetlag: misalignment of biological and social time*, Chronobiology International 23, DOI `10.1080/07420520500545979`. The body clock and the **social clock** (work, school, shop hours) are two different clocks; the gap between them is what is called social jetlag. This is the reason `Hour` stays the wall clock and a *second* input is added rather than shifting `Hour`.
- **Roenneberg, Pilz, Zerbini & Winnebeck 2019**, *Chronotype and social jetlag: a (self-) critical review*, Biology 8(3):54, DOI `10.3390/biology8030054`. MSFsc as the definition; free-day sleep as the natural timing.

## The model

A person has one number, their **chronotype offset** `c` in hours: how far their body clock runs behind the wall clock. `c > 0` is an owl, `c < 0` a lark, `c = 0` the clock everyone has today. The hour on their own clock is

```
body_hour(clock_hour, c) = (clock_hour - c) mod 24
```

so a person with `c = +2` finds `Hours(22, 6)` open at 00:00 wall time and wakes at 08:00. Fischer et al.'s own worked example is the same arithmetic ("assuming an 8 h sleep duration, a chronotype of 4:00 falls asleep at midnight and wakes up at 08:00"): the pack's `Hours(22, 6)` window is centred on 02:00, so `c` is a person's mid-sleep minus 02:00.

- Population: `c ~ Normal(0, sd)`, truncated at three standard deviations. **`sd` = 1.5 h is a design default**: it sits between the interquartile-implied 1.37 h (Fischer 2017), Reiter 2021's 1.18 h and Sogawa 2023's 1.6 h, and below the 2-2.5 h that adult diary data show once recall noise is included. The mean is left at 0 so the pack's authored window stays the village's average night (the published mean is a property of a country and a latitude, not of the pack).
- Sleep length stays the pack's authored 8 h: chronotype is independent of sleep duration (Fischer 2017, b = -0.0004).
- What does **not** shift: `Hour` (shop hours, the baker's dawn, the working day) is the social clock. A lark who wakes at 04:00 has the hours until the workshop opens to fill; an owl who wakes at 09:00 starts work late, and that mismatch is social jetlag (Wittmann 2006), left to the pack's authors to use or not.

## Consequence the model must carry: a sleeper is not a conversation partner

With uniform schedules the village is asleep together or awake together, so nobody ever stands next to someone sleeping. With body clocks an owl awake at 01:00 stands in a home place with seven sleeping neighbours, and `ActivityTarget::Person` offers them as targets. The village would compliment its sleepers. So a definition may mark an activity `unavailable`: a person doing it is in nobody's `near` list, and an activity walking toward a person who has fallen asleep ends. This is the crate's own `Dormant` rule (out of perception, no exchanges) applied per activity instead of per marker, and it is data, not code: the starter pack marks `sleep`.

## What believable output looks like

- Sleep starts spread over about four hours (the 22:00-hour share falls from 62 % toward the share of a normal distribution's central hour, about 25-30 %), and the share of the village asleep at a given clock hour rises and falls smoothly where it is now a step.
- Interrupted sleeps fall (the queue at a home place is no longer a queue).
- A narrative reads as a person with their own day: a lark's first meal at 04:30, with nobody about; an owl compliments someone at 01:00 who is awake.
- Nobody is the target of an exchange while asleep.
- A person with `Chronotype(0.0)` is bit-identical to the crate before this change.

## Not done, and why

- Chronotype is not derived from personality: the Big Five explain a few per cent of its variance, far too little to produce the spread the data show, and a derived value would hide a free parameter inside a trait. The host (or `PlacementFile`) writes it, as it writes a personality.
- No age or sex dependence (Fischer 2017 gives both): the crate has neither.
- No homeostatic sleep pressure beyond the existing `rest` need, and no free-running drift: the pack's `rest` curve already stops a rested person from choosing sleep.

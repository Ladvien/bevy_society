# Research - a week: `Input::Week`, `Curve::Days`, `SocietySettings::start_weekday`

## The gap

The crate has an hour of the day (`Input::Hour`, `Input::BodyHour`, `Curve::Hours`, `ctx.hour()`) and a count of game-days (`SocietyClock::day`, `ctx.day()`), and **no week**. `grep -i weekday|weekend|day_of_week src docs/guide docs/research` finds nothing. So every game-day of every society is the same day: the workshop opens at 08:00 on all of them, nobody has a day of rest, a market or a Sunday visit can only be written as a Rhai `Input::Script("ctx.day() % 7 ...")`, which costs a script evaluation per consideration per person per tick, is not validated at load, and which an editor cannot show as a window the way it shows `Hours(8, 17)`.

Watching the village makes it visible: `docs/loop/runs/2026-10-31-03/evidence/narratives/person3-s42-6days-before.txt` is six days of the same day. A host that wants "people rest on the seventh day" has no first-class way to say it, and the crate has no scenario in which the answer to "what did they do on a rest day" differs from "what did they do on a workday".

## Sources (all read in the corpus, 2026-10-31)

- **Sekara, Stopczynski & Lehmann 2016**, *Fundamental structures of dynamic social networks*, PNAS 113(36), DOI `10.1073/pnas.1602803113` (already cited in `docs/research/relationships.md` for close-tie networks that change over time; this is the first use of its supplement on predictability by weekday). A university-student population with phone location and proximity data. Read from the supplement (Fig. S30): "any other day, while during weekends we are more stationary. If we, however, consider the total number of distinct visited places we see that **Fridays and Saturdays are special because those days are used to explore new locations**. Therefore, predicting location during weekends based on routine is more difficult, since we have higher entropy during these periods. Our social behavior resembles our mobility, where we socialize mainly during the day and less during the night. **Weekends are again special, interestingly we here observe a drop in the number of social states, because we are not required to go to work or school.** Fig. S30d shows that the number of social states decreases during weekends, meaning our participants **reserve weekends to socialize with a few selected friends**."
- **Dodds, Harris, Kloumann, Bliss & Danforth 2011**, *Temporal patterns of happiness and information in a global social network: hedonometrics and Twitter*, PLoS ONE 6(12):e26752, DOI `10.1371/journal.pone.0026752` (Twitter). "We see Saturday has the highest average happiness (havg ~ 6.06), closely followed by Friday and then Sunday. From Saturday, we see a steady decline until the weekly low occurs on Tuesday, which is then followed by small increases on both Wednesday and Thursday (havg ~ 6.03)"; "the three day midweek low followed by a peak around Saturday" holds in each of four time ranges of the data. **Not modelled** (see below): it is the existence of a seven-day cycle in what people feel and do, not its size, that this change needs.
- Not relied on, not read in full: Servia-Rodriguez et al. 2017, DOI `10.1145/3038912.3052618` (phone sensing: weekday and weekend location and sociability panels). Wittmann et al. 2006, DOI `10.1080/07420520500545979` (social jetlag, in `chronotype.md`): the free-day shift of sleep is **not** modelled here.

## What the sources license, and what they do not

They license a **weekly rhythm with two kinds of day**: days on which a person is required to be somewhere (work, school) and days on which they are not, and on the free days **less movement and a narrower circle of company: a few selected friends** (Sekara), against a workday spent with whoever the workplace and the route give. They give a direction, not a number: no paper here says what share of a rest day is visiting. So the rest-day activity in the example overlay is a **design default** in size and a sourced one in direction, and says so.

They do not license: a seven-day week as a law of nature (it is a convention; the crate's `DAYS_PER_WEEK` is 7 and a host with another calendar writes a script), the names of days (the crate counts 0-6 from the day the host calls the first), or a mood cycle (in the text read, Dodds' happiness is 6.06 on the best day and about 6.03 on Wednesday and Thursday: a weekly effect too small to be a parameter the crate could calibrate).

## The model (what is built)

1. **`Curve::Days(from, to)`**: `1` inside the wrapping window `[from, to)` over the week, measured in days (`Days(5.0, 7.0)` is the last two days; `Days(4.75, 6.0)` is Friday 18:00 to the end of Saturday; `Days(6.0, 1.0)` wraps), the exact analogue of `Hours` on a 7-day clock.
2. **`Input::Week`**: the position in the week, `(weekday + hour / 24) / 7`, in `[0, 1)`. It carries the hour too, so a window can start mid-day, and a boundary lands on the midnight tick. Valid only with `Days` (and `Days` only with it), as `Hours` is with `Hour` and `BodyHour`.
3. **`SocietySettings::start_weekday`** (0-6, default 0): the weekday of game-day 0. `SocietyClock::weekday(start)` and `ctx.weekday()` (an integer 0-6) read it.
4. **The week is the village's clock, not the body's**: a body clock shifts `Hour`, never `Week` (a rest day is a social fact).
5. **Opt-in overlay** `examples/assets/weekly.society.ron`: `work` shut on the last two days (`Days(0, 5)` on the workday) and a rest-day `visit` (a long `chat` with someone well liked, `Affinity` strongly weighted), the shape Sekara describes. The starter pack is unchanged, so every existing pin and the default village are byte-for-byte what they were.

## Pre-registered predictions (to be tested, not assumed)

- P1. On the overlay, work and craft person-ticks are **zero on rest days** and equal their baseline share on workdays (trivial, by construction; the check is that nothing else leaks).
- P2. **Fewer distinct partners per person on rest days than on workdays**, and a **larger share of exchanges with close ties** (Sekara's "a few selected friends"). It could fail: removing work frees six hours for chatting with whoever is near, which would raise the count. If it fails, the overlay's `visit` is mis-sized or the claim does not transfer, and the note says which.
- P3. The society as a whole is **not damaged**: friend and rival ties, insults, and the need floor stay within the seed-to-seed spread of the same seeds without the overlay.
- P4. Same seed with the overlay off is byte-identical to `main` (`summary.json`, `series.csv`, `narrative.txt`).

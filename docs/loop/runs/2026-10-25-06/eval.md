# Evaluation: a task is not a skill

Instrument: `examples/evidence.rs` at `village-evening`'s own settings (24 people, `seconds_per_day 64.0`,
`start_hour 15.0`, `travel_timeout_hours 4.0`, `tick_secs 1/64`, 67 200 ticks = 16.4 game-days), plus
`tests/profession.rs` for the mechanism.

## What shipped

`skills` in the definitions file, `profession` on an activity, and `Input::Skill` as a consideration. A
skill's level is a Big Five trait, so nothing is stored.

## The mechanism, measured

One bench, two competing tasks (`craft`, which the skill scores, and `potter`, which it does not), three
villagers differing only in conscientiousness:

| conscientiousness | craft skill | craft starts | of all starts |
|---|---|---|---|
| 0.95 | 0.95 | **5** | 10 |
| 0.50 | 0.50 | **4** | 10 |
| 0.05 | 0.05 | **0** | 10 |

The least diligent villager never chooses the craft task. `tests/profession.rs` pins that, and pins that a
0.01-conscientiousness villager still takes the task when it is the only one available — comparative
advantage, not a licence.

**The first version of this measurement "proved" the skill did nothing: 10 / 10 / 10 starts.** The fixture
had a single activity, and with one option every villager takes it whatever it scores. The rival task is
what makes the test bite, and the failure is why the test file says so in a comment.

## The regression that stopped a bigger change

Applying the same `Skill("craft")` consideration to the starter pack's **`work`** — the village's dominant
daytime activity — was measured on three seeds and rejected:

| seed | friends | rivals | rumor reach |
|---|---|---|---|
| 7 | 16 → 17 | 24 → 20 | 0.08 → 0.29 |
| 42 | 18 → 11 | 29 → **55** | 0.50 → **0.08** |
| 11 | 11 → 12 | 30 → 37 | 0.67 → 0.13 |

Making the dominant activity depend on a trait reshuffles the entire day: people who used to work now do
something else, and the gossip network that rode on the workplace fragments. The mechanic ships on an
**additive** `craft` activity instead:

| seed | friends | rivals | rumor reach |
|---|---|---|---|
| 7 | 16 → 18 | 24 → 26 | 0.08 → 0.33 |
| 42 | 18 → 20 | 29 → 33 | 0.50 → 0.08 |
| 11 | 11 → 14 | 30 → 27 | 0.67 → 0.04 |

Still not free — seed 11's reach falls — and that is recorded rather than argued away. `profession:` alone,
with no consideration, is exactly inert (16 / 24 / 0.083 at seed 7), so the field is metadata until a
consideration uses it.

## What is not good enough

1. **A skill cannot improve.** The level is a trait reading, constant for a life. Learning needs stored
   state and a `SocietySave` field.
2. **`Profession(String)` is a label with no mechanism.** Nothing in the simulation reads it; it exists for
   scripts and barks, and the doc comment says so. That is deliberately the smallest honest thing.
3. **No economy.** No money, no inventory, no price. The design and its prerequisites are in
   `needs-decision/professions-and-economics.md`.
4. **The trait-to-trade mapping has no source.** `Craft` → conscientiousness is the crate's own choice;
   three lines in `SkillGroup::trait_of` so a host can see and disagree with it in one place.

## Determinism

`tests/profession.rs::the_skill_is_deterministic`. The skill adds no draw and no stored state: it is a pure
function of a trait that is itself fixed at spawn.

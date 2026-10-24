//! The society as it stood at the start of a tick: the read side of scripts.
//!
//! [`build_snapshot`] runs in `SocietySystems::Perceive` and freezes every person's state into an
//! [`Arc`]. Scripts read it through `ctx` and never see the live world; their writes go through
//! the `fx` queue instead (plan D8). Their random draws come from a generator seeded once per tick
//! from [`SocietyRng`], so scripts stay deterministic.

use std::sync::{Arc, Mutex};

use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_emerge_rng::{DetRng, seeded};
use rand_chacha::ChaCha8Rng;

use crate::behavior::Doing;
use crate::beliefs::{Knowledge, Opinions};
use crate::core::{PersonId, PersonIndex, SocietyClock, SocietyRng, SocietySettings};
use crate::defs::{
    ActivityId, DeedId, FactionId, IDLE_ACTIVITY, InteractionId, NeedId, SocietyDefs, TopicId,
};
use crate::groups::{Memberships, Status};
use crate::mind::{Emotions, Memories, Mood, Needs, Personality};
use crate::relationships::{TieLabel, Ties};
use crate::rumor::RumorId;

/// Names of every compiled definition, in id order. Scripts name things; the snapshot maps each
/// name to the id of its definition, which is its position in the list.
#[derive(Clone, Default)]
pub(crate) struct Names {
    /// Need names, indexed by [`NeedId`].
    pub(crate) needs: Vec<String>,
    /// Topic names, indexed by [`TopicId`].
    pub(crate) topics: Vec<String>,
    /// Deed names, indexed by [`DeedId`].
    pub(crate) deeds: Vec<String>,
    /// Faction names, indexed by [`FactionId`].
    pub(crate) factions: Vec<String>,
    /// Activity names, indexed by [`ActivityId`].
    pub(crate) activities: Vec<String>,
    /// Interaction names, indexed by [`InteractionId`].
    pub(crate) interactions: Vec<String>,
}

impl Names {
    /// Copies the names out of compiled definitions.
    pub(crate) fn from_defs(defs: &SocietyDefs) -> Self {
        Self {
            needs: defs.needs.iter().map(|s| s.name.clone()).collect(),
            topics: defs.topics.iter().map(|s| s.name.clone()).collect(),
            deeds: defs.deeds.iter().map(|s| s.name.clone()).collect(),
            factions: defs.factions.iter().map(|s| s.name.clone()).collect(),
            activities: defs.activities.iter().map(|s| s.name.clone()).collect(),
            interactions: defs.interactions.iter().map(|s| s.name.clone()).collect(),
        }
    }

    /// The need called `name`.
    pub(crate) fn need(&self, name: &str) -> Result<NeedId, String> {
        position(&self.needs, "need", name).map(|i| NeedId(i as u16))
    }

    /// The topic called `name`.
    pub(crate) fn topic(&self, name: &str) -> Result<TopicId, String> {
        position(&self.topics, "topic", name).map(|i| TopicId(i as u16))
    }

    /// The deed called `name`.
    pub(crate) fn deed(&self, name: &str) -> Result<DeedId, String> {
        position(&self.deeds, "deed", name).map(|i| DeedId(i as u16))
    }

    /// The faction called `name`.
    pub(crate) fn faction(&self, name: &str) -> Result<FactionId, String> {
        position(&self.factions, "faction", name).map(|i| FactionId(i as u16))
    }

    /// The activity called `name`.
    pub(crate) fn activity(&self, name: &str) -> Result<ActivityId, String> {
        position(&self.activities, "activity", name).map(|i| ActivityId(i as u16))
    }

    /// The interaction called `name`.
    pub(crate) fn interaction(&self, name: &str) -> Result<InteractionId, String> {
        position(&self.interactions, "interaction", name).map(|i| InteractionId(i as u16))
    }

    /// The name of an activity, `""` for the idle activity or an unknown id.
    pub(crate) fn activity_name(&self, id: ActivityId) -> &str {
        if id == IDLE_ACTIVITY {
            return "";
        }
        self.activities
            .get(id.0 as usize)
            .map_or("", String::as_str)
    }
}

/// Index of `name` in `names`, or an error naming the unknown kind.
fn position(names: &[String], kind: &str, name: &str) -> Result<usize, String> {
    names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| format!("unknown {kind} '{name}'"))
}

/// One directed tie as scripts see it, with familiarity already evaluated at the tick start.
#[derive(Clone, Copy)]
pub(crate) struct TieView {
    /// The person the tie points at, for lookups by id.
    pub(crate) target: PersonId,
    /// The same person as an entity, for the lookups the `ctx` API makes.
    pub(crate) entity: Entity,
    /// Liking in `[-1, 1]`.
    pub(crate) affinity: f32,
    /// Trust in `[0, 1]`.
    pub(crate) trust: f32,
    /// Respect in `[-1, 1]`.
    pub(crate) respect: f32,
    /// Familiarity in `[0, 1]`, after power-law forgetting.
    pub(crate) familiarity: f32,
    /// The label at the tick start.
    pub(crate) label: TieLabel,
}

/// One person as scripts see them, copied at the tick start.
#[derive(Clone)]
pub(crate) struct PersonView {
    /// The entity the person lives on. Effects resolve ids to it.
    pub(crate) entity: Entity,
    /// The person's id.
    pub(crate) id: PersonId,
    /// The `Name` component, when present.
    pub(crate) name: Option<String>,
    /// Stable personality traits.
    pub(crate) personality: Personality,
    /// Current mood.
    pub(crate) mood: Mood,
    /// Active emotions.
    pub(crate) emotions: Emotions,
    /// Need levels.
    pub(crate) needs: Needs,
    /// Opinions by topic.
    pub(crate) opinions: Opinions,
    /// Outgoing ties, ascending by target id; ties to despawned persons are dropped.
    pub(crate) ties: Vec<TieView>,
    /// Rumors this person knows.
    pub(crate) knowledge: Knowledge,
    /// Factions this person belongs to.
    pub(crate) memberships: Memberships,
    /// Status: respect conferred by others.
    pub(crate) standing: f32,
    /// The activity in progress, `None` when no `Doing` exists.
    pub(crate) activity: Option<ActivityId>,
    /// Memories, used for grudges.
    pub(crate) memories: Memories,
}

impl PersonView {
    /// The tie this person holds toward `target`, looked up by entity as the `ctx` API does.
    pub(crate) fn tie_to_id(&self, target: Entity) -> Option<&TieView> {
        self.ties
            .binary_search_by_key(&target.to_bits(), |t| t.entity.to_bits())
            .ok()
            .map(|i| &self.ties[i])
    }
}

/// The frozen society read by scripts during one tick.
pub(crate) struct SocietySnapshot {
    now_days: f64,
    hour: f32,
    day: i64,
    names: Names,
    /// Persons in ascending id order.
    persons: Vec<PersonView>,
    /// Per-tick generator for `ctx.rand()`, seeded from [`SocietyRng`].
    rng: Mutex<ChaCha8Rng>,
}

impl SocietySnapshot {
    /// Hour of the game-day at the tick start, in `[0, 24)`.
    pub(crate) fn hour(&self) -> f32 {
        self.hour
    }

    /// Whole game-days elapsed at the tick start.
    pub(crate) fn day(&self) -> i64 {
        self.day
    }

    /// The person with id `id`, or an error naming the id.
    pub(crate) fn person(&self, id: i64) -> Result<&PersonView, String> {
        let not_found = || format!("no person with id {id}");
        let key = u32::try_from(id).map_err(|_| not_found())?;
        self.persons
            .binary_search_by_key(&key, |p| p.id.0)
            .map(|i| &self.persons[i])
            .map_err(|_| not_found())
    }

    /// A uniform draw in `[0, 1)` from the tick's script generator.
    pub(crate) fn unit(&self) -> f32 {
        let mut rng = match self.rng.lock() {
            Ok(rng) => rng,
            Err(poisoned) => poisoned.into_inner(),
        };
        rng.unit() as f32
    }

    /// `ctx.affinity(a, b)`: liking in `[-1, 1]`, 0 when there is no tie.
    pub(crate) fn affinity(&self, a: i64, b: i64) -> Result<f32, String> {
        self.tie_value(a, b, |tie| tie.affinity, 0.0)
    }

    /// `ctx.trust(a, b)`: trust in `[0, 1]`, 0.5 when there is no tie (a fresh tie's value).
    pub(crate) fn trust(&self, a: i64, b: i64) -> Result<f32, String> {
        self.tie_value(a, b, |tie| tie.trust, 0.5)
    }

    /// `ctx.respect(a, b)`: respect in `[-1, 1]`, 0 when there is no tie.
    pub(crate) fn respect(&self, a: i64, b: i64) -> Result<f32, String> {
        self.tie_value(a, b, |tie| tie.respect, 0.0)
    }

    /// `ctx.familiarity(a, b)`: familiarity in `[0, 1]` at the tick start, 0 when there is no tie.
    pub(crate) fn familiarity(&self, a: i64, b: i64) -> Result<f32, String> {
        self.tie_value(a, b, |tie| tie.familiarity, 0.0)
    }

    /// `ctx.grudge(a, b)`: the weight of `a`'s bad memories about `b`, in `[0, 1]`.
    pub(crate) fn grudge(&self, a: i64, b: i64) -> Result<f32, String> {
        let person = self.person(a)?;
        let other = self.entity(b)?;
        Ok(person.memories.grudge(other, self.now_days))
    }

    /// `ctx.label(a, b)`: the tie's label as a lowercase name, `"stranger"` when there is no tie.
    pub(crate) fn label(&self, a: i64, b: i64) -> Result<String, String> {
        let person = self.person(a)?;
        let other = self.entity(b)?;
        Ok(person
            .tie_to_id(other)
            .map_or(crate::relationships::TieLabel::Stranger, |tie| tie.label)
            .script_name()
            .to_owned())
    }

    /// `ctx.mood(a)`: the PAD triple `[pleasure, arousal, dominance]`.
    pub(crate) fn mood(&self, a: i64) -> Result<[f32; 3], String> {
        Ok(self.person(a)?.mood.as_array())
    }

    /// `ctx.emotion(a, kind)`: the summed intensity of one emotion kind, over every target. An
    /// unknown name is an error naming it.
    pub(crate) fn emotion(&self, a: i64, kind: &str) -> Result<f32, String> {
        let kind = crate::mind::EmotionKind::from_name(kind)
            .ok_or_else(|| format!("unknown emotion '{kind}'"))?;
        Ok(self.person(a)?.emotions.intensity_of(kind))
    }

    /// `ctx.need(a, id)`: a need's level in `[0, 1]`. An unknown name is an error naming it.
    pub(crate) fn need(&self, a: i64, need: &str) -> Result<f32, String> {
        let id = self.names.need(need)?;
        Ok(self
            .person(a)?
            .needs
            .0
            .get(usize::from(id.0))
            .copied()
            .unwrap_or(0.0))
    }

    /// `ctx.opinion(a, topic)`: an opinion in `[-1, 1]`. An unknown name is an error naming it.
    pub(crate) fn opinion(&self, a: i64, topic: &str) -> Result<f32, String> {
        let id = self.names.topic(topic)?;
        Ok(self.person(a)?.opinions.value(usize::from(id.0)))
    }

    /// `ctx.trait(a, name)`: a personality trait in `[0, 1]`. An unknown name is an error naming it.
    pub(crate) fn trait_value(&self, a: i64, name: &str) -> Result<f32, String> {
        let which =
            crate::mind::Trait::from_name(name).ok_or_else(|| format!("unknown trait '{name}'"))?;
        Ok(self.person(a)?.personality.value(which))
    }

    /// `ctx.knows(a, rumor)`: whether `a` knows the rumor with that id.
    pub(crate) fn knows(&self, a: i64, rumor: i64) -> Result<bool, String> {
        let id = u32::try_from(rumor)
            .map(RumorId)
            .map_err(|_| format!("no rumor with id {rumor}"))?;
        Ok(self.person(a)?.knowledge.knows(id))
    }

    /// `ctx.member(a, faction)`: whether `a` belongs to that faction. An unknown name is an error
    /// naming it.
    pub(crate) fn member(&self, a: i64, faction: &str) -> Result<bool, String> {
        let id = self.names.faction(faction)?;
        Ok(self.person(a)?.memberships.contains(id))
    }

    /// `ctx.faction_size(faction)`: how many persons belong to it. An unknown name is an error
    /// naming it.
    pub(crate) fn faction_size(&self, faction: &str) -> Result<i64, String> {
        let id = self.names.faction(faction)?;
        let count = self
            .persons
            .iter()
            .filter(|p| p.memberships.contains(id))
            .count();
        Ok(count as i64)
    }

    /// `ctx.standing(a)`: respect conferred by familiar persons, in `[-1, 1]`.
    pub(crate) fn standing(&self, a: i64) -> Result<f32, String> {
        Ok(self.person(a)?.standing)
    }

    /// `ctx.name(a)`: the person's `Name`, or `"someone"` when it has none.
    pub(crate) fn name(&self, a: i64) -> Result<String, String> {
        Ok(self
            .person(a)?
            .name
            .clone()
            .unwrap_or_else(|| "someone".to_owned()))
    }

    /// `ctx.doing(a)`: the activity's name, `""` when the person is idle.
    pub(crate) fn doing(&self, a: i64) -> Result<String, String> {
        Ok(self
            .person(a)?
            .activity
            .map_or_else(String::new, |id| self.names.activity_name(id).to_owned()))
    }

    /// `ctx.persons()`: every live person's id, ascending.
    pub(crate) fn person_ids(&self) -> Vec<i64> {
        self.persons.iter().map(|p| i64::from(p.id.0)).collect()
    }

    fn person_entity(&self, id: i64) -> Result<Entity, String> {
        self.person(id).map(|p| p.entity)
    }

    fn entity(&self, id: i64) -> Result<Entity, String> {
        self.person_entity(id)
    }

    fn tie_value(
        &self,
        a: i64,
        b: i64,
        read: impl Fn(&TieView) -> f32,
        absent: f32,
    ) -> Result<f32, String> {
        let person = self.person(a)?;
        let other = self.entity(b)?;
        Ok(person.tie_to_id(other).map_or(absent, &read))
    }
}

/// The components the snapshot copies out of each person.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct SnapshotSource<'w> {
    personality: &'w Personality,
    mood: &'w Mood,
    emotions: &'w Emotions,
    needs: &'w Needs,
    ties: &'w Ties,
    opinions: &'w Opinions,
    knowledge: &'w Knowledge,
    memberships: &'w Memberships,
    status: &'w Status,
    memories: &'w Memories,
    doing: Option<&'w Doing>,
    name: Option<&'w Name>,
}

/// The current snapshot, rebuilt every tick by [`build_snapshot`]. `None` before the first tick.
#[derive(Resource, Default)]
pub(crate) struct SnapshotCell(pub(crate) Option<Arc<SocietySnapshot>>);

/// Freezes the society for scripts. Runs in `SocietySystems::Perceive`.
pub(crate) fn build_snapshot(
    settings: Res<SocietySettings>,
    clock: Res<SocietyClock>,
    defs: Option<Res<SocietyDefs>>,
    index: Res<PersonIndex>,
    mut rng: ResMut<SocietyRng>,
    mut cell: ResMut<SnapshotCell>,
    people: Query<SnapshotSource>,
) {
    let names = defs
        .as_deref()
        .map_or_else(Names::default, Names::from_defs);
    let tuning = &settings.tuning;
    let now = clock.now_days;

    // Entity bits to person id, sorted for binary search: ties name their target by entity.
    let mut by_bits: Vec<(u64, PersonId)> = index
        .sorted
        .iter()
        .map(|&(id, entity)| (entity.to_bits(), id))
        .collect();
    by_bits.sort_unstable();

    let mut persons = Vec::with_capacity(index.sorted.len());
    for &(id, entity) in &index.sorted {
        let Ok(view) = people.get(entity) else {
            continue;
        };
        let (
            personality,
            mood,
            emotions,
            needs,
            ties,
            opinions,
            knowledge,
            memberships,
            status,
            memories,
            doing,
            name,
        ) = (
            view.personality,
            view.mood,
            view.emotions,
            view.needs,
            view.ties,
            view.opinions,
            view.knowledge,
            view.memberships,
            view.status,
            view.memories,
            view.doing,
            view.name,
        );
        let mut views: Vec<TieView> = ties
            .0
            .iter()
            .filter_map(|tie| {
                let key = tie.target.to_bits();
                let slot = by_bits.binary_search_by_key(&key, |&(bits, _)| bits).ok()?;
                Some(TieView {
                    target: by_bits[slot].1,
                    entity: tie.target,
                    affinity: tie.affinity,
                    trust: tie.trust,
                    respect: tie.respect,
                    familiarity: tie.familiarity(now as f32, tuning),
                    label: tie.label,
                })
            })
            .collect();
        views.sort_by_key(|t| t.entity.to_bits());
        persons.push(PersonView {
            entity,
            id,
            name: name.map(|n| n.as_str().to_string()),
            personality: *personality,
            mood: *mood,
            emotions: emotions.clone(),
            needs: needs.clone(),
            opinions: opinions.clone(),
            ties: views,
            knowledge: knowledge.clone(),
            memberships: memberships.clone(),
            standing: status.standing,
            activity: doing.map(|d| d.activity),
            memories: memories.clone(),
        });
    }

    let seed = rng.0.raw_u64();
    cell.0 = Some(Arc::new(SocietySnapshot {
        now_days: now,
        hour: clock.hour(),
        day: i64::from(clock.day()),
        names,
        persons,
        rng: Mutex::new(seeded(seed)),
    }));
}

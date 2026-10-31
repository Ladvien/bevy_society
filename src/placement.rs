//! Authored placement: who lives where, as data that saves, reloads and spawns.
//!
//! [`DefsFile`](crate::DefsFile) says what *kinds* of things exist (needs, factions, activities).
//! A [`PlacementFile`] says which places and which persons a world starts with: each person's name,
//! personality, starting needs, opinions, factions, home, named bound places (a workplace is
//! `bound: [("work", "mill")]`), start position and ties to other authored persons. It is the shape an
//! editor saves and a game loads; the runtime snapshot ([`crate::SocietySave`]) is a different thing
//! (it holds memories, activities in progress and velocities).
//!
//! The file is plain serde data, both ways: [`PlacementFile::from_ron`] and
//! [`PlacementFile::to_ron`]. [`PlacementFile::spawn`] checks the whole file against the world's
//! [`SocietyDefs`] first and spawns nothing when any line is wrong, so a half-built town never
//! exists. Spawning draws no random numbers: the same file gives the same entities, ids and
//! positions in the same order.

use std::collections::BTreeMap;

use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::world::World;
use bevy::math::Vec2;
use bevy::transform::components::Transform;
use serde::{Deserialize, Serialize};

use bevy::ecs::query::With;

use crate::beliefs::Opinions;
use crate::core::{
    Autonomous, BoundPlaces, Home, Person, PersonId, PersonIndex, Place, PlaceTags, SocietySettings,
};
use crate::defs::{DefsError, NeedId, SignalId, SocietyDefs, TopicId};
use crate::groups::Memberships;
use crate::mind::{Chronotype, Needs, Personality, Signals};
use crate::relationships::{Kin, Ties};

/// A list of places and persons, as one RON document. Both lists are optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PlacementFile {
    /// Places persons live at and work at.
    pub places: Vec<PlaceDef>,
    /// Persons, spawned in this order (so ids follow it).
    pub persons: Vec<PersonDef>,
}

/// A place: a named spot with a capacity and tags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceDef {
    /// Unique name; persons refer to it.
    pub id: String,
    /// Position on the society plane.
    pub at: (f32, f32),
    /// How many persons may do an activity there at once ([`Place::capacity`]).
    pub capacity: u32,
    /// Tags activities find it by ([`PlaceTags`]).
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A person's five traits, each in `[0, 1]`. A trait left out is `0.5`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PersonalityDef {
    /// Openness to experience.
    pub openness: f32,
    /// Conscientiousness.
    pub conscientiousness: f32,
    /// Extraversion.
    pub extraversion: f32,
    /// Agreeableness.
    pub agreeableness: f32,
    /// Neuroticism.
    pub neuroticism: f32,
}

impl Default for PersonalityDef {
    fn default() -> Self {
        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        }
    }
}

/// A directed tie from the owning person to another authored person.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TieSpec {
    /// Name of the other person.
    pub with: String,
    /// Starting liking in `[-1, 1]` (see [`crate::Ties::entry_with`] on the ceiling).
    pub affinity: f32,
    /// Starting reliance in `[0, 1]`; `0.5`, a new acquaintance's, when left out.
    #[serde(default = "default_trust")]
    pub trust: f32,
    /// What the other person is to this one when they are blood relatives ([`Kin`]), `None` when left out.
    /// Relatedness is symmetric, so spawning answers it: the other person's tie to this one gets the
    /// [converse](Kin::converse) relation when it names none (and is created when it does not exist), and a tie
    /// that names a different relation is refused. Both ties start at a familiarity of `1.0`, because a relative
    /// is someone known from the start.
    #[serde(default)]
    pub kin: Option<Kin>,
}

fn default_trust() -> f32 {
    0.5
}

/// The familiarity a relative's tie starts at: someone known from the start, not met.
const KIN_FAMILIARITY: f32 = 1.0;

fn default_autonomous() -> bool {
    true
}

/// One authored person.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonDef {
    /// Unique name; becomes the entity's [`Name`] and what ties refer to.
    pub name: String,
    /// Traits; all `0.5` when left out.
    #[serde(default)]
    pub personality: PersonalityDef,
    /// How many hours the person's body clock runs behind the wall clock ([`Chronotype`]): positive is an
    /// owl, negative a lark; within ±12; `0.0` when left out.
    #[serde(default)]
    pub chronotype_hours: f32,
    /// Starting need levels by need id; the others start at their `initial`.
    #[serde(default)]
    pub needs: Vec<(String, f32)>,
    /// Starting signal values by signal id.
    #[serde(default)]
    pub signals: Vec<(String, f32)>,
    /// Starting opinions by topic id (also their anchors).
    #[serde(default)]
    pub opinions: Vec<(String, f32)>,
    /// Faction ids the person belongs to.
    #[serde(default)]
    pub factions: Vec<String>,
    /// Name of the person's home place ([`Home`]).
    #[serde(default)]
    pub home: Option<String>,
    /// Named bound places, `(binding, place name)` ([`BoundPlaces`]); `("work", "mill")` is a workplace.
    #[serde(default)]
    pub bound: Vec<(String, String)>,
    /// Where they start; the home's position when left out, else the origin.
    #[serde(default)]
    pub at: Option<(f32, f32)>,
    /// Ties to other authored persons.
    #[serde(default)]
    pub ties: Vec<TieSpec>,
    /// Whether the person chooses their own activities ([`Autonomous`]); `true` when left out.
    #[serde(default = "default_autonomous")]
    pub autonomous: bool,
}

/// What [`PlacementFile::spawn`] made, in file order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Placed {
    /// Each place's name and entity.
    pub places: Vec<(String, Entity)>,
    /// Each person's name and entity.
    pub persons: Vec<(String, Entity)>,
}

impl Placed {
    /// The place called `name`.
    pub fn place(&self, name: &str) -> Option<Entity> {
        self.places.iter().find(|(n, _)| n == name).map(|(_, e)| *e)
    }

    /// The person called `name`.
    pub fn person(&self, name: &str) -> Option<Entity> {
        self.persons
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, e)| *e)
    }
}

fn invalid(owner: String, message: impl Into<String>) -> DefsError {
    DefsError::Invalid {
        owner,
        message: message.into(),
    }
}

fn unique<'a>(
    kind: &'static str,
    names: impl Iterator<Item = &'a String>,
) -> Result<BTreeMap<&'a str, usize>, DefsError> {
    let mut seen = BTreeMap::new();
    for (i, name) in names.enumerate() {
        if seen.insert(name.as_str(), i).is_some() {
            return Err(DefsError::Duplicate {
                kind,
                id: name.clone(),
            });
        }
    }
    Ok(seen)
}

fn in_range(owner: &str, what: &str, v: f32, lo: f32, hi: f32) -> Result<(), DefsError> {
    if v.is_finite() && (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(invalid(
            owner.to_owned(),
            format!("{what} must be in [{lo}, {hi}], got {v}"),
        ))
    }
}

impl PlacementFile {
    /// Parses a placement file from RON text.
    pub fn from_ron(text: &str) -> Result<Self, DefsError> {
        ron::from_str(text).map_err(|err| DefsError::Ron(err.to_string()))
    }

    /// Writes the file as pretty RON; [`PlacementFile::from_ron`] reads it back equal.
    pub fn to_ron(&self) -> Result<String, DefsError> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|err| DefsError::Ron(err.to_string()))
    }

    /// Checks the file against `defs`: unique names, every reference resolves, every value in range.
    pub fn validate(&self, defs: &SocietyDefs) -> Result<(), DefsError> {
        let places = unique("place", self.places.iter().map(|p| &p.id))?;
        let persons = unique("person", self.persons.iter().map(|p| &p.name))?;
        for place in &self.places {
            let owner = format!("place '{}'", place.id);
            if !place.at.0.is_finite() || !place.at.1.is_finite() {
                return Err(invalid(owner, "position must be finite"));
            }
        }
        let place_ref = |owner: &str, name: &str| {
            if places.contains_key(name) {
                Ok(())
            } else {
                Err(DefsError::UnknownId {
                    kind: "place",
                    id: name.to_owned(),
                    owner: owner.to_owned(),
                })
            }
        };
        for person in &self.persons {
            let owner = format!("person '{}'", person.name);
            let p = person.personality;
            in_range(
                &owner,
                "chronotype_hours",
                person.chronotype_hours,
                -12.0,
                12.0,
            )?;
            for (what, v) in [
                ("openness", p.openness),
                ("conscientiousness", p.conscientiousness),
                ("extraversion", p.extraversion),
                ("agreeableness", p.agreeableness),
                ("neuroticism", p.neuroticism),
            ] {
                in_range(&owner, what, v, 0.0, 1.0)?;
            }
            if let Some(at) = person.at
                && (!at.0.is_finite() || !at.1.is_finite())
            {
                return Err(invalid(owner, "position must be finite"));
            }
            if let Some(home) = &person.home {
                place_ref(&owner, home)?;
            }
            for (_, place) in &person.bound {
                place_ref(&owner, place)?;
            }
            for tie in &person.ties {
                if !persons.contains_key(tie.with.as_str()) {
                    return Err(DefsError::UnknownId {
                        kind: "person",
                        id: tie.with.clone(),
                        owner,
                    });
                }
                if tie.with == person.name {
                    return Err(invalid(owner, "a person cannot have a tie to themselves"));
                }
                in_range(&owner, "tie affinity", tie.affinity, -1.0, 1.0)?;
                in_range(&owner, "tie trust", tie.trust, 0.0, 1.0)?;
                if let Some(kin) = tie.kin
                    && let Some(back) = self.persons[persons[tie.with.as_str()]]
                        .ties
                        .iter()
                        .find(|back| back.with == person.name)
                    && let Some(theirs) = back.kin
                    && theirs != kin.converse()
                {
                    return Err(invalid(
                        owner,
                        format!(
                            "lists '{}' as {kin:?} but '{}' lists them as {theirs:?}, which is not its converse {:?}",
                            tie.with,
                            tie.with,
                            kin.converse()
                        ),
                    ));
                }
            }
            // Resolving the named values is the check: it reports an id the defs lack.
            defs.needs(person.needs.iter().map(|(n, v)| (n.as_str(), *v)))?;
            defs.signals(person.signals.iter().map(|(n, v)| (n.as_str(), *v)))?;
            defs.opinions(person.opinions.iter().map(|(n, v)| (n.as_str(), *v)))?;
            defs.memberships(person.factions.iter().map(String::as_str))?;
        }
        Ok(())
    }

    /// Spawns every place, then every person, into `world`, in file order.
    ///
    /// Reads [`SocietyDefs`] and [`SocietySettings`] (the plane) from the world; with no `SocietyDefs`
    /// it is an error, and on any validation error nothing is spawned. Each person is a [`Person`]
    /// with their traits, needs, signals, opinions, memberships, [`Home`], [`BoundPlaces`] and ties;
    /// they start at `at`, else at their home, else at the origin.
    pub fn spawn(&self, world: &mut World) -> Result<Placed, DefsError> {
        let defs = world
            .get_resource::<SocietyDefs>()
            .ok_or_else(|| invalid("placement".to_owned(), "the world has no SocietyDefs"))?;
        self.validate(defs)?;
        // Validated above, so every lookup below resolves; a miss would be a bug here, and is
        // skipped rather than panicking.
        let mut bundles = Vec::with_capacity(self.persons.len());
        for person in &self.persons {
            bundles.push((
                defs.needs(person.needs.iter().map(|(n, v)| (n.as_str(), *v)))?,
                defs.signals(person.signals.iter().map(|(n, v)| (n.as_str(), *v)))?,
                defs.opinions(person.opinions.iter().map(|(n, v)| (n.as_str(), *v)))?,
                defs.memberships(person.factions.iter().map(String::as_str))?,
            ));
        }
        let plane = world
            .get_resource::<SocietySettings>()
            .map(|s| s.plane)
            .unwrap_or_default();

        let mut placed = Placed::default();
        let mut spot: BTreeMap<&str, (f32, f32)> = BTreeMap::new();
        for place in &self.places {
            let entity = world
                .spawn((
                    Name::new(place.id.clone()),
                    Place {
                        capacity: place.capacity,
                    },
                    PlaceTags(place.tags.clone()),
                    Transform::from_translation(plane.lift(Vec2::new(place.at.0, place.at.1))),
                ))
                .id();
            spot.insert(place.id.as_str(), place.at);
            placed.places.push((place.id.clone(), entity));
        }

        for (person, (needs, signals, opinions, memberships)) in self.persons.iter().zip(bundles) {
            let start = person
                .at
                .or_else(|| person.home.as_deref().and_then(|h| spot.get(h).copied()))
                .unwrap_or((0.0, 0.0));
            let mut entity = world.spawn((
                Person,
                Name::new(person.name.clone()),
                Transform::from_translation(plane.lift(Vec2::new(start.0, start.1))),
                Personality {
                    openness: person.personality.openness,
                    conscientiousness: person.personality.conscientiousness,
                    extraversion: person.personality.extraversion,
                    agreeableness: person.personality.agreeableness,
                    neuroticism: person.personality.neuroticism,
                },
                Chronotype(person.chronotype_hours),
                needs,
                signals,
                opinions,
                memberships,
            ));
            if person.autonomous {
                entity.insert(Autonomous);
            }
            if let Some(home) = person.home.as_deref().and_then(|h| placed.place(h)) {
                entity.insert(Home(home));
            }
            if !person.bound.is_empty() {
                let mut bound = BoundPlaces::default();
                for (binding, name) in &person.bound {
                    if let Some(place) = placed.place(name) {
                        bound.bind(binding.clone(), place);
                    }
                }
                entity.insert(bound);
            }
            placed.persons.push((person.name.clone(), entity.id()));
        }

        // Every person's ties are built before any is attached, because a relative's tie is answered on the
        // other side: relatedness is symmetric, so one authored `kin` makes both directions.
        let slot: BTreeMap<&str, usize> = self
            .persons
            .iter()
            .enumerate()
            .map(|(i, person)| (person.name.as_str(), i))
            .collect();
        let mut built: Vec<Ties> = vec![Ties::default(); self.persons.len()];
        for (i, person) in self.persons.iter().enumerate() {
            for spec in &person.ties {
                if let Some(target) = placed.person(&spec.with) {
                    let tie = built[i].entry_with(target, spec.affinity);
                    tie.trust = spec.trust;
                    if spec.kin.is_some() {
                        tie.kin = spec.kin;
                        tie.familiarity_peak = KIN_FAMILIARITY;
                    }
                }
            }
        }
        for person in &self.persons {
            for spec in &person.ties {
                if let (Some(kin), Some(&other), Some(owner)) = (
                    spec.kin,
                    slot.get(spec.with.as_str()),
                    placed.person(&person.name),
                ) {
                    let back = built[other].entry(owner);
                    if back.kin.is_none() {
                        back.kin = Some(kin.converse());
                        back.familiarity_peak = KIN_FAMILIARITY;
                    }
                }
            }
        }
        for (person, ties) in self.persons.iter().zip(built) {
            if ties.is_empty() {
                continue;
            }
            if let Some(owner) = placed.person(&person.name) {
                world.entity_mut(owner).insert(ties);
            }
        }
        Ok(placed)
    }

    /// Reads the world back into a file: the inverse of [`PlacementFile::spawn`], for an editor that
    /// saves what the user arranged live.
    ///
    /// Every [`Place`] and every [`Person`] must carry a [`Name`], and the names must be unique
    /// (checked by [`PlacementFile::validate`] on the result), else the error names the entity.
    /// Places come out sorted by name, persons in ascending `PersonId`. A person's `at` is where they
    /// stand now, `needs` and `signals` are their current levels, and `opinions` are their anchors
    /// (the authored, Friedkin–Johnsen starting value; drifted values and stubbornness are runtime
    /// state that belongs in [`crate::SocietySave`]). Opinions anchored at `0.0` and signals at `0.0`
    /// are left out, as that is what an unlisted id starts at. Ties to an entity that is not a named
    /// person are dropped.
    pub fn capture(world: &mut World) -> Result<Self, DefsError> {
        let plane = world
            .get_resource::<SocietySettings>()
            .map(|s| s.plane)
            .unwrap_or_default();
        let mut place_names: BTreeMap<Entity, String> = BTreeMap::new();
        let mut places = Vec::new();
        let mut query = world.query::<(
            Entity,
            &Place,
            Option<&Name>,
            Option<&PlaceTags>,
            &Transform,
        )>();
        for (entity, place, name, tags, transform) in query.iter(world) {
            let name = name.map(|n| n.as_str().to_owned()).ok_or_else(|| {
                invalid(format!("place {entity}"), "a captured place needs a Name")
            })?;
            let at = plane.project(transform.translation);
            place_names.insert(entity, name.clone());
            places.push(PlaceDef {
                id: name,
                at: (at.x, at.y),
                capacity: place.capacity,
                tags: tags.map(|t| t.0.clone()).unwrap_or_default(),
            });
        }
        places.sort_by(|a, b| a.id.cmp(&b.id));

        let mut found: Vec<(PersonId, Entity)> = world
            .query_filtered::<(&PersonId, Entity), With<Person>>()
            .iter(world)
            .map(|(id, e)| (*id, e))
            .collect();
        found.sort();
        let mut person_names: BTreeMap<Entity, String> = BTreeMap::new();
        for &(_, entity) in &found {
            let name = world.get::<Name>(entity).ok_or_else(|| {
                invalid(format!("person {entity}"), "a captured person needs a Name")
            })?;
            person_names.insert(entity, name.as_str().to_owned());
        }

        let defs = world
            .get_resource::<SocietyDefs>()
            .ok_or_else(|| invalid("placement".to_owned(), "the world has no SocietyDefs"))?;
        let mut persons = Vec::with_capacity(found.len());
        for &(_, entity) in &found {
            let owner = format!("person '{}'", person_names[&entity]);
            let place_name = |target: Entity| {
                place_names.get(&target).cloned().ok_or_else(|| {
                    invalid(
                        owner.clone(),
                        "its home or bound place is not a named Place",
                    )
                })
            };
            let e = world.entity(entity);
            let personality = e
                .get::<Personality>()
                .map_or_else(PersonalityDef::default, |p| PersonalityDef {
                    openness: p.openness,
                    conscientiousness: p.conscientiousness,
                    extraversion: p.extraversion,
                    agreeableness: p.agreeableness,
                    neuroticism: p.neuroticism,
                });
            let needs = e.get::<Needs>().map_or_else(Vec::new, |n| {
                (0..n.0.len())
                    .map(|i| (defs.need_name(NeedId(i as u16)).to_owned(), n.0[i]))
                    .collect()
            });
            let signals = e.get::<Signals>().map_or_else(Vec::new, |s| {
                (0..s.0.len())
                    .filter(|&i| s.0[i] != 0.0)
                    .map(|i| (defs.signal_name(SignalId(i as u16)).to_owned(), s.0[i]))
                    .collect()
            });
            let opinions = e.get::<Opinions>().map_or_else(Vec::new, |o| {
                (0..o.0.len())
                    .filter(|&i| o.0[i].anchor != 0.0)
                    .map(|i| (defs.topic_name(TopicId(i as u16)).to_owned(), o.0[i].anchor))
                    .collect()
            });
            let factions = e.get::<Memberships>().map_or_else(Vec::new, |m| {
                m.0.iter()
                    .map(|&f| defs.faction_name(f).to_owned())
                    .collect()
            });
            let home = e.get::<Home>().map(|h| place_name(h.0)).transpose()?;
            let mut bound = Vec::new();
            if let Some(b) = e.get::<BoundPlaces>() {
                for (binding, target) in &b.0 {
                    bound.push((binding.clone(), place_name(*target)?));
                }
            }
            let at = e
                .get::<Transform>()
                .map(|t| plane.project(t.translation))
                .map(|p| (p.x, p.y));
            let mut ties = Vec::new();
            if let Some(t) = e.get::<Ties>() {
                for tie in &t.0 {
                    if let Some(with) = person_names.get(&tie.target) {
                        ties.push(TieSpec {
                            with: with.clone(),
                            affinity: tie.affinity,
                            trust: tie.trust,
                            kin: tie.kin,
                        });
                    }
                }
                ties.sort_by(|a, b| a.with.cmp(&b.with));
            }
            persons.push(PersonDef {
                name: person_names[&entity].clone(),
                personality,
                chronotype_hours: e.get::<Chronotype>().map_or(0.0, |c| c.0),
                needs,
                signals,
                opinions,
                factions,
                home,
                bound,
                at,
                ties,
                autonomous: e.contains::<Autonomous>(),
            });
        }
        let file = Self { places, persons };
        file.validate(defs)?;
        Ok(file)
    }
}

/// One person who depends on a place: it is their home or one of their bound places.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceDependent {
    /// The person.
    pub person: Entity,
    /// Their `Name`, when they have one.
    pub name: Option<String>,
    /// `"home"` for [`Home`], otherwise the binding name of a [`BoundPlaces`] entry (`"work"`).
    pub role: String,
}

/// Everyone whose home or bound place is `place`, in ascending `PersonId` order, so an editor can
/// refuse to delete a building that someone lives in or works at and name them. A person holding
/// the place under several roles appears once per role.
pub fn place_dependents(world: &World, place: Entity) -> Vec<PlaceDependent> {
    let Some(index) = world.get_resource::<PersonIndex>() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for &(_, person) in &index.sorted {
        let name = world.get::<Name>(person).map(|n| n.as_str().to_owned());
        if world.get::<Home>(person).is_some_and(|h| h.0 == place) {
            out.push(PlaceDependent {
                person,
                name: name.clone(),
                role: "home".to_owned(),
            });
        }
        if let Some(bound) = world.get::<BoundPlaces>(person) {
            for (binding, target) in &bound.0 {
                if *target == place {
                    out.push(PlaceDependent {
                        person,
                        name: name.clone(),
                        role: binding.clone(),
                    });
                }
            }
        }
    }
    out
}

/// Persons whose position on the society plane lies inside the rectangle `min..=max`, in ascending
/// `PersonId` order: the selection an editor binds a trigger or a bulk edit to.
pub fn persons_in_rect(world: &World, min: Vec2, max: Vec2) -> Vec<Entity> {
    let (Some(index), Some(settings)) = (
        world.get_resource::<PersonIndex>(),
        world.get_resource::<SocietySettings>(),
    ) else {
        return Vec::new();
    };
    index
        .sorted
        .iter()
        .filter_map(|&(_, person)| {
            let p = settings
                .plane
                .project(world.get::<Transform>(person)?.translation);
            (p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y).then_some(person)
        })
        .collect()
}

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

use crate::core::{
    Autonomous, BoundPlaces, Home, Person, PersonIndex, Place, PlaceTags, SocietySettings,
};
use crate::defs::{DefsError, SocietyDefs};
use crate::mind::Personality;
use crate::relationships::Ties;

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
}

fn default_trust() -> f32 {
    0.5
}

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

        for person in &self.persons {
            if person.ties.is_empty() {
                continue;
            }
            let mut ties = Ties::default();
            for spec in &person.ties {
                if let Some(target) = placed.person(&spec.with) {
                    ties.entry_with(target, spec.affinity).trust = spec.trust;
                }
            }
            if let Some(owner) = placed.person(&person.name) {
                world.entity_mut(owner).insert(ties);
            }
        }
        Ok(placed)
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

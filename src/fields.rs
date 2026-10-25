//! Social fields: where persons gather, and where the talk is (feature `fields`, plan D9).
//!
//! Two stigmergic grids from `bevy_stigmergy`. The *crowd* grid has channel 0 (presence of everyone)
//! and channel 1 (commotion from witnessed deeds). Each faction has its own grid whose channel 0 is the
//! presence of its members. Every tick, each person deposits its presence, every witnessed deed
//! deposits its harm as commotion, and then every grid evaporates and diffuses. Daily-life options read
//! the fields through [`SocialFields::field_value`]. A `Gradient` option climbs a field through
//! [`SocialFields::gradient_target`], and `OwnShare` is Schelling's tolerance input.
//!
//! Deposits are order-dependent `+=`, so they run in `PersonIndex.sorted` order.

use bevy::prelude::*;
use bevy_stigmergy::{CellFrame, ChannelDef, StigGrid};

use crate::core::require_core;
use crate::{
    FactionId, FieldInput, Memberships, Person, PersonIndex, SocietyClock, SocietyDefs,
    SocietySettings, SocietySystems,
};

/// Where the fields sit in the world and how they behave. Changing it rebuilds the fields empty.
#[derive(Resource, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource)]
pub struct SocialFieldSettings {
    /// The world↔cell mapping: where the grid starts, how wide a cell is, and how many cells.
    pub frame: CellFrame,
    /// Presence channel: where persons are. Faction grids use this channel too.
    pub presence: ChannelDef,
    /// Commotion channel: `|harm|` of each witnessed deed, deposited at the actor's cell.
    pub commotion: ChannelDef,
}

impl Default for SocialFieldSettings {
    fn default() -> Self {
        Self {
            frame: CellFrame::new(Vec2::new(-32.0, -32.0), 1.0, 64, 64),
            presence: ChannelDef {
                // A fraction per game-day: the grids are stepped with `dt_days`.
                evaporate: 6.0,
                diffuse: 0.1,
                deposit_radius: 1.5,
            },
            commotion: ChannelDef {
                evaporate: 12.0,
                diffuse: 0.2,
                deposit_radius: 2.5,
            },
        }
    }
}

/// Per-channel peak values, cached after each tick. They are the normalisers of
/// [`SocialFields::field_value`].
#[derive(Debug, Clone, Default)]
struct Peaks {
    presence: f32,
    commotion: f32,
    factions: Vec<f32>,
}

/// The grids that daily-life options read. Built from [`SocialFieldSettings`] and the faction count, and
/// rebuilt empty whenever the settings change or the faction count differs. Not `Reflect`, because
/// `bevy_stigmergy`'s grids are plain data without it.
#[derive(Resource)]
pub struct SocialFields {
    /// Channel 0: presence of every person. Channel 1: commotion from witnessed deeds.
    pub crowd: StigGrid<2>,
    /// One grid per faction, indexed by `FactionId.0`. Channel 0 is the presence of its members.
    pub factions: Vec<StigGrid<1>>,
    frame: CellFrame,
    peaks: Peaks,
}

impl SocialFields {
    /// Empty fields for `faction_count` factions on the grid described by `settings`.
    pub fn new(settings: &SocialFieldSettings, faction_count: usize) -> Self {
        let frame = settings.frame;
        // `CellFrame::cells` is ascending row-major, which is what `StigGrid::new` requires.
        let cells: Vec<IVec2> = frame.cells().collect();
        let crowd = StigGrid::new(
            frame.width,
            frame.height,
            cells.iter().copied(),
            [settings.presence, settings.commotion],
        );
        let factions = (0..faction_count)
            .map(|_| {
                StigGrid::new(
                    frame.width,
                    frame.height,
                    cells.iter().copied(),
                    [settings.presence],
                )
            })
            .collect();
        Self {
            crowd,
            factions,
            frame,
            peaks: Peaks {
                presence: 0.0,
                commotion: 0.0,
                factions: vec![0.0; faction_count],
            },
        }
    }

    /// The cell containing a plane position, through the frame's own mapping. Positions off the grid
    /// map to cells that read as zero.
    pub fn world_to_cell(&self, p: Vec2) -> IVec2 {
        self.frame.world_to_cell(p)
    }

    /// The frame the grids currently use. It is a copy, so a host reading it does not need the
    /// resource borrowed.
    pub fn frame(&self) -> CellFrame {
        self.frame
    }

    /// The value of `input` at `cell` for an actor whose faction memberships are `own`, in `[0, 1]`.
    /// Crowd, commotion and the faction inputs divide the cell value by the channel's peak. A zero peak
    /// reads as 0. `OwnShare` is own / (own + other) on raw values, and 0.5 when both are empty
    /// (Schelling 1971, DOI 10.1080/0022250X.1971.9989794).
    pub fn field_value(&self, input: FieldInput, own: &[FactionId], cell: IVec2) -> f32 {
        match input {
            FieldInput::Crowd => normalised(self.crowd.sample_cell(0, cell), self.peaks.presence),
            FieldInput::Commotion => {
                normalised(self.crowd.sample_cell(1, cell), self.peaks.commotion)
            }
            FieldInput::OwnFaction => {
                let (value, peak) = self.faction_sum(own, cell, true);
                normalised(value, peak)
            }
            FieldInput::OtherFactions => {
                let (value, peak) = self.faction_sum(own, cell, false);
                normalised(value, peak)
            }
            FieldInput::OwnShare => {
                let (own_value, _) = self.faction_sum(own, cell, true);
                let (other_value, _) = self.faction_sum(own, cell, false);
                let total = own_value + other_value;
                if total > 0.0 { own_value / total } else { 0.5 }
            }
        }
    }

    /// Where an option `Gradient(input, toward)` should walk from `pos`. The target is four cells
    /// (`4 · cell_size`) from `pos`, along the normalised central-difference gradient of
    /// [`Self::field_value`] at `pos`'s cell. It goes uphill when `toward` is true and downhill
    /// otherwise. `None` on a flat field, which makes that option score zero. Finite differences are used
    /// for every input, because OwnShare is a ratio that has no grid of its own.
    pub fn gradient_target(
        &self,
        input: FieldInput,
        own: &[FactionId],
        pos: Vec2,
        toward: bool,
    ) -> Option<Vec2> {
        let cell = self.world_to_cell(pos);
        let value = |offset: IVec2| self.field_value(input, own, cell + offset);
        let grad = Vec2::new(
            0.5 * (value(IVec2::X) - value(IVec2::NEG_X)),
            0.5 * (value(IVec2::Y) - value(IVec2::NEG_Y)),
        );
        if grad.length_squared() <= 0.0 {
            return None;
        }
        let sign = if toward { 1.0 } else { -1.0 };
        Some(pos + sign * grad.normalize() * 4.0 * self.frame.cell_size)
    }

    /// Sum of the cell values and of the peaks across the factions that match (`member = true`) or do
    /// not match (`member = false`) `own`.
    fn faction_sum(&self, own: &[FactionId], cell: IVec2, member: bool) -> (f32, f32) {
        self.factions
            .iter()
            .enumerate()
            .filter(|(i, _)| own.contains(&FactionId(*i as u16)) == member)
            .fold((0.0_f32, 0.0_f32), |(value, peak), (i, grid)| {
                (
                    value + grid.sample_cell(0, cell),
                    peak + self.peaks.factions.get(i).copied().unwrap_or(0.0),
                )
            })
    }

    /// Recomputes the cached peaks. Called once per tick, after evaporation and diffusion, and once
    /// after a restore so the restored values are readable without waiting for a tick.
    pub(crate) fn refresh_peaks(&mut self) {
        self.peaks.presence = self.crowd.hotspot_cell(0).1;
        self.peaks.commotion = self.crowd.hotspot_cell(1).1;
        self.peaks.factions = self
            .factions
            .iter()
            .map(|grid| grid.hotspot_cell(0).1)
            .collect();
    }
}

/// Raise the commotion field at a point on the society plane.
///
/// The channel is what `FieldInput::Commotion` reads and what a `Gradient(Commotion, true)` activity
/// climbs, so it is how a town's people drift toward trouble. **Two kinds of writer:**
///
/// - a witnessed deed, which deposits `|harm|` at the actor's cell (this module's own writer);
/// - a host, which writes this message directly for a disturbance the society cannot see — a
///   townsperson found afflicted at dawn, a scuffle at the well, a house with a light on.
///
/// `at` is a position on [`SocietySettings::plane`](crate::SocietySettings::plane), not a cell:
/// `SocialFields::world_to_cell` maps it, so a host never needs the grid's geometry. A non-finite or
/// negative `amount`, and a point off the grid, deposit nothing — a commotion is a positive disturbance
/// by definition, and `evaporate_diffuse` never reads a negative one as anything but absence.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Commotion {
    /// Where on the society plane the trouble is.
    pub at: Vec2,
    /// How much commotion to add. Written as `|harm|` by a witnessed deed.
    pub amount: f32,
}

/// Commotions waiting for the next society tick.
///
/// **Drained by an ungated system in `Clock`**, because Bevy keeps a message for two buffer swaps and a
/// society tick is many fixed steps apart: a `Commotion` a host writes in `Update` would be gone before
/// the tick that deposits it. The same pattern `queue_interacts` uses.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct CommotionQueue(pub(crate) Vec<Commotion>);

/// Drains every `Commotion` written since the last fixed step into [`CommotionQueue`].
fn queue_commotion(mut commotions: MessageReader<Commotion>, mut queue: ResMut<CommotionQueue>) {
    for commotion in commotions.read() {
        queue.0.push(*commotion);
    }
}

/// Installs the social fields: the crowd presence and commotion grids, plus one presence grid per
/// faction. They are deposited into and evaporated every tick. Requires
/// [`crate::SocietyCorePlugin`] and is included in `SocietyPlugins` when the `fields` feature is on.
pub struct SocialFieldPlugin;

impl Plugin for SocialFieldPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SocialFieldSettings>()
            .init_resource::<SocialFieldSettings>()
            .add_message::<Commotion>()
            .init_resource::<CommotionQueue>()
            .add_systems(FixedUpdate, sync_fields.in_set(SocietySystems::Clock))
            .add_systems(FixedUpdate, queue_commotion.in_set(SocietySystems::Clock))
            .add_systems(FixedUpdate, deposit_fields.in_set(SocietySystems::Relate));
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "SocialFieldPlugin");
    }
}

/// A value over its channel peak, in `[0, 1]`, or 0 when the peak is zero.
fn normalised(value: f32, peak: f32) -> f32 {
    if peak > 0.0 {
        (value / peak).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Rebuilds the fields when the settings change or the faction count differs. Runs on every fixed step.
fn sync_fields(
    mut commands: Commands,
    settings: Res<SocialFieldSettings>,
    defs: Res<SocietyDefs>,
    fields: Option<Res<SocialFields>>,
) {
    let faction_count = defs.factions.len();
    let stale = match &fields {
        None => true,
        Some(current) => settings.is_changed() || current.factions.len() != faction_count,
    };
    if stale {
        commands.insert_resource(SocialFields::new(&settings, faction_count));
    }
}

/// Deposits presence and commotion, then evaporates and diffuses every grid (plan D9). Tick-gated
/// through the `Relate` set.
fn deposit_fields(
    fields: Option<ResMut<SocialFields>>,
    clock: Res<SocietyClock>,
    society: Res<SocietySettings>,
    index: Res<PersonIndex>,
    persons: Query<(&GlobalTransform, &Memberships), With<Person>>,
    mut commotions: ResMut<CommotionQueue>,
) {
    let Some(mut fields) = fields else {
        return;
    };
    let presence = clock.dt_days * 24.0;
    for &(_, entity) in &index.sorted {
        let Ok((transform, memberships)) = persons.get(entity) else {
            continue;
        };
        let cell = fields.world_to_cell(society.plane.project(transform.translation()));
        fields.crowd.deposit(0, cell, presence);
        for faction in &memberships.0 {
            if let Some(grid) = fields.factions.get_mut(usize::from(faction.0)) {
                grid.deposit(0, cell, presence);
            }
        }
    }
    for commotion in commotions.0.drain(..) {
        // **A commotion is a positive disturbance.** A negative or non-finite amount is not a commotion
        // at all, and depositing one would read back as absence while still costing the channel a
        // deposit walk — so it is dropped here rather than argued about downstream. An off-grid point
        // needs no check: `deposit` is floor-masked and skips it.
        if !commotion.at.is_finite() || !commotion.amount.is_finite() || commotion.amount <= 0.0 {
            continue;
        }
        let cell = fields.world_to_cell(commotion.at);
        fields.crowd.deposit(1, cell, commotion.amount);
    }
    fields.crowd.evaporate_diffuse(clock.dt_days);
    for grid in &mut fields.factions {
        grid.evaporate_diffuse(clock.dt_days);
    }
    fields.refresh_peaks();
}

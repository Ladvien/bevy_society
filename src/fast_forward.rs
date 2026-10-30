//! **Fast-forward**: run a live society a number of game-days in one call, with no frames, so an
//! editor can preview where people go and how their ties change, then compare or reset.
//!
//! It drives exactly the schedule a running app does (`FixedMain`, one fixed step at a time, with the
//! message buffers rotated between steps), so the result is the society the host would have after
//! the same number of ticks: same seed and same inputs give the same society, bit for bit.

use std::fmt;

use bevy::ecs::change_detection::Tick;
use bevy::ecs::message::MessageRegistry;
use bevy::ecs::world::World;
use bevy::time::{Fixed, run_fixed_main_schedule};

use crate::core::{ClockSource, SocietyClock, SocietyRng, SocietySettings};

/// What a [`fast_forward`] did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FastForward {
    /// Society ticks run.
    pub ticks: u64,
    /// Game time before, in days.
    pub from_days: f64,
    /// Game time after, in days.
    pub to_days: f64,
}

/// Why a [`fast_forward`] did not run.
#[derive(Debug, Clone, PartialEq)]
pub enum FastForwardError {
    /// The world has no [`SocietyClock`] or [`SocietySettings`]: the society plugins are not added.
    NoSociety,
    /// The app has not run its `Startup` schedule yet (no [`SocietyRng`]): update it once first.
    NotStarted,
    /// `days` was negative or not finite.
    BadDays(f32),
    /// The clock follows the host's own time ([`ClockSource::Host`]), which a fast-forward cannot
    /// advance: the host owns that clock.
    HostClock,
    /// The fixed timestep or the society tick is zero or not finite, so time cannot pass.
    NoTimeStep,
}

impl fmt::Display for FastForwardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSociety => write!(f, "no SocietyClock or SocietySettings: add SocietyPlugins"),
            Self::NotStarted => write!(
                f,
                "the society has not started: run one app update before fast-forwarding"
            ),
            Self::BadDays(d) => write!(
                f,
                "cannot fast-forward {d} days: need a finite, non-negative count"
            ),
            Self::HostClock => write!(
                f,
                "the society clock follows the host (ClockSource::Host); a fast-forward cannot advance it"
            ),
            Self::NoTimeStep => write!(
                f,
                "the fixed timestep and SocietySettings::tick_secs must be positive and finite"
            ),
        }
    }
}

impl std::error::Error for FastForwardError {}

/// Runs `days` game-days of society ticks on `world` and returns how many ticks ran.
///
/// The count is `ceil(days * seconds_per_day / tick_secs)`; `0.0` runs none. Input messages already
/// queued (`PlantRumor`, `Interact`, `FireTrigger`) are drained by the first step as in a live app.
/// A host's `Update` systems do not run, and the app's real-time clock does not move.
///
/// # Errors
/// [`FastForwardError`] when the society is not installed, `days` is not a finite non-negative
/// number, `Startup` has not run, the clock is [`ClockSource::Host`], or a time step is zero.
pub fn fast_forward(world: &mut World, days: f32) -> Result<FastForward, FastForwardError> {
    if !days.is_finite() || days < 0.0 {
        return Err(FastForwardError::BadDays(days));
    }
    let (tick_secs, seconds_per_day, clock) = {
        let settings = world
            .get_resource::<SocietySettings>()
            .ok_or(FastForwardError::NoSociety)?;
        (settings.tick_secs, settings.seconds_per_day, settings.clock)
    };
    let start = *world
        .get_resource::<SocietyClock>()
        .ok_or(FastForwardError::NoSociety)?;
    if !world.contains_resource::<SocietyRng>() {
        return Err(FastForwardError::NotStarted);
    }
    if clock == ClockSource::Host {
        return Err(FastForwardError::HostClock);
    }
    let timestep = world.resource::<bevy::time::Time<Fixed>>().timestep();
    if !(tick_secs.is_finite() && tick_secs > 0.0 && !timestep.is_zero()) {
        return Err(FastForwardError::NoTimeStep);
    }

    let wanted =
        (f64::from(days) * f64::from(seconds_per_day) / f64::from(tick_secs)).ceil() as u64;
    let target = start.tick + wanted;
    while world.resource::<SocietyClock>().tick < target {
        world
            .resource_mut::<bevy::time::Time<Fixed>>()
            .accumulate_overstep(timestep);
        run_fixed_main_schedule(world);
        world.resource_scope(
            |world, mut registry: bevy::ecs::change_detection::Mut<MessageRegistry>| {
                registry.run_updates(world, Tick::new(0));
            },
        );
    }

    let end = *world.resource::<SocietyClock>();
    Ok(FastForward {
        ticks: end.tick - start.tick,
        from_days: start.now_days,
        to_days: end.now_days,
    })
}

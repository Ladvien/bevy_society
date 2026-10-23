//! The plugin group that installs the whole society.

use bevy::app::{PluginGroup, PluginGroupBuilder};

use crate::core::SocietyCorePlugin;

/// Every society plugin, in dependency order. Add it to an app with `DefaultPlugins` and the
/// host's own systems; disable a member with `.disable::<…>()` to replace it.
///
/// Plugins that are not yet part of the crate are added here as they land; each one is also usable
/// alone after [`SocietyCorePlugin`].
pub struct SocietyPlugins;

impl PluginGroup for SocietyPlugins {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>()
            .add(SocietyCorePlugin)
            .add(crate::mind::MindPlugin)
            .add(crate::relationships::RelationshipsPlugin)
            .add(crate::beliefs::BeliefsPlugin)
            .add(crate::groups::GroupsPlugin);
        #[cfg(feature = "fields")]
        let group = group.add(crate::fields::SocialFieldPlugin);
        #[cfg(feature = "steering")]
        let group = group.add(crate::steering::SteeringPlugin);
        group
    }
}

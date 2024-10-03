//! this is for the user to override workspace settings

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::project::ProjectSettings;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Reflect)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
/// Settings for the user
pub struct UserSettings {
    /// project settings for the user
    pub project_settings: ProjectSettings,
}

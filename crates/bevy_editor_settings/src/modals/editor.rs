//! Editor settings

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::project::ProjectSettings;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Reflect)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
/// Settings for the editor
pub struct EditorSettings {
    /// current project settings
    pub project_settings: ProjectSettings,
}

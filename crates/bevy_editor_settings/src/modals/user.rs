//! this is for the user to overide workspace settings


use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::project::ProjectSettings;


#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Reflect)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
/// Settings for the user
pub struct UserSettings {
    project_settings: ProjectSettings,
}
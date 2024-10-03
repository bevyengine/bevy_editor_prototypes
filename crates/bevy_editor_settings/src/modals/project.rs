//! Project settings for the editor

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Reflect)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
/// Settings for the editor
#[serde(default)]
pub struct ProjectSettings {
    /// The name of the project
    name: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            name: "My Project".to_string(),
        }
    }
}

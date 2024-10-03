//! Workspace settings

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::project::ProjectSettings;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Reflect)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
/// Settings for the entire workspace
/// This should be in the root of your project
pub struct WorkspaceSettings {
    /// Settings for the editor per workspace
    pub editor_settings: ProjectSettings,
    /// Settings for building the project
    pub build: Build,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Reflect)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
/// Settings for building the project
pub struct Build {
    /// The Command for building the project in debug mode
    debug: String,
    /// The Command for building the project in release mode
    release: String,
    /// The Command for running the project in debug mode
    run_debug: String,
    /// The Command for running the project in release mode
    run_release: String,
}

impl Default for Build {
    fn default() -> Self {
        Self {
            debug: "cargo build".to_string(),
            release: "cargo build --release".to_string(),
            run_debug: "cargo run".to_string(),
            run_release: "cargo run --release".to_string(),
        }
    }
}

use bevy::{prelude::Resource, reflect::Reflect};

use crate::{SettingsTags, SettingsType};



#[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
#[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
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

//! A straightforward way to store and retrieve user preferences on disk for Bevy applications.

use bevy::prelude::*;

mod persistant;
pub mod modals;

/// The directory where preferences are stored. under the user's configuration directory.
pub(crate) const DEFAULT_APP_NAME: &str = "bevy_editor";

/// A Bevy plugin for editor settings.
/// This plugin loads the workspace settings, user settings, and project settings.
pub struct EditorSettingsPlugin;

#[derive(Debug, Clone, PartialEq, Eq, Resource, Reflect)]
/// Represents the settings for the editor.
/// This includes workspace settings, user settings, and project settings.
pub struct Settings {
    /// Settings for the workspace
    workspace_settings: Option<modals::workspace::WorkspaceSettings>,
    /// Settings for the user
    user_settings: Option<modals::user::UserSettings>,
    /// default project settings used when no workspace or user settings are present for a given setting
    project_settings: modals::project::ProjectSettings,
}

impl Settings {}

impl Plugin for EditorSettingsPlugin {
    fn build(&self, app: &mut App) {
        let workspace_settings = persistant::load_workspace_settings()
            .inspect_err(|error| {
                error!("Error loading workspace settings: {:?}", error);
            })
            .ok();
        let user_settings = persistant::load_user_settings()
            .inspect_err(|error| {
                error!("Error loading user settings: {:?}", error);
            })
            .ok();

        let project_settings = modals::project::ProjectSettings::default();

        app.insert_resource(Settings {
            workspace_settings,
            user_settings,
            project_settings,
        });
    }
}

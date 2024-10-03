//! A straightforward way to store and retrieve user preferences on disk for Bevy applications.

use bevy::prelude::*;

mod persistant;
pub mod modals;


/// A Bevy plugin for editor settings.
/// This plugin loads the workspace settings, user settings, and project settings.
pub struct EditorSettingsPlugin;

#[derive(Debug, Clone, PartialEq, Eq, Resource, Reflect)]
/// Represents the settings for the editor.
/// This includes workspace settings, user settings, and project settings.
pub struct Settings {
    /// Settings for the workspace
    pub workspace_settings: Option<modals::workspace::WorkspaceSettings>,
    /// Settings for the user
    pub user_settings: Option<modals::user::UserSettings>,
    /// default project settings used when no workspace or user settings are present for a given setting
    pub project_settings: modals::project::ProjectSettings,
}

impl Settings {
    /// Get the project settings.
    /// 
    /// TODO this needs to do some kind of merging of settings
    /// the order of precedence should be from highest to lowest:
    /// 1. user settings
    /// 2. workspace settings
    /// 3. default project settings
    pub fn project_settings(&self) -> &modals::project::ProjectSettings {
        self.user_settings
            .as_ref()
            .map(|settings| &settings.project_settings)
            .or_else(|| self.workspace_settings.as_ref().map(|settings| &settings.editor_settings))
            .unwrap_or(&self.project_settings)
    }

    /// Save the user settings.
    pub fn save_user_settings(&self) -> Result<(), persistant::PersistantError> {
        if let Some(user_settings) = &self.user_settings {
            persistant::save_user_settings(user_settings)?;
            Ok(())
        } else {
            warn!("No user settings to save.");
            Ok(())
        }
    }
}

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

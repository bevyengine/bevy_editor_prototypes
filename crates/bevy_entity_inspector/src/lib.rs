//! A Bevy plugin for inspecting entities in a Bevy application.
//! This plugin provides a UI for viewing and editing properties of entities in the scene.
//! It includes functionality for remote inspection and editing of entities.
//! It is designed to be used with the Bevy game engine and integrates with the Bevy editor

use bevy::app::Plugin;

use pane::EntityInspectorPanesPlugin;
use remote::EntityInspectorRemotePlugin;

mod pane;
mod remote;

/// The main plugin for the Entity Inspector.
/// This plugin sets up the necessary components and systems for the Entity Inspector to function.
/// It includes the Entity Inspector panes and remote functionality.
/// To use this plugin, add it to your Bevy app using `add_plugins(EntityInspectorPlugin)`.
/// This will register the Entity Inspector panes and remote functionality with the Bevy app.
pub struct EntityInspectorPlugin;

impl Plugin for EntityInspectorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(bevy::DefaultPlugins)
            .add_plugins(EntityInspectorPanesPlugin)
            .add_plugins(EntityInspectorRemotePlugin::default());
    }
}

//! A Bevy plugin for inspecting entities in a Bevy application.
//! This plugin provides a UI for viewing and editing properties of entities in the scene.
//! It includes functionality for remote inspection and editing of entities.
//! It is designed to be used with the Bevy game engine and integrates with the Bevy editor
//!
//! # Example Usage
//!
//! To stand up the dummy application that you can connect to with the Bevy Remote Protocol (BRP) and edit,
//! you can use the following command to run the example:
//! ```bash
//! cargo run --example cube_server
//! ```
//!
//! To use the Entity Inspector in your Bevy application, add the `EntityInspectorPlugin` to your app:
//! ```bash
//! cargo run --example inspector
//! ```

use bevy::{
    app::Plugin,
    ecs::{component::ComponentInfo, entity::Entity, resource::Resource},
    platform::collections::HashMap,
    reflect::{PartialReflect, Reflect},
};

use bevy_context_menu::ContextMenuPlugin;
use bevy_editor::ui::EditorUIPlugin;
use bevy_editor_core::EditorCorePlugin;
use bevy_editor_styles::StylesPlugin;
use bevy_pane_layout::PaneLayoutPlugin;
use remote::EntityInspectorRemotePlugin;
use ui::EntityInspectorUiPlugin;

mod remote;
mod ui;

/// The main plugin for the Entity Inspector.
/// This plugin sets up the necessary components and systems for the Entity Inspector to function.
/// It includes the Entity Inspector panes and remote functionality.
/// To use this plugin, add it to your Bevy app using `add_plugins(EntityInspectorPlugin)`.
/// This will register the Entity Inspector panes and remote functionality with the Bevy app.
pub struct EntityInspectorPlugin;

impl Plugin for EntityInspectorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<EntityInspectorRows>()
            .init_resource::<EntityInspectorRow>()
            .add_plugins(EditorCorePlugin)
            .add_plugins(ContextMenuPlugin)
            .add_plugins(StylesPlugin)
            // .add_plugins(PaneLayoutPlugin)
            .add_plugins(EditorUIPlugin)
            .add_plugins(EntityInspectorUiPlugin)
            .add_plugins(EntityInspectorRemotePlugin::default());
    }
}

#[derive(Resource, Default, Debug)]
pub struct EntityInspectorRow {
    pub name: String,
    pub components: HashMap<String, Box<dyn Reflect>>,
}

#[derive(Resource, Default, Debug)]
pub struct EntityInspectorRows {
    pub rows: HashMap<Entity, EntityInspectorRow>,
}

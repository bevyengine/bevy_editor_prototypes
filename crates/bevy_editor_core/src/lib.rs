//! This crate provides core functionality for the Bevy Engine Editor.

use bevy::prelude::*;

/// Plugin for the editor scene tree pane.
pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedEntity>()
            .register_type::<SelectedEntity>();
    }
}

/// The currently selected entity in the scene.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

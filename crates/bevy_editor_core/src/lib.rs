//! This crate provides core functionality for the Bevy Engine Editor.

use bevy::{ecs::entity::Entities, prelude::*};

/// Plugin for the editor scene tree pane.
pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedEntity>()
            .register_type::<SelectedEntity>()
            .add_systems(PostUpdate, reset_selected_entity_if_entity_despawned);
    }
}

/// The currently selected entity in the scene.
#[derive(Component, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

/// System to reset [`SelectedEntity`] when the entity is despawned.
pub fn reset_selected_entity_if_entity_despawned(
    mut selected_entity: ResMut<SelectedEntity>,
    entities: &Entities,
) {
    if let Some(e) = selected_entity.0 {
        if !entities.contains(e) {
            selected_entity.0 = None;
        }
    }
}

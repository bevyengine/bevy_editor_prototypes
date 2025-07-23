//! Editor selection module.

use bevy::{ecs::entity::Entities, prelude::*};

/// Editor selection plugin.
#[derive(Default)]
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedEntity>()
            .register_type::<SelectedEntity>()
            .add_systems(PostUpdate, reset_selected_entity_if_entity_despawned);
    }
}

/// The currently selected entity in the scene.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

impl SelectedEntity {
    /// Toggle selection for an entity.
    pub fn toggle(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        if self.0 == Some(entity) {
            self.0 = None;
        } else {
            self.0 = Some(entity);
        }
    }

    /// Set an entity as selected.
    pub fn set(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        self.0 = Some(entity);
    }

    /// Empty the selection.
    pub fn reset(&mut self) {
        self.0 = None;
    }

    /// Check whether the selection contains a given entity.
    pub fn contains(&self, entity: Entity) -> bool {
        self.0.is_some_and(|selected| selected == entity)
    }
}

/// System to reset [`SelectedEntity`] when the entity is despawned.
pub fn reset_selected_entity_if_entity_despawned(
    mut selected_entity: ResMut<SelectedEntity>,
    entities: &Entities,
) {
    if let Some(e) = selected_entity.0 {
        if !entities.contains(e) {
            selected_entity.reset();
        }
    }
}

/// Common handler observer systems for entity selection behavior.
pub mod common_handlers {
    use crate::utils::DragCancelClick;

    use super::*;

    /// Toggles selection for this entity when it is clicked.
    pub fn toggle_select_on_click(
        mut trigger: On<Pointer<DragCancelClick>>,
        mut selected_entity: ResMut<SelectedEntity>,
    ) {
        if trigger.button == PointerButton::Primary {
            selected_entity.toggle(trigger.target());
            trigger.propagate(false);
        }
    }

    /// Toggles selection for an entity when this entity is clicked.
    pub fn toggle_select_on_click_for_entity(
        entity: Entity,
    ) -> impl FnMut(On<Pointer<DragCancelClick>>, ResMut<SelectedEntity>) {
        move |mut trigger: On<Pointer<DragCancelClick>>,
              mut selected_entity: ResMut<SelectedEntity>| {
            if trigger.button == PointerButton::Primary {
                selected_entity.toggle(entity);
                trigger.propagate(false);
            }
        }
    }
}

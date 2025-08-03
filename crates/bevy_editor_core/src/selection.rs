//! Editor selection module.

use bevy::{
    ecs::entity::{Entities, EntitySetIterator, UniqueEntityVec},
    prelude::*,
};

use crate::utils::DragCancelClick;

/// Editor selection plugin.
#[derive(Default)]
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSelection>()
            .add_systems(PostUpdate, remove_entity_from_selection_if_despawned)
            .add_observer(selection_handler);
    }
}

fn selection_handler(
    mut trigger: On<Pointer<DragCancelClick>>,
    selectable_query: Query<(), With<Selectable>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<EditorSelection>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let target = trigger.target();
    if selectable_query.contains(target) {
        trigger.propagate(false);
        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        if shift {
            selection.toggle(target);
        } else {
            selection.set(target);
        }
    }
}

/// The currently selected entities in the scene.
#[derive(Resource, Default)]
pub struct EditorSelection(UniqueEntityVec);

impl EditorSelection {
    /// Toggle selection for an entity.
    pub fn toggle(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        if !self.remove(entity) {
            // SAFETY: The preceding call to self.remove ensures the entity is not present.
            #[expect(unsafe_code)]
            unsafe {
                self.0.push(entity);
            }
        }
    }

    /// Set the selection to an entity, making it the primary selection.
    pub fn set(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        self.0 = std::iter::once(entity).collect();
    }

    /// Add an entity to the selection, making it the primary selection.
    ///
    /// If the entity was already part of the selection it will be made the primary selection.
    pub fn add(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        self.remove(entity);
        // SAFETY: The preceding call to self.remove ensures the entity is not present.
        #[expect(unsafe_code)]
        unsafe {
            self.0.push(entity);
        }
    }

    /// Remove an entity from the selection if present. Returns `true` if the entity was removed.
    pub fn remove(&mut self, entity: Entity) -> bool {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        let mut was_removed = false;
        self.0.retain(|selected| {
            if *selected == entity {
                was_removed = true;
                false
            } else {
                true
            }
        });
        was_removed
    }

    /// Empty the selection.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Check whether the selection contains a given entity.
    pub fn contains(&self, entity: Entity) -> bool {
        self.0.contains(&entity)
    }

    /// The last entity in the selection.
    pub fn primary(&self) -> Option<Entity> {
        self.0.last().copied()
    }

    /// Returns an iterator over all entities in the selection in the order they were selected.
    pub fn iter(&self) -> impl EntitySetIterator<Item = Entity> {
        self.0.iter().copied()
    }
}

/// Marker component for selectable entities.
#[derive(Component, Default, Clone)]
pub struct Selectable;

/// This system removes entities from the [`EditorSelection`] when they are despawned.
pub fn remove_entity_from_selection_if_despawned(
    mut selection: ResMut<EditorSelection>,
    entities: &Entities,
) {
    // Avoid triggering change detection every frame.
    if selection.0.iter().any(|entity| !entities.contains(*entity)) {
        selection.0.retain(|entity| entities.contains(*entity));
    }
}

/// Common [`EditorSelection`] conditions.
pub mod common_conditions {
    use bevy::prelude::*;

    use crate::prelude::EditorSelection;

    /// True if the primary [`EditorSelection`] changed.
    pub fn primary_selection_changed(
        mut cache: Local<Option<Entity>>,
        selection: Res<EditorSelection>,
    ) -> bool {
        let changed = *cache != selection.primary();
        *cache = selection.primary();
        changed
    }
}

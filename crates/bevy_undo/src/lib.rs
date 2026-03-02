//! Undo/Redo functionality for Bevy applications
//!
//! This crate provides a flexible, memory-efficient, and easy-to-use undo/redo system for Bevy applications.
//! It allows you to track changes to entities and components, and revert or reapply those
//! changes as needed.
//!
//! # Features
//!
//! - Automatic undo/redo for components
//! - Support for custom undo/redo commands
//! - Integration with Bevy's entity and component system
//! - Efficient change tracking and storage
//! - Memory-efficient design using Arc for change storage
//!
//! # Usage
//!
//! To use this crate in your Bevy application:
//!
//! 1. Add the `UndoPlugin` to your app
//! 2. Use the `auto_undo` or `auto_reflected_undo` methods to enable automatic undo for specific components
//! 3. Mark entities that should support undo/redo with the `UndoMarker` component
//! 4. Use `UndoRedo` events to trigger undo and redo operations
//!
//! # Example
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_undo::*;
//! use std::sync::Arc;
//! use bevy::platform::collections::HashMap;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(UndoPlugin)
//!         .auto_reflected_undo::<Transform>()
//!         .add_systems(Update, (handle_input, apply_transform));
//!         //.run();
//! }
//!
//! fn handle_input(
//!     keys: Res<ButtonInput<KeyCode>>,
//!     mut undo_redo: MessageWriter<UndoRedo>,
//! ) {
//!     if keys.just_pressed(KeyCode::KeyZ) {
//!         undo_redo.write(UndoRedo::Undo);
//!     }
//!     if keys.just_pressed(KeyCode::KeyY) {
//!         undo_redo.write(UndoRedo::Redo);
//!     }
//! }
//!
//! fn apply_transform(
//!     mut query: Query<(Entity, &mut Transform), With<UndoMarker>>,
//!     mut new_changes: MessageWriter<NewChange>,
//! ) {
//!     for (entity, mut transform) in query.iter_mut() {
//!         // Apply some change
//!         let old_transform = transform.clone();
//!         transform.translation.x += 1.0;
//!
//!         // Register custom change
//!         new_changes.write(NewChange::new(CustomTransformChange {
//!             entity: entity,
//!             old_transform,
//!             new_transform: transform.clone(),
//!         }));
//!     }
//! }
//!
//! #[derive(Clone)]
//! struct CustomTransformChange {
//!     entity: Entity,
//!     old_transform: Transform,
//!     new_transform: Transform,
//! }
//!
//! impl EditorChange for CustomTransformChange {
//!     fn revert(&self, world: &mut World, entity_remap: &HashMap<Entity, Entity>) -> Result<ChangeResult, String> {
//!         // Implementation details...
//!         Err("Not implemented".to_string())
//!     }
//!
//!     fn debug_text(&self) -> String {
//!         format!("Custom transform change for entity {:?}", self.entity)
//!     }
//!
//!     fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
//!         Arc::new(CustomTransformChange {
//!             entity: self.entity,
//!             old_transform: self.new_transform.clone(),
//!             new_transform: self.old_transform.clone(),
//!         })
//!     }
//! }
//! ```
//!
//! This example demonstrates:
//! - Setting up automatic undo/redo for the `Transform` component
//! - Handling input to trigger undo and redo operations
//! - Registering custom changes for more complex operations
//!
//! # Memory Efficiency
//!
//! # Memory Efficiency
//!
//! This crate uses a design that aims for efficiency:
//!
//! - Changes are stored individually, allowing for fine-grained control over the undo/redo history.
//! - Only the differences between states are stored, not entire world snapshots.
//! - The undo history has a configurable maximum size to prevent unbounded memory growth.
//! - For complex operations involving multiple changes, the `ManyChanges` struct is used to group
//!   related changes together, reducing overhead.
//!
//! The overall
//! design still promotes efficient memory usage by storing only necessary information for
//! each change and allowing for batching of related changes.
//!
//! # Custom Undo/Redo Commands
//!
//! For complex undo/redo operations, you can implement the `EditorChange` trait
//! and use the `NewChange` event to register custom changes. This allows you to
//! define precisely how changes should be applied and reverted for any type of operation.
//!
//! # Limitations
//!
//! - The undo system may not capture all types of changes automatically. Complex operations
//!   might require custom `EditorChange` implementations.
//! - Performance impact should be considered when enabling undo/redo for frequently changing components.
//!
//! For more advanced usage and API details, refer to the documentation of individual
//! types and traits in this crate.

// Remove after update to newer rust version
#![allow(clippy::type_complexity)]
use std::sync::Arc;

use bevy::{platform::collections::HashMap, prelude::*};

const MAX_REFLECT_RECURSION: i32 = 10;
const AUTO_UNDO_LATENCY: i32 = 2;

/// Plugin for implementing undo/redo functionality
#[derive(Default)]
pub struct UndoPlugin;

/// Components with this marker will be used for undo
#[derive(Component)]
pub struct UndoMarker;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeChain>();
        app.init_resource::<UndoIgnoreStorage>();
        app.init_resource::<ChangeChainSettings>();

        app.add_message::<NewChange>();
        app.add_message::<UndoRedo>();

        app.configure_sets(
            PostUpdate,
            (UndoSet::PerType, UndoSet::UpdateAll, UndoSet::Remapping)
                .chain()
                .in_set(UndoSet::Global),
        );

        app.add_systems(
            PostUpdate,
            (
                clear_one_frame_ignore,
                update_change_chain,
                undo_redo_logic,
                undo_ignore_tick,
            )
                .chain()
                .in_set(UndoSet::UpdateAll),
        );
    }
}

/// Allows to make `UndoMarker` attached to another marker M so that
/// if there is an entity with marker M, then `UndoMarker` will be added to that entity,
/// and likewise, if there is an entity with `UndoMarker` but without marker M, then `UndoMarker` will be removed
#[derive(Default)]
pub struct SyncUndoMarkersPlugin<M: Component> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Component> Plugin for SyncUndoMarkersPlugin<M> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_system::<M>);
    }
}

fn sync_system<M: Component>(
    mut commands: Commands,
    add_undo: Query<Entity, (With<M>, Without<UndoMarker>)>,
    remove_undo: Query<Entity, (Without<M>, With<UndoMarker>)>,
) {
    for e in add_undo.iter() {
        commands.entity(e).insert(UndoMarker);
    }

    for e in remove_undo.iter() {
        commands.entity(e).remove::<UndoMarker>();
    }
}

/// Defines the set of systems related to undo/redo functionality.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub enum UndoSet {
    /// Systems that handle per-component undo/redo operations.
    PerType,

    /// Systems that manage the global undo/redo chain and logic.
    UpdateAll,

    /// Systems responsible for remapping entities after undo/redo operations.
    Remapping,

    /// A set containing all undo-related systems.
    Global,
}

/// An event that is sent when an undo or redo operation is applied to a component of type T.
///
/// This event is useful for systems that need to react to undo/redo operations
/// on specific component types, such as updating derived data or triggering side effects.
///
/// # Type Parameters
///
/// * `T`: The component type that was affected by the undo/redo operation.
///
/// # Fields
///
/// * `entity`: The entity that owns the component that was modified.
/// * `_phantom`: A zero-sized type used to carry the generic type information.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_undo::*;
/// fn react_to_transform_undo_redo(mut events: MessageReader<UndoRedoApplied<Transform>>) {
///     for event in events.read() {
///         println!("Transform of entity {:?} was modified by undo/redo", event.entity);
///         // Perform any necessary updates or side effects
///     }
/// }
/// ```
#[derive(Event, Message)]
pub struct UndoRedoApplied<T> {
    /// The entity that was modified.
    pub entity: Entity,
    _phantom: std::marker::PhantomData<T>,
}

/// A component that marks an entity to be ignored by the undo system for a short period.
///
/// This component is typically added to entities that have just been modified by an undo/redo
/// operation to prevent these changes from being immediately recorded as new changes.
///
/// # Fields
///
/// * `counter`: The number of frames remaining before the entity is no longer ignored.
///   This value decreases by 1 each frame until it reaches 0, at which point the component is removed.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_undo::*;
///
/// fn apply_undo(mut commands: Commands, entity: Entity) {
///     // Apply undo changes
///     // ...
///
///     // Mark the entity to be ignored by the undo system for the next 10 frames
///     commands.entity(entity).insert(OneFrameUndoIgnore::default());
/// }
/// ```
#[derive(Component)]
pub struct OneFrameUndoIgnore {
    /// How many frames entity still must be ignored
    /// Will reduce by 1 each frame until it reaches 0, at which point the `OneFrameUndoIgnore` will be removed.
    pub counter: i32,
}

impl Default for OneFrameUndoIgnore {
    fn default() -> Self {
        Self { counter: 10 }
    }
}

fn update_change_chain(
    mut buffer: Local<Vec<NewChange>>, //Buffer will use for chain reaction changes and collecting them together
    settings: Res<ChangeChainSettings>,
    mut change_chain: ResMut<ChangeChain>,
    mut events: MessageReader<NewChange>,
) {
    //collect buffer
    let mut events_on_current_frame = 0;
    for event in events.read() {
        buffer.push(event.clone());
        events_on_current_frame += 1;
    }

    if events_on_current_frame > 0 {
        return;
    }

    if buffer.is_empty() {
        return;
    }

    //Drop buffer to vec of arc
    let mut new_changes = vec![];
    for b in buffer.iter() {
        new_changes.push(b.change.clone());
        change_chain.changes_for_redo.clear();
    }

    //Clear buffer
    buffer.clear();

    match new_changes.len().cmp(&1) {
        std::cmp::Ordering::Less => {}
        std::cmp::Ordering::Equal => {
            change_chain.changes.push(new_changes[0].clone());
        }
        std::cmp::Ordering::Greater => {
            change_chain.changes.push(Arc::new(ManyChanges {
                changes: new_changes,
            }));
        }
    };

    if change_chain.changes.len() > settings.max_change_chain_size {
        let count = change_chain.changes.len() - settings.max_change_chain_size;
        change_chain.changes.drain(0..count);
    }
}

fn clear_one_frame_ignore(
    mut commands: Commands,
    mut query: Query<(Entity, &mut OneFrameUndoIgnore)>,
) {
    for (e, mut ignore) in query.iter_mut() {
        ignore.counter -= 1;
        if ignore.counter <= 0 {
            commands.entity(e).remove::<OneFrameUndoIgnore>();
        }
    }
}

fn undo_redo_logic(world: &mut World) {
    world.resource_scope::<Messages<UndoRedo>, _>(|world, mut events| {
        world.resource_scope::<ChangeChain, _>(|world, mut change_chain| {
            {
                let mut reader = events.get_cursor();
                for event in reader.read(&events) {
                    match event {
                        UndoRedo::Undo => {
                            if let Some(change) = change_chain.changes.pop() {
                                let res = change.revert(world, &change_chain.entity_remap).unwrap();
                                if let ChangeResult::SuccessWithRemap(remap) = res {
                                    change_chain.entity_remap.extend(remap);
                                }
                                change_chain.changes_for_redo.push(change);
                            }
                        }
                        UndoRedo::Redo => {
                            if let Some(change) = change_chain.changes_for_redo.pop() {
                                let inverse_change = change.get_inverse();
                                let res = inverse_change
                                    .revert(world, &change_chain.entity_remap)
                                    .unwrap();
                                if let ChangeResult::SuccessWithRemap(remap) = res {
                                    change_chain.entity_remap.extend(remap);
                                }
                                change_chain.changes.push(change);
                            }
                        }
                    }
                }
            }
            events.clear();
        });
    });
}

/// Resource for storing the chain of changes
/// All undo/redo moving of world state will be in this change
#[derive(Resource, Default)]
pub struct ChangeChain {
    /// Changes that were applied to world and registered in this `change_chain`
    pub changes: Vec<Arc<dyn EditorChange + Send + Sync>>,
    /// Changes for redo
    pub changes_for_redo: Vec<Arc<dyn EditorChange + Send + Sync>>,
    /// We need to store entity remapping if any of the entities changed their id by
    /// destroying/spawning, and to handle entity links in component fields.
    entity_remap: HashMap<Entity, Entity>,
}

/// Settings for `ChangeChain` resource
#[derive(Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct ChangeChainSettings {
    /// Maximum number of changes in the change chain that can be stored
    pub max_change_chain_size: usize,
}

impl Default for ChangeChainSettings {
    fn default() -> Self {
        Self {
            max_change_chain_size: 200,
        }
    }
}

impl ChangeChain {
    /// Undo last registered change
    pub fn undo(&mut self, world: &mut World) {
        if let Some(change) = self.changes.pop() {
            let res = change.revert(world, &self.entity_remap).unwrap();
            self.changes_for_redo.push(change);
            self.update_remap(res);
        }
    }

    /// Redo last undone change
    pub fn redo(&mut self, world: &mut World) {
        if let Some(change) = self.changes_for_redo.pop() {
            let inverse_change = change.get_inverse();
            let res = inverse_change.revert(world, &self.entity_remap).unwrap();
            self.changes.push(change);
            self.update_remap(res);
        }
    }

    /// Update destroyed-entity->new-entity mapping for handling entities links after undo / redo
    fn update_remap(&mut self, result: ChangeResult) {
        match result {
            ChangeResult::Success => {}
            ChangeResult::SuccessWithRemap(new_remap) => {
                for (prev, new) in new_remap {
                    self.entity_remap.insert(prev, new);
                }
            }
        }
    }
}

/// Returns the entity with the given Entity. If the entity was remapped, the remapped entity is returned.
pub fn get_entity_with_remap(entity: Entity, entity_remap: &HashMap<Entity, Entity>) -> Entity {
    *entity_remap.get(&entity).unwrap_or(&entity)
}

/// Change, which can be stored in the change chain
pub trait EditorChange {
    /// Revert all changes applied to the world by this change
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String>;

    /// Returns a human-readable text describing the change
    fn debug_text(&self) -> String;

    /// Returns the inverse of this change.
    /// For example:
    /// for `spawn()` -> `despawn()`
    /// for `despawn()` -> `spawn()`
    /// for insert component -> remove component
    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync>;
}

/// Represents the result of applying or reverting a change in the undo/redo system.
pub enum ChangeResult {
    /// The change was applied or reverted successfully without any entity remapping.
    Success,

    /// The change was applied or reverted successfully, but some entities were remapped.
    /// Contains a vector of (`old_entity`, `new_entity`) pairs representing the remapping.
    SuccessWithRemap(Vec<(Entity, Entity)>),
}
/// Represents an undo or redo operation to be performed on the change chain.
#[derive(Event, Message)]
pub enum UndoRedo {
    /// Requests to undo the last change in the change chain.
    Undo,

    /// Requests to redo the last undone change in the change chain.
    Redo,
}

/// Represents a new change to be added to the change chain.
#[derive(Event, Message, Clone)]
pub struct NewChange {
    /// The change to be added to the change chain, wrapped in an Arc for shared ownership.
    pub change: Arc<dyn EditorChange + Send + Sync>,
}

impl NewChange {
    /// Creates a new [`NewChange`] with the given [`EditorChange`].
    pub fn new<T: EditorChange + Send + Sync + 'static>(change: T) -> NewChange {
        NewChange {
            change: Arc::new(change),
        }
    }
}

/// Represents an change for adding an entity to the world.
///
/// This struct is used to revert the spawning of an entity by storing its ID,
/// allowing the undo system to remove it when necessary.
pub struct AddedEntity {
    /// The ID of the entity that was added to the world.
    pub entity: Entity,
}

impl EditorChange for AddedEntity {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        world.entity_mut(e).despawn();
        world
            .resource_mut::<UndoIgnoreStorage>()
            .storage
            .insert(e, OneFrameUndoIgnore::default());
        info!("Removed Entity: {}", e.index());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("Added Entity: {}", self.entity.index())
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(RemovedEntity {
            entity: self.entity,
        })
    }
}

/// Represents an change for removing an entity from the world.
///
/// This struct is used to revert the removal of an entity by storing its ID,
/// allowing the undo system to respawn entity when necessary.
pub struct RemovedEntity {
    /// The ID of the entity that was removed from the world.
    pub entity: Entity,
}

impl EditorChange for RemovedEntity {
    fn revert(
        &self,
        world: &mut World,
        remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        if let Some(e) = remap.get(&self.entity) {
            if world.get_entity(*e).is_ok() {
                let id = world
                    .spawn_empty()
                    .insert(OneFrameUndoIgnore::default())
                    .id();
                info!("Reverted Removed Entity: {}", e.index());
                Ok(ChangeResult::SuccessWithRemap(vec![(self.entity, id)]))
            } else {
                info!("Reverted Removed Entity: {}", e.index());
                Ok(ChangeResult::Success)
            }
        } else {
            let id = world
                .spawn_empty()
                .insert(OneFrameUndoIgnore::default())
                .id();
            info!("Reverted Removed Entity: {}", self.entity.index());
            Ok(ChangeResult::SuccessWithRemap(vec![(self.entity, id)]))
        }
    }

    fn debug_text(&self) -> String {
        format!("Removed Entity: {}", self.entity.index())
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(AddedEntity {
            entity: self.entity,
        })
    }
}

/// Represents an changing a component in an entity.
///
/// This struct stores both the old and new values of a component, as well as
/// the entity ID, allowing the undo system to revert changes to components.
///
/// # Type Parameters
///
/// * `T`: The type of the component that was changed. Must implement the `Component` trait.
pub struct ComponentChange<T: Component> {
    /// The previous value of the component before the change.
    old_value: T,
    /// The new value of the component after the change.
    new_value: T,
    /// The ID of the entity whose component was changed.
    entity: Entity,
}

impl<T: Component + Clone> EditorChange for ComponentChange<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);

        world
            .entity_mut(e)
            .insert(self.old_value.clone())
            .insert(OneFrameUndoIgnore::default());
        info!("Reverted ComponentChange for entity: {}", e.index());
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("ComponentChange for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(ComponentChange {
            old_value: self.new_value.clone(),
            new_value: self.old_value.clone(),
            entity: self.entity,
        })
    }
}

/// Represents a change in a component that supports reflection.
///
/// This struct is used to track changes to components that implement `Component`, `Reflect`, and `FromReflect` traits.
/// It stores both the old and new values of the component, as well as the entity ID, allowing the undo system
/// to revert changes to reflected components.
///
/// # Type Parameters
///
/// * `T`: The type of the component that was changed. Must implement `Component`, `Reflect`, and `FromReflect` traits.
///
/// # Fields
///
/// * `old_value`: The previous value of the component before the change.
/// * `new_value`: The new value of the component after the change.
/// * `entity`: The ID of the entity whose component was changed.
///
/// # Usage
///
/// This struct is primarily used internally by the undo system to track and revert changes to reflected components.
/// It's particularly useful for components that haven't clone and partial equal triats.
/// Note: This struct is part of the internal undo system and is typically not used directly in user code.
pub struct ReflectedComponentChange<T: Component + Reflect + FromReflect> {
    /// The old value of the component
    old_value: T,
    /// The new value of the component
    new_value: T,
    /// The ID of the entity whose component was changed
    entity: Entity,
}

impl<T: Component + Reflect + FromReflect> EditorChange for ReflectedComponentChange<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);

        world
            .entity_mut(e)
            .insert(<T as FromReflect>::from_reflect(&self.old_value).unwrap())
            .insert(OneFrameUndoIgnore::default());
        world.write_message(UndoRedoApplied::<T> {
            entity: e,
            _phantom: std::marker::PhantomData,
        });

        info!(
            "Reverted ReflectedComponentChange for entity: {}",
            e.index()
        );
        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!(
            "{:?} changed for entity {:?}",
            pretty_type_name::pretty_type_name::<T>(),
            self.entity
        )
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(ReflectedComponentChange {
            old_value: <T as FromReflect>::from_reflect(&self.new_value).unwrap(),
            new_value: <T as FromReflect>::from_reflect(&self.old_value).unwrap(),
            entity: self.entity,
        })
    }
}

/// Represents a change for adding a component to an entity.
///
/// This struct is used to track the addition of a component to an entity,
/// allowing the undo system to remove it when necessary.
///
/// # Type Parameters
///
/// * `T`: The type of the component that was added. Must implement the `Component` trait.
pub struct AddedComponent<T: Component> {
    /// The value of the component that was added.
    new_value: T,
    /// The ID of the entity to which the component was added.
    entity: Entity,
}

impl<T: Component + Clone> EditorChange for AddedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let e = get_entity_with_remap(self.entity, entity_remap);
        let mut add_to_ignore = false;
        if let Ok(mut e) = world.get_entity_mut(e) {
            e.remove::<T>().insert(OneFrameUndoIgnore::default());
            add_to_ignore = true;
        }
        if add_to_ignore {
            world
                .resource_mut::<UndoIgnoreStorage>()
                .storage
                .insert(e, OneFrameUndoIgnore::default());
        }

        info!("Reverted AddedComponent for entity: {}", e.index());

        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("AddedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(RemovedComponent {
            entity: self.entity,
            old_value: self.new_value.clone(),
        })
    }
}

/// Represents a change for adding a reflected component to an entity.
///
/// This struct is used to track the addition of a component that supports reflection
/// to an entity, allowing the undo system to remove it when necessary.
///
/// # Type Parameters
///
/// * `T`: The type of the component that was added. Must implement `Component`, `Reflect`,
///   and `FromReflect` traits.
pub struct ReflectedAddedComponent<T: Component + Reflect + FromReflect> {
    /// The value of the component that was added.
    new_value: T,
    /// The ID of the entity to which the component was added.
    entity: Entity,
}

impl<T: Component + Reflect + FromReflect> EditorChange for ReflectedAddedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let dst = entity_remap
            .get(&self.entity)
            .map_or(self.entity, |remapped| *remapped);
        if let Ok(mut e) = world.get_entity_mut(dst) {
            e.remove::<T>().insert(OneFrameUndoIgnore::default());
        }
        world
            .resource_mut::<UndoIgnoreStorage>()
            .storage
            .insert(dst, OneFrameUndoIgnore::default());
        world.write_message(UndoRedoApplied::<T> {
            entity: dst,
            _phantom: std::marker::PhantomData,
        });

        info!(
            "Reverted ReflectedAddedComponent for entity: {}",
            dst.index()
        );

        Ok(ChangeResult::Success)
    }

    fn debug_text(&self) -> String {
        format!("ReflectedAddedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(ReflectedRemovedComponent {
            old_value: <T as FromReflect>::from_reflect(&self.new_value).unwrap(),
            entity: self.entity,
        })
    }
}

/// Represents a change for removing a component from an entity.
///
/// This struct is used to track the removal of a component from an entity,
/// allowing the undo system to re-add it when necessary.
///
/// # Type Parameters
///
/// * `T`: The type of the component that was removed. Must implement the `Component` and `Clone` traits.
pub struct RemovedComponent<T: Component + Clone> {
    /// The value of the component that was removed.
    old_value: T,
    /// The ID of the entity from which the component was removed.
    entity: Entity,
}

impl<T: Component + Clone> EditorChange for RemovedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = vec![];
        let dst = entity_remap.get(&self.entity).map_or_else(
            || {
                if world.get_entity(self.entity).is_ok() {
                    self.entity
                } else {
                    let id = world.spawn_empty().id();
                    remap.push((self.entity, id));
                    id
                }
            },
            |remapped| *remapped,
        );

        world
            .entity_mut(dst)
            .insert(self.old_value.clone())
            .insert(OneFrameUndoIgnore::default());

        info!("Reverted RemovedComponent for entity: {}", dst.index());

        Ok(ChangeResult::SuccessWithRemap(remap))
    }

    fn debug_text(&self) -> String {
        format!("RemovedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(AddedComponent {
            new_value: self.old_value.clone(),
            entity: self.entity,
        })
    }
}

/// Represents a change for removing a reflected component from an entity.
///
/// This struct is used to track the removal of a component that supports reflection
/// from an entity, allowing the undo system to re-add it when necessary.
///
/// # Type Parameters
///
/// * `T`: The type of the component that was removed. Must implement `Component` and `Reflect` traits.
pub struct ReflectedRemovedComponent<T: Component + Reflect> {
    /// The value of the component that was removed.
    old_value: T,
    /// The ID of the entity from which the component was removed.
    entity: Entity,
}

impl<T: Component + Reflect + FromReflect> EditorChange for ReflectedRemovedComponent<T> {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = vec![];
        let dst = entity_remap.get(&self.entity).map_or_else(
            || {
                if world.get_entity(self.entity).is_ok() {
                    self.entity
                } else {
                    let id = world.spawn_empty().id();
                    remap.push((self.entity, id));
                    id
                }
            },
            |remapped| *remapped,
        );

        world
            .entity_mut(dst)
            .insert(<T as FromReflect>::from_reflect(&self.old_value).unwrap())
            .insert(OneFrameUndoIgnore::default());
        world.write_message(UndoRedoApplied::<T> {
            entity: dst,
            _phantom: std::marker::PhantomData,
        });

        info!(
            "Reverted ReflectedRemovedComponent for entity: {}",
            dst.index()
        );

        Ok(ChangeResult::SuccessWithRemap(remap))
    }

    fn debug_text(&self) -> String {
        format!("ReflectedRemovedComponent for entity {:?}", self.entity)
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        Arc::new(ReflectedAddedComponent {
            new_value: <T as FromReflect>::from_reflect(&self.old_value).unwrap(),
            entity: self.entity,
        })
    }
}

/// Represents a collection of multiple changes that occurred simultaneously and should be applied or reverted together.
///
/// `ManyChanges` is automatically generated by the undo system to group multiple `EditorChange`
/// instances that occur within the same frame or update cycle. This allows complex or multi-part
/// operations to be treated as a single, atomic change for undo/redo purposes.
///
/// # Implementation
///
/// `ManyChanges` implements the `EditorChange` trait, allowing it to be treated as a single change
/// in the undo/redo system. When reverted, it applies all contained changes in reverse order
/// to ensure proper undo behavior.
pub struct ManyChanges {
    changes: Vec<Arc<dyn EditorChange + Send + Sync>>,
}

impl EditorChange for ManyChanges {
    fn revert(
        &self,
        world: &mut World,
        entity_remap: &HashMap<Entity, Entity>,
    ) -> Result<ChangeResult, String> {
        let mut remap = entity_remap.clone();
        for change in self.changes.iter() {
            let res = change.revert(world, &remap)?;
            match res {
                ChangeResult::Success => {}
                ChangeResult::SuccessWithRemap(new_remap) => {
                    remap.extend(new_remap);
                }
            }
        }

        info!("Reverted ManyChanges");

        Ok(ChangeResult::SuccessWithRemap(
            remap.iter().map(|(key, value)| (*key, *value)).collect(),
        ))
    }

    fn debug_text(&self) -> String {
        "ManyChanges".to_string()
    }

    fn get_inverse(&self) -> Arc<dyn EditorChange + Send + Sync> {
        let mut old_changes = self.changes.clone();
        old_changes.reverse();
        let new_changes = old_changes
            .iter()
            .map(|change| change.get_inverse())
            .collect::<Vec<_>>();

        Arc::new(ManyChanges {
            changes: new_changes,
        })
    }
}

/// A component that marks an entity as having a changed component of type `T`.
///
/// This marker is used internally by the undo system to track changes to components
/// and manage the timing of when these changes should be recorded for undo operations.
///
/// # Type Parameters
///
/// * `T`: The type of the component that has changed.
///
/// # Fields
///
/// * `latency`: An integer representing the number of frames to wait before recording the change.
///   This helps to batch rapid changes and avoid creating unnecessary undo entries.
///
/// # Usage
///
/// This component is automatically added and removed by the undo system and should not
/// be manipulated directly by users.
#[derive(Component)]
pub struct ChangedMarker<T> {
    latency: i32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for ChangedMarker<T> {
    fn default() -> Self {
        Self {
            latency: AUTO_UNDO_LATENCY, //2 frame latency
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A resource that stores entities that should be ignored by the undo system for a short period.
///
/// This storage is used to prevent newly undone or redone changes from immediately
/// triggering new undo entries, which could lead to infinite loops or unexpected behavior.
///
/// # Fields
///
/// * `storage`: A `HashMap` that associates entities with `OneFrameUndoIgnore` components.
///
/// # Usage
///
/// This resource is managed internally by the undo system. It is updated automatically
/// when undo or redo operations are performed, and its contents are used to filter
/// which entity changes should be recorded for future undo operations.
#[derive(Resource, Default)]
pub struct UndoIgnoreStorage {
    /// A `HashMap` that associates entities which should be ignored by the undo system for a short period.
    pub storage: HashMap<Entity, OneFrameUndoIgnore>,
}

/// A resource that stores the previous state of components for automatic undo functionality.
///
/// `AutoUndoStorage<T>` is used internally by the undo system to keep track of component values
/// before they are changed. This allows the system to revert components to their previous state
/// when an undo operation is performed.
///
/// # Type Parameters
///
/// * `T`: The type of component being stored. Must implement the `Component` trait.
///
/// # Fields
///
/// * `storage`: A `HashMap` that associates entity IDs with their corresponding component values.
///
/// # Usage
///
/// This resource is automatically created and managed by the undo system when you use the
/// `auto_undo` or `auto_reflected_undo` methods. You typically don't need to interact with
/// this resource directly in your application code.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_undo::*;
///
///
/// App::new()
///     .add_plugins(UndoPlugin)
///     .auto_undo::<Transform>();
///
/// ```
///
/// In this example, `AutoUndoStorage<Transform>` will be automatically created and managed
/// by the undo system.
#[derive(Resource)]
pub struct AutoUndoStorage<T: Component> {
    /// The storage of "old" components values to allow store "old" component value in `ComponentChange<T>` change
    pub storage: HashMap<Entity, T>,
}

impl<T: Component> Default for AutoUndoStorage<T> {
    fn default() -> Self {
        Self {
            storage: HashMap::default(),
        }
    }
}

/// A trait that extends `App` with methods for setting up automatic undo functionality.
///
/// `AppAutoUndo` provides methods to easily configure automatic undo/redo support for
/// specific component types in your Bevy application.
///
/// # Methods
///
/// * `auto_undo<T: Component + Clone>`: Sets up automatic undo for components that implement
///   `Clone`.
/// * `auto_reflected_undo<T: Component + Reflect + FromReflect>`: Sets up automatic undo for
///   components that support reflection.
///
/// # Usage
///
/// Import this trait and use its methods in your `App` setup to enable automatic undo
/// functionality for specific components.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_undo::*;
///
///
///  App::new()
///       .add_plugins(UndoPlugin)
///       .auto_undo::<Transform>()
///       .auto_reflected_undo::<MyReflectedComponent>();
///
///
/// #[derive(Component, Reflect)]
/// struct MyReflectedComponent {
///     // fields...
/// }
/// ```
///
/// This example sets up automatic undo for the `Transform` component and a custom
/// `MyReflectedComponent` that supports reflection.
pub trait AppAutoUndo {
    /// Sets up automatic undo logic for components that implement `Clone`.
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self;
    /// Sets up automatic undo logic for components that implement `Reflect` and `FromReflect`.
    fn auto_reflected_undo<T: Component + Reflect + FromReflect>(&mut self) -> &mut Self;
}

impl AppAutoUndo for App {
    fn auto_undo<T: Component + Clone>(&mut self) -> &mut Self {
        if !self.world().contains_resource::<ChangeChain>() {
            return self;
        }

        self.world_mut()
            .insert_resource(AutoUndoStorage::<T>::default());
        self.add_message::<UndoRedoApplied<T>>();

        self.add_systems(
            PostUpdate,
            (
                auto_undo_update_cache::<T>,
                auto_undo_add_init::<T>,
                auto_undo_remove_detect::<T>,
                ApplyDeferred,
                auto_undo_system_changed::<T>,
                auto_undo_system::<T>,
            )
                .chain()
                .in_set(UndoSet::PerType),
        );

        self
    }

    fn auto_reflected_undo<T: Component + Reflect + FromReflect>(&mut self) -> &mut Self {
        if !self.world().contains_resource::<ChangeChain>() {
            return self;
        }

        self.world_mut()
            .insert_resource(AutoUndoStorage::<T>::default());
        self.add_message::<UndoRedoApplied<T>>();

        self.add_systems(
            PostUpdate,
            (
                auto_undo_reflected_update_cache::<T>,
                auto_undo_reflected_add_init::<T>,
                auto_undo_reflected_remove_detect::<T>,
                ApplyDeferred,
                auto_undo_system_changed::<T>,
                auto_undo_reflected_system::<T>,
            )
                .chain()
                .in_set(UndoSet::PerType),
        );

        self.add_systems(
            PostUpdate,
            auto_remap_undo_redo::<T>.in_set(UndoSet::Remapping),
        );

        self
    }
}

fn apply_for_every_typed_field<D: Reflect + TypePath>(
    value: &mut dyn PartialReflect,
    applyer: &dyn Fn(&mut D),
    max_recursion: i32,
) {
    if max_recursion < 0 {
        return;
    }

    if let Some(v) = value.try_downcast_mut::<D>() {
        applyer(v);
    } else {
        match value.reflect_mut() {
            bevy::reflect::ReflectMut::Struct(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_at_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::TupleStruct(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Tuple(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::List(s) => {
                for field_idx in 0..s.len() {
                    apply_for_every_typed_field(
                        s.get_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Array(s) => {
                for field_idx in 0..s.len() {
                    apply_for_every_typed_field(
                        s.get_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Map(_s) => {
                unimplemented!("See: https://github.com/bevyengine/bevy/pull/19802");
            }
            bevy::reflect::ReflectMut::Enum(s) => {
                for field_idx in 0..s.field_len() {
                    apply_for_every_typed_field(
                        s.field_at_mut(field_idx).unwrap(),
                        applyer,
                        max_recursion - 1,
                    );
                }
            }
            bevy::reflect::ReflectMut::Opaque(op) => {
                apply_for_every_typed_field(op, applyer, max_recursion - 1);
            }
            bevy::reflect::ReflectMut::Set(set) => {
                let mut queue_to_replace = vec![];
                for field in set.iter() {
                    if field.represents::<D>() {
                        queue_to_replace.push(field.reflect_clone().unwrap());
                    }
                }
                for mut field in queue_to_replace {
                    set.remove(field.as_ref());
                    applyer(field.as_mut().downcast_mut().unwrap());
                    set.insert_boxed(field);
                }
            }
        }
    }
}

fn auto_remap_undo_redo<T: Component + Reflect>(
    mut undoredo_applied: MessageReader<UndoRedoApplied<T>>,
    mut commands: Commands,
) {
    for event in undoredo_applied.read() {
        println!("remapping {:?}", event.entity);
        let entity_id = event.entity;
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world: &mut World, change_chain: Mut<ChangeChain>| {
                let mut entity = world.entity_mut(entity_id);
                if let Some(mut data) = entity.take::<T>() {
                    let reflect = data.as_reflect_mut();

                    apply_for_every_typed_field::<Entity>(
                        reflect.as_partial_reflect_mut(),
                        &|v| {
                            if let Some(e) = change_chain.entity_remap.get(v) {
                                println!("remap {v:?} to {e:?}");
                                *v = *e;
                            }
                        },
                        MAX_REFLECT_RECURSION,
                    );

                    entity.insert(data);
                }
            });
        });
    }
}

fn auto_undo_update_cache<T: Component + Clone>(
    mut storage: ResMut<AutoUndoStorage<T>>,
    ignored_query: Query<(Entity, &T), With<OneFrameUndoIgnore>>,
) {
    for (e, data) in ignored_query.iter() {
        storage.storage.insert(e, data.clone());
    }
}

fn auto_undo_reflected_update_cache<T: Component + Reflect + FromReflect>(
    mut storage: ResMut<AutoUndoStorage<T>>,
    ignored_query: Query<(Entity, &T), With<OneFrameUndoIgnore>>,
) {
    for (e, data) in ignored_query.iter() {
        storage
            .storage
            .insert(e, <T as FromReflect>::from_reflect(data).unwrap());
    }
}

fn auto_undo_add_init<T: Component + Clone>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<UndoMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
    just_maker_added_query: Query<(Entity, &T), (Added<UndoMarker>, Without<OneFrameUndoIgnore>)>,
    mut new_changes: MessageWriter<NewChange>,
) {
    for (e, data) in query.iter() {
        storage.storage.insert(e, data.clone());
        commands.entity(e).insert(OneFrameUndoIgnore::default());
        new_changes.write(NewChange::new(AddedComponent {
            new_value: data.clone(),
            entity: e,
        }));
    }

    for (e, data) in just_maker_added_query.iter() {
        storage.storage.insert(e, data.clone());
    }
}

fn auto_undo_reflected_add_init<T: Component + Reflect + FromReflect>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    query: Query<(Entity, &T), (With<UndoMarker>, Added<T>, Without<OneFrameUndoIgnore>)>,
    just_maker_added_query: Query<(Entity, &T), (Added<UndoMarker>, Without<OneFrameUndoIgnore>)>,
    mut new_changes: MessageWriter<NewChange>,
) {
    for (e, data) in query.iter() {
        storage
            .storage
            .insert(e, <T as FromReflect>::from_reflect(data).unwrap());
        commands.entity(e).insert(OneFrameUndoIgnore::default());
        new_changes.write(NewChange {
            change: Arc::new(ReflectedAddedComponent {
                new_value: <T as FromReflect>::from_reflect(data).unwrap(),
                entity: e,
            }),
        });
    }

    for (e, data) in just_maker_added_query.iter() {
        storage
            .storage
            .insert(e, <T as FromReflect>::from_reflect(data).unwrap());
    }
}

fn undo_ignore_tick(mut ignore_storage: ResMut<UndoIgnoreStorage>) {
    for (_, frame) in ignore_storage.storage.iter_mut() {
        frame.counter -= 1;
    }
    ignore_storage.storage.retain(|_, frame| frame.counter > 0);
}

fn auto_undo_remove_detect<T: Component + Clone>(
    _commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut removed_query: RemovedComponents<T>,
    mut new_changes: MessageWriter<NewChange>,
    ignore_storage: ResMut<UndoIgnoreStorage>,
) {
    for e in removed_query.read() {
        if !ignore_storage.storage.contains_key(&e)
            && let Some(prev_value) = storage.storage.remove(&e)
        {
            new_changes.write(NewChange {
                change: Arc::new(RemovedComponent {
                    old_value: prev_value,
                    entity: e,
                }),
            });
        }
    }
}

fn auto_undo_reflected_remove_detect<T: Component + Reflect + FromReflect>(
    _commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut removed_query: RemovedComponents<T>,
    mut new_changes: MessageWriter<NewChange>,
    ignore_storage: ResMut<UndoIgnoreStorage>,
) {
    for e in removed_query.read() {
        if !ignore_storage.storage.contains_key(&e)
            && let Some(prev_value) = storage.storage.remove(&e)
        {
            new_changes.write(NewChange {
                change: Arc::new(ReflectedRemovedComponent {
                    old_value: prev_value,
                    entity: e,
                }),
            });
        }
    }
}

fn auto_undo_system_changed<T: Component>(
    mut commands: Commands,
    query: Query<Entity, (With<UndoMarker>, Changed<T>, Without<OneFrameUndoIgnore>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(ChangedMarker::<T>::default());
    }
}

fn auto_undo_system<T: Component + Clone>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut query: Query<(Entity, Ref<T>), With<ChangedMarker<T>>>,
    mut new_change: MessageWriter<NewChange>,
) {
    for (e, data) in query.iter_mut() {
        if !data.is_changed() {
            commands.entity(e).remove::<ChangedMarker<T>>();

            if let Some(prev_value) = storage.storage.get(&e) {
                new_change.write(NewChange {
                    change: Arc::new(ComponentChange {
                        old_value: prev_value.clone(),
                        new_value: data.clone(),
                        entity: e,
                    }),
                });
                info!("Auto undo change for entity {:?}", e);
            }

            storage.storage.insert(e, data.clone());
        }
    }
}

fn auto_undo_reflected_system<T: Component + Reflect + FromReflect>(
    mut commands: Commands,
    mut storage: ResMut<AutoUndoStorage<T>>,
    mut query: Query<(Entity, Ref<T>, &mut ChangedMarker<T>)>,
    mut new_change: MessageWriter<NewChange>,
) {
    for (e, data, mut marker) in query.iter_mut() {
        if !data.is_changed() {
            marker.latency -= 1;
            if marker.latency > 0 {
                continue;
            }

            commands.entity(e).remove::<ChangedMarker<T>>();

            if let Some(prev_value) = storage.storage.get(&e) {
                new_change.write(NewChange {
                    change: Arc::new(ReflectedComponentChange {
                        old_value: <T as FromReflect>::from_reflect(prev_value).unwrap(),
                        new_value: <T as FromReflect>::from_reflect(data.as_ref()).unwrap(),
                        entity: e,
                    }),
                });
                info!("Auto undo change for entity {:?}", e);
            }

            storage
                .storage
                .insert(e, <T as FromReflect>::from_reflect(data.as_ref()).unwrap());
        } else {
            marker.latency = AUTO_UNDO_LATENCY;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn configure_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(UndoPlugin);
        app
    }

    #[test]
    fn test_undo() {
        let mut app = configure_app();
        app.auto_undo::<Name>();

        app.update();

        let test_id = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(NewChange {
            change: Arc::new(AddedEntity { entity: test_id }),
        });

        app.update();
        app.update();

        app.world_mut()
            .entity_mut(test_id)
            .insert(Name::default())
            .insert(UndoMarker);
        app.world_mut()
            .get_mut::<Name>(test_id)
            .unwrap()
            .set_changed();

        app.update();
        app.update();
        app.update();
        app.update();
        app.update();
        app.update();

        assert!(app.world_mut().get_entity(test_id).is_ok());

        app.world_mut().write_message(UndoRedo::Undo);

        app.update();
        app.update();

        app.update();
        app.update();
        app.update();

        assert!(app.world_mut().get::<Name>(test_id).is_none());
        assert!(app.world_mut().get_entity(test_id).is_ok());

        app.world_mut().write_message(UndoRedo::Undo);
        app.update();
        app.update();

        assert!(app.world_mut().get_entity(test_id).is_err());
    }

    #[test]
    fn test_undo_with_remap() {
        let mut app = configure_app();

        app.auto_reflected_undo::<ChildOf>();
        app.auto_reflected_undo::<Children>();

        let test_id_1 = app.world_mut().spawn(UndoMarker).id();
        let test_id_2 = app.world_mut().spawn(UndoMarker).id();

        app.world_mut().write_message(NewChange {
            change: Arc::new(AddedEntity { entity: test_id_1 }),
        });
        app.world_mut().write_message(NewChange {
            change: Arc::new(AddedEntity { entity: test_id_2 }),
        });

        app.update();
        app.update();

        app.world_mut().entity_mut(test_id_1).add_child(test_id_2);

        app.update();
        app.update();
        app.cleanup();

        app.world_mut().entity_mut(test_id_1).despawn();
        app.world_mut().write_message(NewChange {
            change: Arc::new(RemovedEntity { entity: test_id_1 }),
        });

        app.update();
        app.update();

        app.world_mut().write_message(UndoRedo::Undo);

        app.update();
        app.update();
        app.update();

        assert!(app.world_mut().get_entity(test_id_1).is_err());
        assert!(app.world_mut().get_entity(test_id_2).is_err());
        assert_eq!(app.world_mut().entities().len(), 2);

        let mut query = app.world_mut().query::<&Children>();
        assert!(query.single(app.world_mut()).is_ok());
    }
}

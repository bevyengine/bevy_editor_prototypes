//! Events and core data structures for the entity inspector.
//!
//! This module contains the event system that drives the inspector's updates,
//! as well as the core data structures for representing entity data.
//!
//! # Related Documentation
//!
//! - [Bevy Events](https://docs.rs/bevy/latest/bevy/ecs/event/index.html) - Core event system used by this inspector
//! - [Bevy Reflection](https://docs.rs/bevy/latest/bevy/reflect/index.html) - Reflection system for component introspection
//! - [`InspectorEvent`] - Main event enum for inspector state changes
//! - [`EntityInspectorRows`] - Central data store with change tracking

use bevy::ecs::event::BufferedEvent;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Events for entity inspector updates.
///
/// This event system provides granular change detection and enables efficient UI updates.
/// Instead of polling or hash-based detection, the inspector emits specific events when
/// entities or components change, allowing the UI to update only what's necessary.
///
/// For more details on Bevy's event system, see the [Events Guide](https://docs.rs/bevy/latest/bevy/ecs/event/index.html).
///
/// # Design: Single Enum vs Multiple Event Types
///
/// This is designed as a single enum rather than multiple distinct event types for several reasons:
/// - **Serde compatibility**: A single enum serializes more cleanly for remote inspection
/// - **Discoverability**: All related events are grouped together in one place
/// - **Shared handling**: All variants currently trigger UI rebuilds, so they can be processed uniformly
/// - **Future extensibility**: Easy to add new event types while maintaining backward compatibility
/// - **Type safety**: Ensures all inspector events go through the same processing pipeline
///
/// # Event Types
///
/// - **Entity Events**: `EntityAdded`, `EntityRemoved`, `EntityUpdated` - Fired when entities are created, destroyed, or modified
/// - **Component Events**: `ComponentAdded`, `ComponentRemoved`, `ComponentUpdated` - Fired for individual component changes
/// - **System Events**: `FullRefresh` - Forces a complete UI rebuild
///
/// # Usage
///
/// These events are typically emitted by data source plugins (like the remote inspection plugin)
/// and consumed by the main event handler (see [`crate::ui_systems::handle_inspector_events`]) to update the tree UI.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub enum InspectorEvent {
    /// Entity was added to the inspector.
    ///
    /// Triggers a full tree rebuild since the entity hierarchy may have changed.
    EntityAdded(Entity),

    /// Entity was removed from the inspector.
    ///
    /// Triggers a full tree rebuild since the entity hierarchy may have changed.
    EntityRemoved(Entity),

    /// Entity's components were updated.
    ///
    /// Allows for more targeted updates in the future, currently triggers a full rebuild.
    EntityUpdated(Entity),

    /// Component was added to an entity.
    ///
    /// Currently triggers an entity update, but could be used for more granular updates.
    ComponentAdded(Entity, String),

    /// Component was removed from an entity.
    ///
    /// Currently triggers an entity update, but could be used for more granular updates.
    ComponentRemoved(Entity, String),

    /// Component's data was updated.
    ///
    /// Currently triggers an entity update, but could be used for more granular updates.
    ComponentUpdated(Entity, String),

    /// Full refresh requested.
    ///
    /// Forces a complete rebuild of the entire tree, useful for initialization or error recovery.
    FullRefresh,
}

/// Data types that can be displayed in the inspector tree
#[derive(Clone, Debug)]
pub enum InspectorNodeData {
    /// An entity node
    Entity(Entity),
    /// A component node
    Component(String),
    /// A field node with name and value
    Field(String, String),
    /// An array/list item node with name and index
    Array(String, usize),
}

/// A single entity's inspection data.
///
/// Contains all the information needed to display an entity in the inspector tree,
/// including its display name, reflected component data, and change detection hash.
///
/// This structure leverages Bevy's [reflection system](https://docs.rs/bevy/latest/bevy/reflect/index.html)
/// to store component data in a format that can be displayed in the UI without knowing
/// the specific component types at compile time.
///
/// # Fields
///
/// - `name`: Display name for the entity (extracted from [`Name`](https://docs.rs/bevy/latest/bevy/core/struct.Name.html) component if available)
/// - `components`: Map of component type names to their reflected data using [`PartialReflect`](https://docs.rs/bevy/latest/bevy/reflect/trait.PartialReflect.html)
/// - `data_hash`: Optional hash of raw component data for efficient change detection
///
/// # Change Detection
///
/// The `data_hash` field enables efficient change detection for remote inspection.
/// When component data is received from a remote source, a hash is calculated from
/// the raw JSON data. This allows the system to quickly determine if an entity
/// has changed without expensive deep comparisons of reflected data.
#[derive(Debug)]
pub struct EntityInspectorRow {
    /// The display name of the entity.
    ///
    /// Extracted from the entity's `Name` component if available,
    /// otherwise defaults to "Entity {id}".
    pub name: String,

    /// The reflected components of the entity.
    ///
    /// Maps component type names (in format "`crate::Type`") to their
    /// reflected data using Bevy's [`PartialReflect`](https://docs.rs/bevy/latest/bevy/reflect/trait.PartialReflect.html) trait.
    /// Components without [`ReflectDeserialize`](https://docs.rs/bevy/latest/bevy/reflect/serde/trait.ReflectDeserialize.html) support
    /// are stored as placeholder [`DynamicStruct`](https://docs.rs/bevy/latest/bevy/reflect/struct.DynamicStruct.html) instances.
    pub components: HashMap<String, Box<dyn PartialReflect>>,

    /// Hash of the raw component data for change detection.
    ///
    /// Used primarily for remote inspection to efficiently detect
    /// when entity data has changed between polling cycles.
    /// Local inspection may leave this as `None`.
    pub data_hash: Option<u64>,
}

impl Default for EntityInspectorRow {
    fn default() -> Self {
        Self {
            name: String::new(),
            components: HashMap::new(),
            data_hash: None,
        }
    }
}

/// Collection of all entity inspection data with change tracking.
///
/// This resource serves as the central data store for the entity inspector,
/// maintaining both the current state of all inspected entities and tracking
/// what has changed since the last update cycle.
///
/// # Change Tracking
///
/// The struct maintains separate vectors for entities that have been added,
/// removed, or updated. This enables the event system to emit only the
/// necessary events and allows the UI to perform targeted updates.
///
/// # Usage Pattern
///
/// 1. Data sources (local queries, remote polling) call `update_rows()` with new data
/// 2. The method automatically detects changes and populates the change tracking vectors
/// 3. Event emitters check `has_changes()` and emit appropriate `InspectorEvent`s
/// 4. After events are emitted, `clear_changes()` resets the tracking state
///
/// # Performance
///
/// Change detection uses efficient hash-based comparison for remote data,
/// and simple `HashMap` key comparison for local data. The system is designed
/// to scale well with large numbers of entities.
#[derive(Resource, Debug)]
pub struct EntityInspectorRows {
    /// Map of entity to its inspection data.
    ///
    /// Contains the current state of all entities being inspected.
    /// The key is the Bevy `Entity` and the value contains all
    /// displayable information about that entity.
    pub rows: HashMap<Entity, EntityInspectorRow>,

    /// Last known entity count for change detection.
    ///
    /// Used as a quick check to determine if the entity set
    /// has changed size, which can help optimize change detection.
    pub last_entity_count: usize,

    /// Entities that were added since last check.
    ///
    /// Populated by `update_rows()` and consumed by event emitters.
    /// Cleared by `clear_changes()`.
    pub added_entities: Vec<Entity>,

    /// Entities that were removed since last check.
    ///
    /// Populated by `update_rows()` and consumed by event emitters.
    /// Cleared by `clear_changes()`.
    pub removed_entities: Vec<Entity>,

    /// Entities that were updated since last check.
    ///
    /// Populated by `update_rows()` and consumed by event emitters.
    /// Cleared by `clear_changes()`.
    pub updated_entities: Vec<Entity>,
}

impl Default for EntityInspectorRows {
    fn default() -> Self {
        Self {
            rows: HashMap::new(),
            last_entity_count: 0,
            added_entities: Vec::new(),
            removed_entities: Vec::new(),
            updated_entities: Vec::new(),
        }
    }
}

impl EntityInspectorRows {
    /// Updates the entity data and automatically tracks changes.
    ///
    /// This method performs a comprehensive diff between the current entity data
    /// and the provided new data, populating the change tracking vectors with
    /// entities that have been added, removed, or modified.
    ///
    /// # Arguments
    ///
    /// * `new_rows` - The updated entity data to replace the current data
    ///
    /// # Change Detection Logic
    ///
    /// - **Added**: Entities in `new_rows` but not in current `rows`
    /// - **Removed**: Entities in current `rows` but not in `new_rows`
    /// - **Updated**: Entities present in both with different data (component count, names, or hash)
    /// - **Initial Population**: If current `rows` is empty and `new_rows` is not, all entities are marked as added
    ///
    /// # Performance
    ///
    /// The method uses hash comparison when available (for remote data) or falls back
    /// to component name comparison. This provides efficient change detection even
    /// with large numbers of entities.
    pub fn update_rows(&mut self, new_rows: HashMap<Entity, EntityInspectorRow>) {
        // Special case: if we're going from empty to non-empty, consider all entities as added
        let is_initial_population = self.rows.is_empty() && !new_rows.is_empty();

        if is_initial_population {
            // Mark all entities as added for initial population
            for entity in new_rows.keys() {
                self.added_entities.push(*entity);
            }
        } else {
            // Find added entities
            for entity in new_rows.keys() {
                if !self.rows.contains_key(entity) {
                    self.added_entities.push(*entity);
                }
            }
        }

        // Find removed entities
        for entity in self.rows.keys() {
            if !new_rows.contains_key(entity) {
                self.removed_entities.push(*entity);
            }
        }

        // Find updated entities (entities that exist in both but have different data)
        for (entity, new_row) in &new_rows {
            if let Some(old_row) = self.rows.get(entity) {
                // Compare component counts as a quick check
                if old_row.components.len() != new_row.components.len()
                    || old_row.name != new_row.name
                {
                    self.updated_entities.push(*entity);
                } else if let (Some(old_hash), Some(new_hash)) =
                    (old_row.data_hash, new_row.data_hash)
                {
                    // If we have hashes, use them for comparison
                    if old_hash != new_hash {
                        self.updated_entities.push(*entity);
                    }
                } else {
                    // Check if component names changed
                    let old_comp_names: std::collections::HashSet<_> =
                        old_row.components.keys().collect();
                    let new_comp_names: std::collections::HashSet<_> =
                        new_row.components.keys().collect();
                    if old_comp_names != new_comp_names {
                        self.updated_entities.push(*entity);
                    }
                }
            }
        }

        self.rows = new_rows;
        self.last_entity_count = self.rows.len();
    }

    /// Clears all change tracking data.
    ///
    /// Should be called after processing change events to reset the tracking
    /// state for the next update cycle. This prevents the same changes from
    /// being reported multiple times.
    pub fn clear_changes(&mut self) {
        self.added_entities.clear();
        self.removed_entities.clear();
        self.updated_entities.clear();
    }

    /// Checks if there are any tracked changes.
    ///
    /// Returns `true` if any entities have been added, removed, or updated
    /// since the last call to `clear_changes()`. This is used by event
    /// emitters to determine whether to emit change events.
    ///
    /// # Returns
    ///
    /// `true` if changes are pending, `false` if no changes have been detected.
    pub fn has_changes(&self) -> bool {
        !self.added_entities.is_empty()
            || !self.removed_entities.is_empty()
            || !self.updated_entities.is_empty()
    }
}

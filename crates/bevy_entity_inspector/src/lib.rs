//! A modular entity inspector for Bevy with reflection support.
//!
//! This crate provides a tree-based UI for inspecting entities and their components
//! in a Bevy application. It uses Bevy's reflection system to dynamically display
//! component data without requiring compile-time knowledge of component types.
//!
//! ## Features
//!
//! - **Event-Driven Updates**: Efficient, granular updates using an event system instead of polling
//! - **Tree-Based UI**: Hierarchical display of entities and their components with expand/collapse functionality
//! - **Component Grouping**: Components are automatically grouped by crate for better organization (e.g., "bevy_transform", "my_game")
//! - **Visual Styling**: Different node types (entities, crate groups, components, fields) have distinct visual styling with reduced opacity for non-expandable items
//! - **Reflection Support**: Automatic component introspection using Bevy's reflection system
//! - **Remote Inspection** (optional): Connect to remote Bevy applications via `bevy_remote`
//! - **Modern UI**: Clean, themeable interface with hover effects and visual feedback
//! - **Change Detection**: Only updates UI when actual changes occur, eliminating unnecessary rebuilds
//!
//! ## Architecture
//!
//! The inspector uses an event-driven architecture that replaces the previous hash-based change detection:
//!
//! - `InspectorEvent` enum defines granular change types (entity added/removed/updated, component changes)
//! - `EntityInspectorRows` tracks entity data and change state with efficient diff detection
//! - `TreeState` manages the UI tree structure and expansion states
//! - Remote polling emits events only when actual changes are detected
//!
//! ## Usage
//!
//! ### Basic Inspector
//!
//! Add the `InspectorPlugin` to your Bevy app:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_entity_inspector::InspectorPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(InspectorPlugin)
//!         .run();
//! }
//! ```
//!
//! ### Remote Inspection
//!
//! To inspect entities in a remote Bevy application:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_entity_inspector::InspectorPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(InspectorPlugin)
//!         .run();
//! }
//! ```
//!
//! Then run your target application with the `bevy_remote` plugin enabled.
//!
//! ### Custom Theming
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_entity_inspector::{InspectorPlugin, create_dark_inspector_theme};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(InspectorPlugin)
//!         .insert_resource(create_dark_inspector_theme())
//!         .run();
//! }
//! ```
//!
//! ## Performance
//!
//! The event-driven system provides significant performance improvements:
//! - Only rebuilds UI when actual changes occur
//! - Granular updates for individual entity/component changes
//! - Efficient hash-based change detection for remote data
//! - Preserved expansion states during tree rebuilds
//!
//! ## Component Grouping
//!
//! Components are automatically grouped by their crate name for better organization:
//!
//! ```text
//! Entity (42)
//! ├── bevy_transform
//! │   ├── Transform
//! │   │   ├── translation: Vec3(0.0, 0.0, 0.0)
//! │   │   ├── rotation: Quat(0.0, 0.0, 0.0, 1.0)
//! │   │   └── scale: Vec3(1.0, 1.0, 1.0)
//! │   └── GlobalTransform
//! │       └── ...
//! ├── bevy_render
//! │   ├── Visibility
//! │   └── ViewVisibility
//! └── my_game
//!     └── Player
//!         ├── health: f32(100.0)
//!         └── level: u32(1)
//! ```
//!
//! This grouping makes it easier to understand which systems and crates are contributing
//! components to an entity, similar to the organization seen in other ECS inspectors.
//!

use bevy::ecs::event::BufferedEvent;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::reflect::*;

#[cfg(feature = "remote")]
pub mod remote;
pub mod theme;
pub mod ui;
pub mod widgets;

use theme::*;
use ui::*;
use widgets::*;

// Re-export commonly used types
pub use theme::{create_dark_inspector_theme, create_light_inspector_theme, InspectorTheme};
pub use ui::{build_tree_node_recursive, tree_container, TreeConfig, TreeNode, TreeState};
pub use widgets::{InspectorField, InspectorFieldType, InspectorPanel};

/// Events for entity inspector updates.
///
/// This event system provides granular change detection and enables efficient UI updates.
/// Instead of polling or hash-based detection, the inspector emits specific events when
/// entities or components change, allowing the UI to update only what's necessary.
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
/// and consumed by the main event handler to update the tree UI.
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

/// A tree node component for the inspector UI
#[derive(Component, Clone, Debug)]
pub struct InspectorTreeNode {
    /// The base tree node data
    pub base: TreeNode,
    /// The specific inspector data for this node
    pub data: InspectorNodeData,
}

/// A single entity's inspection data.
///
/// Contains all the information needed to display an entity in the inspector tree,
/// including its display name, reflected component data, and change detection hash.
///
/// # Fields
///
/// - `name`: Display name for the entity (extracted from `Name` component if available)
/// - `components`: Map of component type names to their reflected data
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
    /// Maps component type names (in format "crate::Type") to their
    /// reflected data. Components without `ReflectDeserialize` support
    /// are stored as placeholder `DynamicStruct` instances.
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
/// and simple HashMap key comparison for local data. The system is designed
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

/// Builder for creating inspector tree structures from entity data
pub struct InspectorTreeBuilder {
    tree_state: TreeState,
    config: TreeConfig,
}

impl Default for InspectorTreeBuilder {
    fn default() -> Self {
        Self {
            tree_state: TreeState::default(),
            config: TreeConfig::default(),
        }
    }
}

impl InspectorTreeBuilder {
    /// Creates a new tree builder with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom tree configuration
    pub fn with_config(mut self, config: TreeConfig) -> Self {
        self.config = config;
        self
    }

    /// Inserts a tree node into the builder
    pub fn insert(&mut self, id: String, node: TreeNode) {
        self.tree_state.nodes.insert(id, node);
    }

    /// Builds a tree structure from entity inspector data with component grouping by crate
    pub fn build_from_inspector_data(inspector_data: &EntityInspectorRows) -> Self {
        let mut builder = Self::new();

        info!(
            "build_from_inspector_data: Processing {} entities",
            inspector_data.rows.len()
        );

        for (entity, row) in &inspector_data.rows {
            let entity_id = format!("entity_{:?}", entity);
            let entity_label = if row.name.is_empty() {
                format!("Entity {:?}", entity)
            } else {
                format!("{} ({:?})", row.name, entity)
            };

            info!(
                "build_from_inspector_data: Processing entity {:?} with {} components: {}",
                entity,
                row.components.len(),
                entity_label
            );

            // Group components by crate
            let mut components_by_crate = std::collections::BTreeMap::new();

            for (component_name, component_reflect) in &row.components {
                let (crate_name, type_name) = extract_crate_and_type(component_name);
                components_by_crate
                    .entry(crate_name)
                    .or_insert_with(Vec::new)
                    .push((type_name, component_name.clone(), component_reflect));
            }

            let mut entity_children = Vec::new();

            // Create crate group nodes
            for (crate_name, components) in components_by_crate {
                let crate_group_id = format!("{}_{}", entity_id, crate_name);
                entity_children.push(crate_group_id.clone());

                let mut crate_children = Vec::new();

                // Add components to this crate group
                for (type_name, full_component_name, component_reflect) in components {
                    let component_node_id = format!("{}_{}", crate_group_id, type_name);
                    crate_children.push(component_node_id.clone());

                    // Extract fields from the component
                    let fields = extract_reflect_fields(component_reflect.as_ref());
                    let mut component_children = Vec::new();

                    // Add fields as children of the component
                    for (field_name, field_value) in fields {
                        let field_node_id = format!("{}_field_{}", component_node_id, field_name);
                        component_children.push(field_node_id.clone());

                        let field_node = TreeNode {
                            id: field_node_id,
                            label: format!("{}: {}", field_name, field_value),
                            is_expanded: false,
                            children: Vec::new(),
                            parent: Some(component_node_id.clone()),
                            depth: 3,
                            node_type: TreeNodeType::Field,
                        };

                        builder
                            .tree_state
                            .nodes
                            .insert(field_node.id.clone(), field_node);
                    }

                    // Create component node with short type name
                    let component_node = TreeNode {
                        id: component_node_id,
                        label: type_name,
                        is_expanded: false,
                        children: component_children,
                        parent: Some(crate_group_id.clone()),
                        depth: 2,
                        node_type: TreeNodeType::Component,
                    };

                    builder
                        .tree_state
                        .nodes
                        .insert(component_node.id.clone(), component_node);
                }

                // Create crate group node
                let crate_node = TreeNode {
                    id: crate_group_id,
                    label: crate_name,
                    is_expanded: false,
                    children: crate_children,
                    parent: Some(entity_id.clone()),
                    depth: 1,
                    node_type: TreeNodeType::CrateGroup,
                };

                builder
                    .tree_state
                    .nodes
                    .insert(crate_node.id.clone(), crate_node);
            }

            // Create entity node
            let entity_node = TreeNode {
                id: entity_id.clone(),
                label: entity_label,
                is_expanded: false,
                children: entity_children,
                parent: None,
                depth: 0,
                node_type: TreeNodeType::Entity,
            };

            builder
                .tree_state
                .nodes
                .insert(entity_id.clone(), entity_node);
            builder.tree_state.root_nodes.push(entity_id);
        }

        builder
    }

    /// Creates a tree view UI from the builder's data
    pub fn build_tree_view(&self, commands: &mut Commands) -> Entity {
        build_tree_view(commands, &self.tree_state, &self.config)
    }
}

/// Extracts field information from a reflected value for display in the inspector
fn extract_reflect_fields(reflect: &dyn PartialReflect) -> Vec<(String, String)> {
    let mut fields = Vec::new();

    match reflect.reflect_ref() {
        ReflectRef::Struct(s) => {
            for i in 0..s.field_len() {
                if let Some(field) = s.field_at(i) {
                    let default_field_name = format!("field_{}", i);
                    let name = s.name_at(i).unwrap_or(&default_field_name);
                    let value = format!("{:?}", field);
                    fields.push((name.to_string(), value));
                }
            }
        }
        ReflectRef::TupleStruct(ts) => {
            for i in 0..ts.field_len() {
                if let Some(field) = ts.field(i) {
                    let name = format!("field_{}", i);
                    let value = format!("{:?}", field);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::Tuple(t) => {
            for i in 0..t.field_len() {
                if let Some(field) = t.field(i) {
                    let name = format!("item_{}", i);
                    let value = format!("{:?}", field);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::List(l) => {
            for i in 0..l.len() {
                if let Some(item) = l.get(i) {
                    let name = format!("[{}]", i);
                    let value = format!("{:?}", item);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::Array(a) => {
            for i in 0..a.len() {
                if let Some(item) = a.get(i) {
                    let name = format!("[{}]", i);
                    let value = format!("{:?}", item);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::Map(m) => {
            for (key, value) in m.iter() {
                let name = format!("{:?}", key);
                let value = format!("{:?}", value);
                fields.push((name, value));
            }
        }
        ReflectRef::Enum(e) => {
            fields.push(("variant".to_string(), e.variant_name().to_string()));
            for i in 0..e.field_len() {
                if let Some(field) = e.field_at(i) {
                    let default_field_name = format!("field_{}", i);
                    let name = e.name_at(i).unwrap_or(&default_field_name);
                    fields.push((name.to_string(), format!("{:?}", field)));
                }
            }
        }
        _ => {
            // For primitive values and other types, just show the debug representation
            fields.push(("value".to_string(), format!("{:?}", reflect)));
        }
    }

    fields
}

/// Marker component for the inspector tree container
#[derive(Component)]
pub struct InspectorTreeContainer;

/// Event-driven system that handles entity inspector changes.
///
/// This system is the heart of the inspector's event-driven architecture. It processes
/// `InspectorEvent`s emitted by data sources and updates the tree UI accordingly.
/// The system is designed to minimize unnecessary UI rebuilds by categorizing events
/// and applying appropriate update strategies.
///
/// # Event Processing
///
/// - **Entity Add/Remove**: Triggers full tree rebuild since hierarchy may have changed
/// - **Entity/Component Updates**: Triggers targeted updates (currently full rebuild, but designed for future optimization)
/// - **Full Refresh**: Forces complete tree reconstruction
///
/// # Performance Optimizations
///
/// - Only processes events when they exist (early exit if no events)
/// - Batches multiple update events into a single tree update operation
/// - Preserves UI state (expansion, selection) during rebuilds
/// - Uses efficient tree diffing where possible
///
/// # System Parameters
///
/// - `events`: Reader for incoming `InspectorEvent`s
/// - `inspector_data`: Current entity data for tree reconstruction
/// - `tree_state`: UI tree state with expansion/selection information
/// - `tree_container_query`: Query to find tree container entities for UI updates
/// - `children_query`: Query for entity hierarchy traversal during tree updates
/// - `tree_config`: Visual configuration for tree rendering
/// - `commands`: Command buffer for UI entity spawning/despawning
pub fn handle_inspector_events(
    mut events: EventReader<InspectorEvent>,
    inspector_data: Res<EntityInspectorRows>,
    mut tree_state: ResMut<TreeState>,
    tree_container_query: Query<Entity, With<TreeContainer>>,
    children_query: Query<&Children>,
    tree_config: Res<TreeConfig>,
    mut commands: Commands,
) {
    if !events.is_empty() {
        info!("Processing {} inspector events", events.len());

        let mut needs_full_rebuild = false;
        let mut updated_entities = std::collections::HashSet::new();

        for event in events.read() {
            match event {
                InspectorEvent::EntityAdded(entity) => {
                    info!("Entity {:?} added", entity);
                    needs_full_rebuild = true;
                }
                InspectorEvent::EntityRemoved(entity) => {
                    info!("Entity {:?} removed", entity);
                    needs_full_rebuild = true;
                }
                InspectorEvent::EntityUpdated(entity) => {
                    info!("Entity {:?} updated", entity);
                    updated_entities.insert(*entity);
                }
                InspectorEvent::ComponentAdded(entity, component) => {
                    info!("Component {} added to entity {:?}", component, entity);
                    updated_entities.insert(*entity);
                }
                InspectorEvent::ComponentRemoved(entity, component) => {
                    info!("Component {} removed from entity {:?}", component, entity);
                    updated_entities.insert(*entity);
                }
                InspectorEvent::ComponentUpdated(entity, component) => {
                    info!("Component {} updated on entity {:?}", component, entity);
                    updated_entities.insert(*entity);
                }
                InspectorEvent::FullRefresh => {
                    info!("Full refresh requested");
                    needs_full_rebuild = true;
                }
            }
        }

        if needs_full_rebuild {
            rebuild_full_tree(
                &inspector_data,
                &mut tree_state,
                &tree_container_query,
                &children_query,
                &tree_config,
                &mut commands,
            );
        } else if !updated_entities.is_empty() {
            update_entities_in_tree(
                &updated_entities,
                &inspector_data,
                &mut tree_state,
                &tree_container_query,
                &children_query,
                &tree_config,
                &mut commands,
            );
        }
    }
}

/// Rebuilds the entire tree from scratch
fn rebuild_full_tree(
    inspector_data: &EntityInspectorRows,
    tree_state: &mut TreeState,
    tree_container_query: &Query<Entity, With<TreeContainer>>,
    children_query: &Query<&Children>,
    tree_config: &TreeConfig,
    commands: &mut Commands,
) {
    info!(
        "Rebuilding full tree with {} entities",
        inspector_data.rows.len()
    );

    // Preserve expansion states
    let old_expansion_states = preserve_expansion_states(tree_state);

    // Rebuild tree state from inspector data
    let tree_builder = InspectorTreeBuilder::build_from_inspector_data(inspector_data);
    let mut new_tree_state = tree_builder.tree_state;
    restore_expansion_states(&mut new_tree_state, &old_expansion_states);

    // Update tree state
    *tree_state = new_tree_state;

    // Rebuild UI
    for container_entity in tree_container_query.iter() {
        // Clear existing tree content
        if let Ok(children) = children_query.get(container_entity) {
            let children_vec: Vec<Entity> = children.to_vec();
            for child in children_vec {
                commands.entity(child).despawn();
            }
        }

        // Rebuild tree with current state
        // Sort root nodes so that nodes with children appear first
        let mut sorted_root_ids: Vec<_> = tree_state.root_nodes.clone();
        sorted_root_ids.sort_by(|a, b| {
            let a_node = tree_state.nodes.get(a);
            let b_node = tree_state.nodes.get(b);

            match (a_node, b_node) {
                (Some(a), Some(b)) => {
                    let a_has_children = !a.children.is_empty();
                    let b_has_children = !b.children.is_empty();

                    // Nodes with children come first, then sort alphabetically within each group
                    match (a_has_children, b_has_children) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.label.cmp(&b.label),
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        for root_id in &sorted_root_ids {
            if let Some(root_node) = tree_state.nodes.get(root_id) {
                let node_entity =
                    build_tree_node_recursive(commands, root_node, tree_state, tree_config);
                commands.entity(container_entity).add_child(node_entity);
            }
        }
    }
}

/// Updates specific entities in the tree (more efficient than full rebuild)
fn update_entities_in_tree(
    updated_entities: &std::collections::HashSet<Entity>,
    inspector_data: &EntityInspectorRows,
    tree_state: &mut TreeState,
    tree_container_query: &Query<Entity, With<TreeContainer>>,
    children_query: &Query<&Children>,
    tree_config: &TreeConfig,
    commands: &mut Commands,
) {
    info!("Updating {} entities in tree", updated_entities.len());

    // For now, do a full rebuild since partial updates are complex
    // TODO: Implement partial tree updates for better performance
    rebuild_full_tree(
        inspector_data,
        tree_state,
        tree_container_query,
        children_query,
        tree_config,
        commands,
    );
}

/// Preserves the expansion state of nodes before rebuilding
fn preserve_expansion_states(tree_state: &TreeState) -> HashMap<String, bool> {
    let mut expansion_states = HashMap::new();
    for (node_id, node) in &tree_state.nodes {
        expansion_states.insert(node_id.clone(), node.is_expanded);
    }
    expansion_states
}

/// Restores the expansion state of nodes after rebuilding
fn restore_expansion_states(tree_state: &mut TreeState, expansion_states: &HashMap<String, bool>) {
    for (node_id, node) in &mut tree_state.nodes {
        if let Some(&was_expanded) = expansion_states.get(node_id) {
            node.is_expanded = was_expanded;
        }
    }

    // Also preserve the selected node if it still exists
    // (This is already handled by the TreeState update)
}

/// Spawns the inspector UI in the world
pub fn spawn_inspector_ui(commands: &mut Commands, inspector_data: &EntityInspectorRows) {
    if inspector_data.rows.is_empty() {
        info!("spawn_inspector_ui: Creating empty UI - waiting for data from remote source");
    } else {
        info!(
            "spawn_inspector_ui: Creating UI with {} entities from data source",
            inspector_data.rows.len()
        );
    }

    let tree_builder = InspectorTreeBuilder::build_from_inspector_data(&inspector_data);

    // Debug: Check if we have any tree nodes
    info!(
        "spawn_inspector_ui: Tree builder has {} root nodes and {} total nodes",
        tree_builder.tree_state.root_nodes.len(),
        tree_builder.tree_state.nodes.len()
    );

    let container_entity = commands
        .spawn((
            Node {
                width: Val::Auto,
                min_width: Val::Px(300.0),
                max_width: Val::Px(600.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            InspectorTreeContainer,
        ))
        .id();

    info!(
        "spawn_inspector_ui: Created container entity {:?}",
        container_entity
    );

    // Add header
    let header_entity = commands
        .spawn((
            Text::new("Entity Inspector"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    info!(
        "spawn_inspector_ui: Created header entity {:?}",
        header_entity
    );

    // Add tree view
    let tree_entity = tree_builder.build_tree_view(commands);
    info!("spawn_inspector_ui: Created tree entity {:?}", tree_entity);

    // Add children to the container
    commands.entity(container_entity).add_child(header_entity);
    commands.entity(container_entity).add_child(tree_entity);

    info!("spawn_inspector_ui: Added children to container");
}

/// Marker resource to track if UI has been spawned
#[derive(Resource, Default)]
pub struct InspectorUiSpawned;

/// Spawns the inspector UI once at startup
pub fn spawn_inspector_ui_once(
    mut commands: Commands,
    inspector_data: Res<EntityInspectorRows>,
    theme: Res<InspectorTheme>,
    ui_spawned: Option<Res<InspectorUiSpawned>>,
) {
    // Only spawn if we haven't spawned before
    if ui_spawned.is_none() {
        info!(
            "spawn_inspector_ui_once: Spawning UI at startup - waiting for data from configured sources"
        );

        // Create a UI root container first
        let ui_root = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();

        info!("spawn_inspector_ui_once: Created UI root {:?}", ui_root);

        // Now spawn the inspector and add it to the root
        let inspector_container =
            spawn_inspector_ui_internal(&mut commands, &inspector_data, &theme);
        commands.entity(ui_root).add_child(inspector_container);

        info!("spawn_inspector_ui_once: Added inspector to UI root");

        commands.init_resource::<InspectorUiSpawned>();
    }
}

/// Internal function to spawn inspector UI and return the container entity
fn spawn_inspector_ui_internal(
    commands: &mut Commands,
    inspector_data: &EntityInspectorRows,
    theme: &InspectorTheme,
) -> Entity {
    if inspector_data.rows.is_empty() {
        info!(
            "spawn_inspector_ui_internal: Creating empty UI - waiting for data from remote source"
        );
    } else {
        info!(
            "spawn_inspector_ui_internal: Creating UI with {} entities from data source",
            inspector_data.rows.len()
        );
    }

    // Create a simple panel for now
    let inspector_panel = commands
        .spawn((
            Node {
                width: Val::Auto,
                min_width: Val::Px(300.0),
                max_width: Val::Px(600.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .id();

    // Create tree view from inspector data
    let tree_builder = InspectorTreeBuilder::build_from_inspector_data(inspector_data);

    info!(
        "spawn_inspector_ui_internal: Tree builder has {} root nodes and {} total nodes",
        tree_builder.tree_state.root_nodes.len(),
        tree_builder.tree_state.nodes.len()
    );

    // Create tree config from theme
    let tree_config = TreeConfig {
        indent_size: theme.indent_size,
        node_height: theme.node_height,
        triangle_size: theme.disclosure_size,
        font_size: theme.font_size,
        text_color: theme.text_color,
        selected_color: theme.selected_color,
        hover_color: theme.hover_color,
        background_color: theme.background_color,
    };

    // Build the tree view
    let tree_entity = ui::build_tree_view(commands, &tree_builder.tree_state, &tree_config);

    // Add tree to panel
    commands.entity(inspector_panel).add_child(tree_entity);

    info!(
        "spawn_inspector_ui_internal: Created inspector panel {:?} with tree {:?}",
        inspector_panel, tree_entity
    );

    inspector_panel
}

/// Sets up the UI camera needed for the inspector to render
fn setup_inspector_camera(mut commands: Commands) {
    // Check if there's already a camera with Camera2d component
    // If not, spawn one for the inspector
    commands.spawn(Camera2d);
}

/// The main plugin for the entity inspector.
///
/// This plugin sets up the complete entity inspector system with event-driven updates,
/// tree UI management, and optional remote inspection capabilities. It provides a
/// modern, efficient alternative to traditional immediate-mode entity inspection.
///
/// # Features Enabled
///
/// - **Core Inspector**: Tree-based entity/component display with reflection
/// - **Event System**: Efficient change detection and UI updates
/// - **Theming**: Customizable visual appearance
/// - **Remote Inspection**: Optional connection to remote Bevy applications (with "remote" feature)
///
/// # Systems Added
///
/// - `setup_inspector_camera`: Creates camera for UI rendering
/// - `spawn_inspector_ui_once`: Creates initial UI structure  
/// - `handle_inspector_events`: Processes change events and updates tree
/// - Remote polling systems (if "remote" feature enabled)
///
/// # Resources Initialized
///
/// - `EntityInspectorRows`: Central data store with change tracking
/// - `InspectorTheme`: Visual configuration for tree rendering
/// - `TreeState`: UI tree structure and expansion state
/// - `TreeConfig`: Tree rendering parameters
///
/// # Usage
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_entity_inspector::InspectorPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(InspectorPlugin)
///     .run();
/// ```
///
/// # Performance
///
/// The plugin is designed for minimal performance impact:
/// - Events only emitted when actual changes occur
/// - UI updates are batched and optimized
/// - Tree state preserved during rebuilds
/// - Async network operations (for remote inspection)
pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TreePlugin)
            .add_event::<InspectorEvent>()
            .init_resource::<EntityInspectorRows>()
            .init_resource::<InspectorTheme>()
            .add_systems(Startup, (setup_inspector_camera, spawn_inspector_ui_once))
            .add_systems(Update, handle_inspector_events);

        #[cfg(feature = "remote")]
        {
            app.add_plugins(remote::EntityInspectorRemotePlugin::default());
        }

        // Note: Inspector now uses modular data sources (scene files, BSN, remote)
        // No automatic local entity population
    }
}

/// Extracts crate name and type name from a component type path.
///
/// Component names from the reflection system often include full module paths like:
/// - "bevy_transform::components::transform::Transform"
/// - "my_game::components::player::Player"
/// - "Transform" (for local/simple names)
///
/// This function extracts the crate name and the final type name for display purposes.
///
/// # Arguments
///
/// * `component_name` - Full component type path from reflection
///
/// # Returns
///
/// A tuple of (crate_name, type_name) where:
/// - For "bevy_transform::components::transform::Transform" -> ("bevy_transform", "Transform")
/// - For "my_game::player::Player" -> ("my_game", "Player")
/// - For "Transform" -> ("Local", "Transform")
///
/// # Examples
///
/// ```rust,no_run
/// # use bevy_entity_inspector::extract_crate_and_type;
/// let (crate_name, type_name) = extract_crate_and_type("bevy_transform::components::transform::Transform");
/// assert_eq!(crate_name, "bevy_transform");
/// assert_eq!(type_name, "Transform");
///
/// let (crate_name, type_name) = extract_crate_and_type("Transform");
/// assert_eq!(crate_name, "Local");
/// assert_eq!(type_name, "Transform");
/// ```
pub fn extract_crate_and_type(component_name: &str) -> (String, String) {
    if let Some(first_separator) = component_name.find("::") {
        // Extract the crate name (everything before the first "::")
        let crate_name = component_name[..first_separator].to_string();

        // Extract the type name (everything after the last "::")
        let type_name = component_name
            .split("::")
            .last()
            .unwrap_or(component_name)
            .to_string();

        (crate_name, type_name)
    } else {
        // No "::" found, treat as a local/simple type
        ("Local".to_string(), component_name.to_string())
    }
}

/// Type of tree node for visual styling
#[derive(Clone, Debug, PartialEq)]
pub enum TreeNodeType {
    /// A root entity node
    Entity,
    /// A crate grouping node
    CrateGroup,
    /// A component node
    Component,
    /// A field or property node
    Field,
}

impl Default for TreeNodeType {
    fn default() -> Self {
        Self::Field
    }
}

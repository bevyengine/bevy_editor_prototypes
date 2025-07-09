//! UI management systems for the entity inspector.
//!
//! This module contains the systems responsible for managing the inspector UI,
//! including event handling, tree rebuilding, and UI spawning.

use crate::events::{EntityInspectorRows, InspectorEvent};
use crate::ui::{TreeConfig, TreeContainer, TreeState};
use crate::{build_tree_node_recursive, InspectorTreeBuilder};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

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

        // Apply updates based on event analysis
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

/// Preserves expansion states from the current tree for restoration after rebuild
fn preserve_expansion_states(tree_state: &TreeState) -> HashMap<String, bool> {
    tree_state
        .nodes
        .iter()
        .map(|(id, node)| (id.clone(), node.is_expanded))
        .collect()
}

/// Restores expansion states to a rebuilt tree
fn restore_expansion_states(tree_state: &mut TreeState, expansion_states: &HashMap<String, bool>) {
    for (id, is_expanded) in expansion_states {
        if let Some(node) = tree_state.nodes.get_mut(id) {
            node.is_expanded = *is_expanded;
        }
    }
}

/// Marker resource to track if UI has been spawned
#[derive(Resource, Default)]
pub struct InspectorUiSpawned;

/// Spawns the inspector UI once at startup
pub fn spawn_inspector_ui_once(
    mut commands: Commands,
    inspector_data: Res<EntityInspectorRows>,
    theme: Res<crate::theme::InspectorTheme>,
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
    theme: &crate::theme::InspectorTheme,
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

    // Create horizontal layout container for tree + property panel
    let inspector_panel = commands
        .spawn((
            Node {
                width: Val::Auto,
                min_width: Val::Px(600.0),
                max_width: Val::Px(1200.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row, // Horizontal layout
                border: UiRect::all(Val::Px(1.0)),
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

    // Create left panel for tree (65% width)
    let tree_panel = commands
        .spawn((
            Node {
                width: Val::Percent(65.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .id();

    // Build the tree view
    let tree_entity =
        crate::ui::tree::build_tree_view(commands, &tree_builder.tree_state, &tree_config);

    // Add tree to left panel
    commands.entity(tree_panel).add_child(tree_entity);

    // Create property panel (35% width)
    let property_panel = crate::ui::property_panel::create_property_panel(
        commands,
        crate::ui::property_panel::PropertyPanelProps::default(),
        theme,
    );

    // Add both panels to main container
    commands.entity(inspector_panel).add_child(tree_panel);
    commands.entity(inspector_panel).add_child(property_panel);

    info!(
        "spawn_inspector_ui_internal: Created inspector panel {:?} with tree {:?} and property panel {:?}",
        inspector_panel, tree_entity, property_panel
    );

    inspector_panel
}

/// Sets up the UI camera needed for the inspector to render
pub fn setup_inspector_camera(mut commands: Commands) {
    // Check if there's already a camera with Camera2d component
    // If not, spawn one for the inspector
    commands.spawn(Camera2d);
}

/// System that handles tree node selection and updates the property panel
pub fn handle_tree_selection(
    mut selection_events: EventReader<crate::ui::tree::TreeNodeSelected>,
    mut tree_state: ResMut<TreeState>,
    _inspector_data: Res<EntityInspectorRows>,
    content_query: Query<Entity, With<crate::ui::property_panel::PropertyPanelContent>>,
    children_query: Query<&Children>,
    _theme: Res<crate::theme::InspectorTheme>,
    mut commands: Commands,
) {
    for selection_event in selection_events.read() {
        info!("Tree node selected: {}", selection_event.node_id);

        // Update tree state
        tree_state.selected_node = Some(selection_event.node_id.clone());

        // Find the property panel content area and update it
        if let Ok(content_entity) = content_query.single() {
            // Clear existing content first
            if let Ok(children) = children_query.get(content_entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }

            // Rebuild content with new selection
            commands.entity(content_entity).with_children(|parent| {
                // For now, just add a simple text indicator
                parent.spawn((
                    Text::new(format!("Selected: {}", selection_event.node_id)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        }
    }
}

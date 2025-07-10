//! Tree view UI components for displaying hierarchical data.
//!
//! This module provides a comprehensive tree view system for displaying hierarchical
//! entity data in the inspector. It includes selection handling, expansion states,
//! and visual styling for different node types.
//!
//! # Related Documentation
//!
//! - [Bevy UI Guide](https://docs.rs/bevy/latest/bevy/ui/index.html) - Core UI system documentation
//! - [`TreeNode`] - Individual tree node component with metadata
//! - [`TreeState`] - Global tree state resource for tracking nodes and selection
//! - [`crate::TreeNodeType`] - Node type enum for visual styling

use bevy::{
    core_widgets::{ControlOrientation, CoreScrollbar, CoreScrollbarPlugin, CoreScrollbarThumb},
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    picking::hover::Hovered,
    prelude::*,
};
use std::collections::HashMap;

/// A node in the tree structure representing entities, crate groups, components, or fields.
///
/// This component stores the complete hierarchical information for displaying inspector data
/// in a tree format. Each node represents a different level of the entity hierarchy:
/// - **Entity nodes**: Root level, showing entity names or IDs
/// - **Crate group nodes**: Group components by their crate (e.g., "bevy_transform")
/// - **Component nodes**: Individual components within a crate group
/// - **Field nodes**: Individual fields/properties within a component
///
/// # Field Population
///
/// Values are populated by [`crate::tree_builder::InspectorTreeBuilder`] when processing
/// [`crate::events::EntityInspectorRows`] data:
/// - `id`: Generated hierarchically (e.g., "entity_42", "entity_42_bevy_transform", "entity_42_bevy_transform_Transform")
/// - `label`: Human-readable display text extracted from component names, field names, or entity names
/// - `is_expanded`: Defaults to `false`, controlled by user interaction
/// - `children`: Populated automatically based on reflection data structure
/// - `parent`: Set during tree construction to maintain hierarchy
/// - `depth`: Calculated based on position in hierarchy (entity=0, crate=1, component=2, field=3)
/// - `node_type`: Determined by the data type being represented
///
/// # Label vs ID Difference
///
/// - **`id`**: Technical identifier for tree management, unique across entire tree
///   - Example: `"entity_42_bevy_transform_Transform_translation"`
/// - **`label`**: User-friendly display text shown in UI
///   - Example: `"translation: Vec3(0.0, 1.0, 0.0)"`
///
/// The `id` is used for tree state management, event routing, and parent-child relationships,
/// while `label` is purely for display purposes and may contain formatted values.
#[derive(Component, Clone, Debug)]
pub struct TreeNode {
    /// Unique hierarchical identifier for tree state management.
    ///
    /// Format: `"entity_{id}_{crate}_{component}_{field}"` where each level
    /// is appended as you go deeper in the hierarchy. Used internally for
    /// tracking expansion state, selection, and parent-child relationships.
    pub id: String,

    /// Human-readable display text shown in the UI.
    ///
    /// Contains the formatted display name that users see, such as:
    /// - Entity: `"Player (42)"` or `"Entity 42"`
    /// - Crate group: `"bevy_transform"`
    /// - Component: `"Transform"`
    /// - Field: `"translation: Vec3(0.0, 1.0, 0.0)"`
    ///
    /// This differs from `id` by being display-focused and may include
    /// formatted values, spaces, and special characters.
    pub label: String,

    /// Whether this node is currently expanded to show its children.
    ///
    /// Controlled by user clicks on disclosure triangles. When `true`,
    /// child nodes are visible in the tree. When `false`, children are
    /// hidden. Automatically set to `false` for leaf nodes (fields).
    pub is_expanded: bool,

    /// List of child node IDs in display order.
    ///
    /// Populated automatically during tree construction based on:
    /// - Entity children: Crate groups containing that entity's components
    /// - Crate children: Components belonging to that crate
    /// - Component children: Fields extracted via [`crate::reflection::extract_reflect_fields`]
    /// - Field children: None (fields are leaf nodes)
    ///
    /// Order is maintained for consistent display (crate alphabetical, then component alphabetical).
    pub children: Vec<String>,

    /// ID of the parent node in the hierarchy.
    ///
    /// `None` for root-level entity nodes. For all other nodes, contains
    /// the ID of their immediate parent. Used for tree navigation and
    /// maintaining hierarchical relationships during UI updates.
    pub parent: Option<String>,

    /// Nesting depth in the tree hierarchy (0-based).
    ///
    /// Determines visual indentation and helps with styling:
    /// - `0`: Entity nodes (root level)
    /// - `1`: Crate group nodes
    /// - `2`: Component nodes  
    /// - `3`: Field nodes (leaf level)
    ///
    /// Used to calculate left padding: `depth * indent_size`.
    pub depth: usize,

    /// Classification for visual styling and behavior.
    ///
    /// Determines text color, opacity, and other visual properties.
    /// See [`crate::TreeNodeType`] for details on each type's appearance.
    pub node_type: crate::TreeNodeType,
}

/// Global state for the tree view, managing all nodes and selection.
///
/// This resource serves as the central data store for the entire tree UI,
/// maintaining both the hierarchical structure and interactive state.
/// It's updated by [`crate::tree_builder::InspectorTreeBuilder`] when
/// entity data changes, and by interaction systems when users click nodes.
///
/// # State Management
///
/// The tree state is rebuilt when:
/// - New [`crate::events::InspectorEvent`]s indicate entity/component changes
/// - User interactions trigger expansion/collapse via [`handle_tree_node_interactions`]
/// - Full refresh is requested
///
/// Selection and expansion states are preserved across rebuilds to maintain
/// user context and avoid UI flickering.
#[derive(Resource, Default, Clone)]
pub struct TreeState {
    /// Hierarchical map of all tree nodes by their unique ID.
    ///
    /// Contains every node in the tree, from root entities down to individual
    /// component fields. The key is the node's `id` field, allowing fast
    /// lookups during rendering and interaction handling.
    pub nodes: HashMap<String, TreeNode>,

    /// Ordered list of top-level entity node IDs.
    ///
    /// These are the entities that appear at the root level of the tree.
    /// Order determines display sequence and is typically sorted for
    /// consistent presentation (entities with components first, then alphabetical).
    pub root_nodes: Vec<String>,

    /// ID of the currently selected node for property panel display.
    ///
    /// When a user clicks any tree node, this is updated to that node's ID,
    /// triggering [`TreeNodeSelected`] events that update the property panel.
    /// `None` indicates no selection (property panel shows placeholder content).
    pub selected_node: Option<String>,
}

/// Configuration parameters for tree node creation and styling.
///
/// Used when programmatically creating tree nodes to control their
/// interactive behavior and visual state. This is separate from the
/// persistent state stored in [`TreeNode`] itself.
#[derive(Default)]
pub struct TreeNodeProps {
    /// Whether this node should respond to selection clicks.
    ///
    /// When `true`, clicking the node will update [`TreeState::selected_node`]
    /// and emit [`TreeNodeSelected`] events. All nodes are typically selectable
    /// in the inspector to allow property panel updates.
    pub selectable: bool,

    /// Whether this node should be visually highlighted as selected.
    ///
    /// Controls the initial background color when creating the node.
    /// This is usually determined by comparing the node's ID with
    /// [`TreeState::selected_node`] during UI construction.
    pub selected: bool,
}

/// Component marker for interactive tree node UI elements.
///
/// This component is attached to the clickable [`Button`] entity that represents
/// a tree node in the UI. It stores the node's ID to enable event routing and
/// state synchronization between the UI element and the logical tree data.
///
/// # Purpose
///
/// - **Event Routing**: When a tree node is clicked, this component helps identify
///   which logical tree node was selected
/// - **State Sync**: Links UI interaction state with tree data state
/// - **Selection Management**: Enables updating both visual styling and logical selection
///
/// Used by [`handle_tree_node_interactions`] to process click events and
/// [`update_tree_node_style`] to apply selection/hover styling.
#[derive(Component)]
pub struct TreeNodeWidget {
    /// Reference to the logical tree node's unique identifier.
    ///
    /// This must match a key in [`TreeState::nodes`] to enable proper
    /// event handling and state synchronization. When this UI element
    /// is clicked, the corresponding [`TreeNode`] will be found using this ID.
    pub node_id: String,
}

/// Component marker for tree node text display elements.
///
/// Attached to the [`Text`] entity that shows the node's label text.
/// This separation allows for independent styling and interaction handling
/// between the clickable button area and the text content.
///
/// # Purpose
///
/// - **Text Styling**: Enables targeted styling of text elements separate from button styling
/// - **Content Updates**: Allows updating text content without affecting button interaction
/// - **Accessibility**: Maintains semantic separation between interactive and display elements
#[derive(Component)]
pub struct TreeLabel {
    /// Reference to the tree node this label represents.
    ///
    /// Links the text element back to its logical tree node for
    /// potential text updates or styling based on node state.
    pub node_id: String,
}

/// Component marker for the scrollable tree container.
///
/// This marks the main scrollable area that contains all tree nodes.
/// It's separate from the outer frame that contains scrollbars, allowing
/// for proper scroll behavior and content management.
///
/// # Purpose
///
/// - **Content Management**: Identifies where tree nodes should be spawned
/// - **Scroll Behavior**: Enables proper scrolling when content overflows
/// - **Layout Isolation**: Separates tree content from scrollbar UI elements
///
/// Used by [`handle_tree_expansion_changes`] to find where to spawn new
/// tree nodes when the tree structure is rebuilt.
#[derive(Component)]
pub struct TreeContainer;

/// Event emitted when a tree node is selected by user interaction.
///
/// This event is fired whenever a user clicks on any tree node, regardless
/// of whether it's an entity, crate group, component, or field. The selection
/// triggers updates to both the tree visual state and the property panel content.
///
/// # Event Flow
///
/// 1. User clicks a tree node button
/// 2. [`handle_tree_node_interactions`] detects the click
/// 3. [`TreeState::selected_node`] is updated
/// 4. This event is emitted with the selected node's ID
/// 5. Property panel systems listen for this event to update their content
///
/// # Property Panel Integration
///
/// The property panel uses this event to determine what component data to display:
/// - **Entity selection**: Shows all components for that entity
/// - **Component selection**: Shows detailed fields for that specific component
/// - **Field selection**: Shows the parent component with the selected field highlighted
#[derive(Event, BufferedEvent)]
pub struct TreeNodeSelected {
    /// ID of the tree node that was selected.
    ///
    /// This corresponds to a key in [`TreeState::nodes`] and can be used
    /// to retrieve the full node data including its type, depth, and content.
    pub node_id: String,
}

/// Event fired when a tree node's expansion state changes.
///
/// Currently not actively used but provides infrastructure for future
/// expansion-specific functionality like lazy loading, animation, or
/// state persistence across sessions.
///
/// # Future Use Cases
///
/// - **Lazy Loading**: Load component data only when nodes are expanded
/// - **Animations**: Trigger expand/collapse animations
/// - **State Persistence**: Remember expansion states across app restarts
/// - **Performance**: Defer expensive reflection operations until needed
#[derive(Event, BufferedEvent)]
pub struct TreeNodeExpanded {
    /// ID of the node whose expansion state changed.
    pub node_id: String,
    /// New expansion state after the change.
    pub is_expanded: bool,
}

/// Visual and layout configuration for the tree view appearance.
///
/// This resource controls all aspects of tree rendering including spacing,
/// colors, and sizing. It can be modified at runtime to change the tree's
/// appearance or replaced entirely to switch between light/dark themes.
///
/// # Theming Integration
///
/// While this provides tree-specific configuration, it works alongside
/// [`crate::theme::InspectorTheme`] for overall inspector theming.
/// Consider using [`crate::theme::InspectorTheme`] for broader styling
/// consistency across all inspector components.
///
/// # Visual Hierarchy
///
/// The configuration supports visual hierarchy through:
/// - **Indentation**: `indent_size` creates nested structure
/// - **Colors**: Different node types get different `text_color` variations
/// - **Interactive Feedback**: `selected_color` and `hover_color` for user feedback
#[derive(Resource)]
pub struct TreeConfig {
    /// Horizontal indentation per tree depth level in pixels.
    ///
    /// Each level deeper in the hierarchy adds this amount of left padding.
    /// For example, with `indent_size: 20.0`:
    /// - Entities (depth 0): 0px padding
    /// - Crate groups (depth 1): 20px padding  
    /// - Components (depth 2): 40px padding
    /// - Fields (depth 3): 60px padding
    pub indent_size: f32,

    /// Height of each tree node row in pixels.
    ///
    /// Consistent height for all tree nodes regardless of content.
    /// Affects vertical spacing and click target size. Should be
    /// large enough to accommodate text and provide comfortable clicking.
    pub node_height: f32,

    /// Size of disclosure triangle indicators in pixels.
    ///
    /// Controls both the font size of the triangle characters (▶/▼)
    /// and the size of their container area. Affects visual prominence
    /// of expandable vs non-expandable nodes.
    pub triangle_size: f32,

    /// Font size for tree node text labels.
    ///
    /// Applied to all tree text content. Should balance readability
    /// with information density, especially for deeply nested structures.
    pub font_size: f32,

    /// Default text color for tree node labels.
    ///
    /// Individual node types may override this color in the rendering
    /// logic for visual hierarchy (entities, crate groups, components, fields).
    pub text_color: Color,

    /// Background color for the currently selected tree node.
    ///
    /// Provides clear visual feedback about which node is selected
    /// and will be displayed in the property panel. Should contrast
    /// well with `text_color` for accessibility.
    pub selected_color: Color,

    /// Background color for tree nodes under mouse hover.
    ///
    /// Provides interactive feedback before clicking. Should be
    /// subtle enough not to distract but visible enough to indicate
    /// interactivity. Often a lighter version of `selected_color`.
    pub hover_color: Color,

    /// Background color for the entire tree container.
    ///
    /// Sets the overall tree panel background. Should provide good
    /// contrast with node colors and fit the overall inspector theme.
    pub background_color: Color,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            indent_size: 20.0,
            node_height: 24.0,
            triangle_size: 12.0,
            font_size: 14.0,
            text_color: Color::srgb(0.9, 0.9, 0.9),
            selected_color: Color::srgb(0.2, 0.4, 0.8),
            hover_color: Color::srgb(0.15, 0.15, 0.15),
            background_color: Color::srgb(0.1, 0.1, 0.1),
        }
    }
}

/// Theme tokens for tree components
pub mod tree_tokens {
    /// Tree node styles
    pub const TREE_NODE_TEXT: &str = "tree_node_text";
    /// Tree node selected style
    pub const TREE_NODE_SELECTED: &str = "tree_node_selected";
    /// Tree node hover style
    pub const TREE_NODE_HOVER: &str = "tree_node_hover";
    /// Tree background style
    pub const TREE_BACKGROUND: &str = "tree_background";
    /// Tree border style
    pub const TREE_BORDER: &str = "tree_border";
}

/// Creates a tree node widget bundle with interaction and styling components.
///
/// This function constructs the UI bundle for a single tree node, including
/// the layout, interaction area, and visual styling. The resulting bundle
/// can be spawned as an entity to create an interactive tree node.
///
/// # Parameters
///
/// - `node`: The logical tree node data containing ID, label, hierarchy info
/// - `props`: Display properties controlling selection state and interactivity
/// - `config`: Visual configuration for sizing, colors, and spacing
///
/// # Returns
///
/// A bundle containing [`Node`], [`TreeNodeWidget`], [`BackgroundColor`], and [`BorderRadius`]
/// components that together create a styled, interactive tree node UI element.
///
/// # Usage
///
/// Typically called by [`build_tree_node_recursive`] during tree construction,
/// but can also be used for custom tree node creation.
pub fn tree_node(node: &TreeNode, props: TreeNodeProps, config: &TreeConfig) -> impl Bundle {
    let node_id = node.id.clone();
    let _has_children = !node.children.is_empty();

    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(config.node_height),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::left(Val::Px(node.depth as f32 * config.indent_size)),
            ..default()
        },
        TreeNodeWidget {
            node_id: node_id.clone(),
        },
        BackgroundColor(if props.selected {
            config.selected_color
        } else {
            Color::NONE
        }),
        BorderRadius::all(Val::Px(2.0)),
    )
}

/// Creates a scrollable tree container with visible scrollbars and grid layout.
///
/// This function builds the complete tree container infrastructure including:
/// - A grid-based frame that positions the scroll area and scrollbars
/// - A scrollable content area marked with [`TreeContainer`] for tree nodes
/// - Vertical and horizontal scrollbars with interactive thumbs
/// - Proper spacing and visual styling
///
/// # Architecture
///
/// The container uses a CSS Grid layout with:
/// - Row 1, Column 1: Main scrollable tree content area
/// - Row 1, Column 2: Vertical scrollbar
/// - Row 2, Column 1: Horizontal scrollbar  
/// - Row 2, Column 2: (Empty corner space)
///
/// # Parameters
///
/// - `config`: Visual configuration for colors and styling
///
/// # Returns
///
/// A complex bundle using [`Children::spawn`] that creates the multi-entity
/// container structure. The actual [`TreeContainer`] marker is on a child entity,
/// not the returned root entity.
///
/// # Usage
///
/// Called once during tree UI setup to create the container where tree nodes
/// will be spawned by [`handle_tree_expansion_changes`].
pub fn tree_container(config: &TreeConfig) -> impl Bundle {
    let background_color = config.background_color;

    (
        // Frame element which contains the scroll area and scrollbars.
        Node {
            display: Display::Grid,
            width: Val::Auto,
            min_width: Val::Percent(100.0),
            height: Val::Auto,
            min_height: Val::Percent(100.0),
            grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            row_gap: Val::Px(2.0),
            column_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(background_color),
        BorderRadius::all(Val::Px(4.0)),
        Children::spawn((SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
            // The actual scrolling area.
            let scroll_area_id = parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        overflow: Overflow::scroll(),
                        ..default()
                    },
                    TreeContainer,
                    BackgroundColor(background_color),
                    ScrollPosition(Vec2::ZERO),
                ))
                .id();

            // Vertical scrollbar
            parent.spawn((
                Node {
                    min_width: Val::Px(16.0),
                    grid_row: GridPlacement::start(1),
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                CoreScrollbar {
                    target: scroll_area_id,
                    orientation: ControlOrientation::Vertical,
                    min_thumb_length: 20.0,
                },
                Children::spawn(Spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    Hovered::default(),
                    BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    BorderRadius::all(Val::Px(4.0)),
                    CoreScrollbarThumb,
                ))),
            ));

            // Horizontal scrollbar (bottom row)
            parent.spawn((
                Node {
                    min_height: Val::Px(16.0),
                    grid_row: GridPlacement::start(2),
                    grid_column: GridPlacement::start(1),
                    ..default()
                },
                CoreScrollbar {
                    target: scroll_area_id,
                    orientation: ControlOrientation::Horizontal,
                    min_thumb_length: 20.0,
                },
                Children::spawn(Spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    Hovered::default(),
                    BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    BorderRadius::all(Val::Px(4.0)),
                    CoreScrollbarThumb,
                ))),
            ));
        }),)),
    )
}

/// System to handle tree node selection styling
pub fn update_tree_node_style(
    mut query: Query<
        (&mut BackgroundColor, &TreeNodeWidget, &Interaction),
        (Changed<Interaction>, With<TreeNodeWidget>),
    >,
    tree_state: Option<Res<TreeState>>,
    config: Res<TreeConfig>,
) {
    if let Some(state) = tree_state {
        for (mut bg_color, node_widget, interaction) in query.iter_mut() {
            let is_selected = state.selected_node.as_ref() == Some(&node_widget.node_id);

            match *interaction {
                Interaction::Pressed => {
                    bg_color.0 = config.selected_color;
                }
                Interaction::Hovered => {
                    bg_color.0 = config.hover_color;
                }
                Interaction::None => {
                    if is_selected {
                        bg_color.0 = config.selected_color;
                    } else {
                        bg_color.0 = Color::NONE;
                    }
                }
            }
        }
    }
}

/// System to handle tree node clicks and expand/collapse functionality
pub fn handle_tree_node_interactions(
    tree_node_query: Query<(&Interaction, &TreeNodeWidget), (Changed<Interaction>, With<Button>)>,
    mut tree_state: ResMut<TreeState>,
    mut selection_events: EventWriter<TreeNodeSelected>,
) {
    for (interaction, node_widget) in tree_node_query.iter() {
        if *interaction == Interaction::Pressed {
            let node_id = &node_widget.node_id;

            if let Some(node) = tree_state.nodes.get_mut(node_id) {
                // Toggle expansion state if the node has children
                if !node.children.is_empty() {
                    node.is_expanded = !node.is_expanded;
                    info!(
                        "Toggled node {} expansion to: {}",
                        node_id, node.is_expanded
                    );
                }

                // Always update selected node for any clickable node
                tree_state.selected_node = Some(node_id.clone());
                info!("Selected node: {}", node_id);

                // Emit selection event for property panel updates
                selection_events.write(TreeNodeSelected {
                    node_id: node_id.clone(),
                });
            }
        }
    }
}

/// System to handle tree expansion changes - simplified observer pattern
pub fn handle_tree_expansion_changes(
    tree_state: ResMut<TreeState>,
    tree_container_query: Query<Entity, With<TreeContainer>>,
    children_query: Query<&Children>,
    tree_config: Res<TreeConfig>,
    mut commands: Commands,
) {
    // Only rebuild UI when the tree state has actually changed (user interaction)
    if tree_state.is_changed() && !tree_state.is_added() {
        info!("Tree state changed (user interaction), rebuilding UI");

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
                    let node_entity = build_tree_node_recursive(
                        &mut commands,
                        root_node,
                        &tree_state,
                        &tree_config,
                    );
                    commands.entity(container_entity).add_child(node_entity);
                }
            }
        }
    }
}

/// Builds a tree view UI from the given state and configuration
pub fn build_tree_view(
    commands: &mut Commands,
    _tree_state: &TreeState,
    config: &TreeConfig,
) -> Entity {
    let frame = commands.spawn(tree_container(config)).id();

    // We need to find the actual TreeContainer within the frame structure
    // Since we can't easily access it here, we'll need to modify our approach

    frame
}

/// Recursively builds tree nodes and their children using standard Bevy UI
pub fn build_tree_node_recursive(
    commands: &mut Commands,
    node: &TreeNode,
    tree_state: &TreeState,
    config: &TreeConfig,
) -> Entity {
    let has_children = !node.children.is_empty();

    // Create a container for the entire node (button + children)
    let node_container = commands
        .spawn(Node {
            width: Val::Auto,
            min_width: Val::Percent(100.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    // Create the main clickable node row - make all nodes clickable for selection
    let node_row = commands
        .spawn((
            Button, // All nodes are now buttons for selection
            Node {
                width: Val::Auto,
                min_width: Val::Percent(100.0),
                height: Val::Auto,
                min_height: Val::Px(config.node_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::left(Val::Px(node.depth as f32 * config.indent_size)),
                ..default()
            },
            TreeNodeWidget {
                node_id: node.id.clone(),
            },
            BackgroundColor(if tree_state.selected_node.as_ref() == Some(&node.id) {
                config.selected_color
            } else if has_children {
                Color::NONE
            } else {
                // Slightly different background for non-expandable components
                Color::srgba(0.1, 0.1, 0.1, 0.3)
            }),
            BorderRadius::all(Val::Px(2.0)),
        ))
        .id();

    // Add disclosure triangle for visual indication
    if has_children {
        let triangle_char = if node.is_expanded { "▼" } else { "▶" };
        let triangle = commands
            .spawn((
                Text::new(triangle_char),
                TextFont {
                    font_size: config.triangle_size,
                    ..default()
                },
                TextColor(config.text_color),
                Node {
                    width: Val::Px(config.triangle_size + 8.0),
                    height: Val::Px(config.triangle_size + 8.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::right(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(node_row).add_child(triangle);
    } else {
        // Add a subtle dot indicator for non-expandable items
        let dot_indicator = commands
            .spawn((
                Text::new("•"),
                TextFont {
                    font_size: config.triangle_size * 0.6,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
                Node {
                    width: Val::Px(config.triangle_size + 8.0),
                    height: Val::Px(config.triangle_size + 8.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::right(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(node_row).add_child(dot_indicator);
    }

    // Add label with flexible container and styling based on node type
    let label_container = commands
        .spawn(Node {
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    // Determine styling based on node type and expandability
    let (text_color, _font_weight, prefix) = match node.node_type {
        crate::TreeNodeType::Entity => (config.text_color, None::<()>, ""),
        crate::TreeNodeType::CrateGroup => (Color::srgb(0.8, 0.9, 1.0), None::<()>, ""),
        crate::TreeNodeType::Component => {
            // Reduce alpha for components without children (non-expandable)
            if has_children {
                (Color::srgb(0.9, 0.8, 0.7), None::<()>, "")
            } else {
                (Color::srgba(0.9, 0.8, 0.7, 0.5), None::<()>, "")
            }
        }
        crate::TreeNodeType::Field => (Color::srgb(0.7, 0.7, 0.7), None::<()>, ""),
    };

    let label_text = commands
        .spawn((
            Text::new(format!("{}{}", prefix, node.label)),
            TextFont {
                font_size: config.font_size,
                ..default()
            },
            TextColor(text_color),
            TreeLabel {
                node_id: node.id.clone(),
            },
        ))
        .id();

    commands.entity(label_container).add_child(label_text);
    commands.entity(node_row).add_child(label_container);

    // Add the row to the container
    commands.entity(node_container).add_child(node_row);

    // Add children below the row if expanded
    if node.is_expanded {
        // Sort child nodes so that nodes with children appear first
        let mut sorted_child_ids = node.children.clone();
        sorted_child_ids.sort_by(|a, b| {
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

        for child_id in &sorted_child_ids {
            if let Some(child_node) = tree_state.nodes.get(child_id) {
                let child_entity =
                    build_tree_node_recursive(commands, child_node, tree_state, config);
                commands.entity(node_container).add_child(child_entity);
            }
        }
    }

    node_container
}

/// Plugin for tree view functionality
pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TreeNodeSelected>()
            .add_event::<TreeNodeExpanded>()
            .init_resource::<TreeConfig>()
            .init_resource::<TreeState>()
            .add_plugins(CoreScrollbarPlugin)
            .add_systems(
                Update,
                (
                    update_tree_node_style,
                    handle_tree_node_interactions,
                    handle_tree_expansion_changes,
                    update_scrollbar_thumb_style,
                ),
            );
    }
}

/// Update the color of the scrollbar thumb
fn update_scrollbar_thumb_style(
    mut query: Query<
        (&mut BackgroundColor, &Hovered),
        (With<CoreScrollbarThumb>, Changed<Hovered>),
    >,
) {
    for (mut thumb_bg, hovered) in query.iter_mut() {
        let color = if hovered.0 {
            Color::srgb(0.7, 0.7, 0.7) // Lighter color when hovering
        } else {
            Color::srgb(0.5, 0.5, 0.5) // Default color
        };

        thumb_bg.0 = color;
    }
}

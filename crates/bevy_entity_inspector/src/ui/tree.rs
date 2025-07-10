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

/// A node in the tree structure
#[derive(Component, Clone, Debug)]
pub struct TreeNode {
    /// Unique identifier for the node
    pub id: String,
    /// Display label for the node
    pub label: String,
    /// Whether the node is expanded (showing children)
    pub is_expanded: bool,
    /// IDs of child nodes
    pub children: Vec<String>,
    /// ID of parent node, if any
    pub parent: Option<String>,
    /// Depth level in the tree (0 for root nodes)
    pub depth: usize,
    /// Type of node for visual styling
    pub node_type: crate::TreeNodeType,
}

/// Global state for the tree view
#[derive(Resource, Default, Clone)]
pub struct TreeState {
    /// Map of node ID to node data
    pub nodes: HashMap<String, TreeNode>,
    /// IDs of root-level nodes
    pub root_nodes: Vec<String>,
    /// Currently selected node ID
    pub selected_node: Option<String>,
}

/// Properties for tree nodes, used to control selection and interaction
#[derive(Default)]
pub struct TreeNodeProps {
    /// Whether the node is selectable
    pub selectable: bool,
    /// Whether the node is currently selected
    pub selected: bool,
}

/// Component markers for tree UI elements
#[derive(Component)]
pub struct TreeNodeWidget {
    /// Unique identifier for the tree node
    pub node_id: String,
}

/// Component for tree node labels, used for text display
#[derive(Component)]
pub struct TreeLabel {
    /// Unique identifier for the node this label belongs to
    pub node_id: String,
}

/// Container for the tree view, used to apply styles and scrolling
#[derive(Component)]
pub struct TreeContainer;

/// Tree-related events
#[derive(Event, BufferedEvent)]
pub struct TreeNodeSelected {
    /// ID of the node that was selected
    pub node_id: String,
}

/// Event fired when a tree node is expanded or collapsed
#[derive(Event, BufferedEvent)]
pub struct TreeNodeExpanded {
    /// ID of the node that was expanded or collapsed
    pub node_id: String,
    /// Whether the node is now expanded
    pub is_expanded: bool,
}

/// Configuration for tree appearance
#[derive(Resource)]
pub struct TreeConfig {
    /// Size of indentation for child nodes
    pub indent_size: f32,
    /// Height of each tree node
    pub node_height: f32,
    /// Size of disclosure triangle
    pub triangle_size: f32,
    /// Font size for node text
    pub font_size: f32,
    /// Color of node text
    pub text_color: Color,
    /// Color of selected node background
    pub selected_color: Color,
    /// Color of hovered node background
    pub hover_color: Color,
    /// Background color of the tree container
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

/// Creates a tree node widget
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

/// Creates a tree container widget with scrolling support and visible scrollbars
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

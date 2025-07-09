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
//! │   └── ComputedVisibility
//! └── my_game
//!     └── Player
//!         ├── health: 100
//!         └── score: 1500
//! ```

use bevy::prelude::*;

// Core modules
pub mod events;
pub mod reflection;
pub mod tree_builder;
pub mod ui_systems;

// UI and utility modules
pub mod theme;
pub mod ui;
pub mod widgets;

// Optional remote module
#[cfg(feature = "remote")]
pub mod remote;

// Re-export commonly used types
pub use events::{EntityInspectorRow, EntityInspectorRows, InspectorEvent, InspectorNodeData};
pub use reflection::{extract_crate_and_type, extract_reflect_fields};
pub use theme::{create_dark_inspector_theme, create_light_inspector_theme, InspectorTheme};
pub use tree_builder::{InspectorTreeBuilder, InspectorTreeNode, TreeNodeType};
pub use ui::{
    build_tree_node_recursive, build_tree_view, tree_container, TreeConfig, TreeContainer,
    TreeNode, TreePlugin, TreeState,
};
pub use ui_systems::handle_inspector_events;
pub use widgets::{
    create_inspector_field, create_inspector_panel, InspectorField, InspectorFieldType,
    InspectorPanel, InspectorPanelProps,
};

use crate::ui_systems::{handle_tree_selection, setup_inspector_camera, spawn_inspector_ui_once};

/// Main plugin for the Bevy Entity Inspector.
///
/// This plugin provides a complete entity inspection system with a tree-based UI
/// that displays entities and their components in a hierarchical view. Components
/// are automatically grouped by crate name for better organization.
///
/// # Features
///
/// - **Event-Driven Architecture**: Efficient updates using `InspectorEvent` system
/// - **Component Grouping**: Automatic grouping by crate (e.g., "bevy_transform", "my_game")
/// - **Remote Inspection**: Optional remote inspection via `bevy_remote` (with "remote" feature)
/// - **Reflection Support**: Automatic component introspection using Bevy's reflection system
/// - **Modern UI**: Clean, themeable interface with expand/collapse functionality
/// - **Performance Optimized**: Only updates when actual changes occur
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
            .add_systems(Update, (handle_inspector_events, handle_tree_selection));

        #[cfg(feature = "remote")]
        {
            use crate::remote::EntityInspectorRemotePlugin;
            app.add_plugins(EntityInspectorRemotePlugin::default());
        }
    }
}

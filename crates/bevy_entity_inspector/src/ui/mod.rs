//! UI components and widgets for the entity inspector.
//!
//! This module organizes all UI-related functionality for the inspector,
//! including tree views, property panels, and interactive widgets.
//!
//! # Modules
//!
//! - [`disclosure`] - Expandable/collapsible disclosure triangles
//! - [`property_panel`] - Right-side property display panel
//! - [`tree`] - Main tree view for entity hierarchy
//!
//! # Related Documentation
//!
//! - [Bevy UI Guide](https://docs.rs/bevy/latest/bevy/ui/index.html) - Core UI system documentation
//! - [`InspectorWidgetsPlugin`] - Plugin that enables all UI widgets

use bevy::prelude::*;

/// Disclosure triangle UI components
pub mod disclosure;
/// Property panel for detailed component inspection
pub mod property_panel;
/// Tree view UI components  
pub mod tree;

pub use disclosure::*;
pub use property_panel::*;
pub use tree::*;

/// Plugin that provides all inspector UI widgets
pub struct InspectorWidgetsPlugin;

impl Plugin for InspectorWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DisclosurePlugin, TreePlugin));
    }
}

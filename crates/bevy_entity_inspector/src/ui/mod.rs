//! UI components and widgets for the entity inspector

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

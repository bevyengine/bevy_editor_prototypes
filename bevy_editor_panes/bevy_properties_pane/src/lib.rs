//! 3D Viewport for Bevy
use bevy::prelude::*;
use bevy_entity_inspector::{EntityInspector, EntityInspectorPlugin};
use bevy_pane_layout::{prelude::{PaneAppExt, PaneStructure}, PaneContentNode};

pub use bevy_entity_inspector::InspectedEntity;

/// The identifier for the 3D Viewport.
/// This is present on any pane that is a 3D Viewport.
#[derive(Component)]
pub struct BevyPropertiesPane;

impl Default for BevyPropertiesPane {
    fn default() -> Self {
        BevyPropertiesPane
    }
}

/// Plugin for the 3D Viewport pane.
pub struct PropertiesPanePlugin;

impl Plugin for PropertiesPanePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntityInspectorPlugin);

        app.register_pane("Properties", on_pane_creation);
    }
}

fn on_pane_creation(
    structure: In<PaneStructure>,
    mut commands: Commands
) {
    println!("Properties pane created");

    let content_node = structure.content;

    commands.entity(content_node).with_child((
        EntityInspector,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        }
    ));
}

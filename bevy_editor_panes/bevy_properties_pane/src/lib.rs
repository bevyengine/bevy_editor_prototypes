//! 3D Viewport for Bevy
use bevy::prelude::*;
use bevy_entity_inspector::{EntityInspector, EntityInspectorPlugin};
use bevy_pane_layout::{PaneContentNode, PaneRegistry};

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

        app.add_observer(on_pane_creation);

        app.world_mut()
            .get_resource_or_init::<PaneRegistry>()
            .register("Properties", |mut commands, pane_root| {
                commands.entity(pane_root).insert(BevyPropertiesPane::default());
            });
    }
}

fn on_pane_creation(
    trigger: Trigger<OnAdd, BevyPropertiesPane>,
    mut commands: Commands,
    children_query: Query<&Children>,
    content: Query<&PaneContentNode>
) {
    info!("Properties pane created");

    let pane_root = trigger.entity();
    let content_node = children_query
        .iter_descendants(pane_root)
        .find(|e| content.contains(*e))
        .unwrap();

    commands.entity(content_node).with_child((
        EntityInspector,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        }
    ));
}

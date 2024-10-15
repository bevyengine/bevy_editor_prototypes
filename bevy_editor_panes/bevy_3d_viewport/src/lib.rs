//! 3D Viewport for Bevy

use bevy::prelude::*;
use bevy_pane_layout::PaneRegistry;

/// The identifier for the 3D Viewport.
/// This is present on any pane that is a 3D Viewport.
#[derive(Component)]
pub struct Bevy3DViewport;

/// Plugin for the 3D Viewport pane.
pub struct Viewport3dPanePlugin;

impl Plugin for Viewport3dPanePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .get_resource_or_init::<PaneRegistry>()
            .register("Viewport 3D", |mut commands, pane_root| {
                commands.entity(pane_root).insert(Bevy3DViewport);
            });
    }
}

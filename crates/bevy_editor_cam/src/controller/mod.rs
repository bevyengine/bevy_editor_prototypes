//! Camera controller implementation.

use bevy::app::prelude::*;
use bevy::ecs::prelude::*;

pub mod component;
pub mod inputs;
pub mod momentum;
pub mod motion;
pub mod projections;
pub mod smoothing;
pub mod zoom;

/// Adds [`bevy_editor_cam`](crate) functionality without an input plugin or any extensions. This
/// requires an input plugin to function! If you don't use the [`crate::input::DefaultInputPlugin`],
/// you will need to provide your own.
pub struct MinimalEditorCamPlugin;

impl Plugin for MinimalEditorCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                component::EditorCam::update_camera_positions,
                projections::update_orthographic,
                projections::update_perspective,
            )
                .chain()
                .after(bevy::picking::PickingSystems::Last),
        )
        .register_type::<component::EditorCam>();
    }
}

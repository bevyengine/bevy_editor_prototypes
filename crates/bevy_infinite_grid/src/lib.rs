#![allow(missing_docs)]

mod render;

use bevy::camera::visibility::{
    NoFrustumCulling, VisibilityClass, VisibleEntities, add_visibility_class,
};
use bevy::{prelude::*, render::sync_world::SyncToRenderWorld};

pub struct InfiniteGridPlugin;

impl Plugin for InfiniteGridPlugin {
    fn build(&self, _: &mut App) {}

    fn finish(&self, app: &mut App) {
        render::render_app_builder(app);
    }
}

#[derive(Component, Default)]
#[require(
    InfiniteGridSettings,
    Transform,
    Visibility,
    VisibleEntities,
    NoFrustumCulling,
    SyncToRenderWorld
)]
pub struct InfiniteGrid;

#[derive(Component, Copy, Clone)]
#[require(VisibilityClass)]
#[component(on_add = add_visibility_class::<InfiniteGridSettings>)]
pub struct InfiniteGridSettings {
    pub x_axis_color: Color,
    pub z_axis_color: Color,
    pub minor_line_color: Color,
    pub major_line_color: Color,
    pub fadeout_distance: f32,
    pub dot_fadeout_strength: f32,
    pub scale: f32,
}

impl Default for InfiniteGridSettings {
    fn default() -> Self {
        Self {
            x_axis_color: Color::oklch(0.65, 0.24, 27.0),
            z_axis_color: Color::oklch(0.65, 0.19, 255.0),
            minor_line_color: Color::srgb(0.1, 0.1, 0.1),
            major_line_color: Color::srgb(0.25, 0.25, 0.25),
            fadeout_distance: 100.,
            dot_fadeout_strength: 0.25,
            scale: 1.,
        }
    }
}

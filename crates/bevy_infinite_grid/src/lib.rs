mod render;

use bevy::prelude::*;
use bevy::render::view::{check_visibility, NoFrustumCulling, VisibleEntities};

pub struct InfiniteGridPlugin;

impl Plugin for InfiniteGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, check_visibility::<With<InfiniteGridSettings>>);
    }

    fn finish(&self, app: &mut App) {
        render::render_app_builder(app);
    }
}

#[derive(Component, Default)]
pub struct InfiniteGrid;

#[derive(Component, Copy, Clone)]
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
            x_axis_color: Color::srgb(1.0, 0.2, 0.2),
            z_axis_color: Color::srgb(0.2, 0.2, 1.0),
            minor_line_color: Color::srgb(0.1, 0.1, 0.1),
            major_line_color: Color::srgb(0.25, 0.25, 0.25),
            fadeout_distance: 100.,
            dot_fadeout_strength: 0.25,
            scale: 1.,
        }
    }
}

#[derive(Bundle, Default)]
pub struct InfiniteGridBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub settings: InfiniteGridSettings,
    pub grid: InfiniteGrid,
    pub visibility: Visibility,
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
    pub shadow_casters: VisibleEntities,
    pub no_frustum_culling: NoFrustumCulling,
}

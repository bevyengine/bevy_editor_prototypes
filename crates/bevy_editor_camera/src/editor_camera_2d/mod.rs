//! A 2d editor camera controller.
//!
//! This module provides a 2d editor camera controller which can be used to pan and zoom the
//! camera.

// Heavily inspired by https://github.com/johanhelsing/bevy_pancam/blob/main/src/lib.rs#L279

use std::ops::RangeInclusive;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::bounding::{Aabb2d, BoundingVolume},
    prelude::*,
    render::camera::{CameraProjection, ScalingMode},
    window::PrimaryWindow,
};

/// Plugin which adds necessary components and systems for 2d editor cameras to work.
pub struct EditorCamera2dPlugin;

/// System set to allow ordering of the [`EditorCamera2dPlugin`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct EditorCamera2dSet;

impl Plugin for EditorCamera2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_zoom, camera_pan).in_set(EditorCamera2dSet));
    }
}

/// Component which represents a 2d editor camera.
///
/// This will provide panning and zooming functionality to an orthographic camera.
#[derive(Component)]
pub struct EditorCamera2d {
    /// Whether the camera will respond to input.
    pub enabled: bool,
    /// Mouse buttons used for panning.
    ///
    /// When one of these mouse buttons is pressed the camera will be panned.
    pub pan_mouse_buttons: Vec<MouseButton>,
    /// The bound which the camera will be clamped to when panning
    /// and zooming. Use infinity values to disable any clamping.
    pub bound: Rect,
    /// The minimum allowed zoom of the camera.
    ///
    /// The orthographic projection's scale will be clamped
    /// at this value when zooming in. Use [`f32::NEG_INFINITY`]
    /// to disable clamping.
    // TODO: Figure out if negative infinity or 0.0 makes more sense since
    pub min_scale: f32,
    /// The maximum allowed zoom of the camera.
    ///
    /// The orthographic projection's scale will be clamped
    /// at this value when zooming out. Use [`f32::INFINITY`]
    /// to disable clamping.
    pub max_scale: f32,
    /// The sensitivity of the mouse wheel input when zooming.
    pub zoom_sensitivity: f32,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// The number of pixels the camera should scroll per line.
    ///
    /// This is used to convert [`MouseScrollUnity::Line`] to [`MouseScrollUnity::Pixel`].
    pub scroll_pixels_per_line: f32,
}

impl EditorCamera2d {
    fn aabb(&self) -> Aabb2d {
        Aabb2d {
            min: self.bound.min,
            max: self.bound.max,
        }
    }

    /// Returns the scale inclusive range
    fn scale_range(&self) -> RangeInclusive<f32> {
        self.min_scale..=self.max_scale
    }
}

impl Default for EditorCamera2d {
    fn default() -> Self {
        Self {
            enabled: true,
            pan_mouse_buttons: vec![MouseButton::Right],
            bound: Rect {
                min: Vec2::ONE * f32::NEG_INFINITY,
                max: Vec2::ONE * f32::INFINITY,
            },
            min_scale: f32::NEG_INFINITY,
            max_scale: f32::INFINITY,
            zoom_sensitivity: 0.001,
            zoom_to_cursor: true,
            scroll_pixels_per_line: 100.0,
        }
    }
}

/// Clamps a camera position to a safe zone. "Safe" means that each screen
/// corner is constrained to the corresponding bound corner.
///
/// # Parameters
/// - `pos` - The position if the bounded area.
/// - `bounded_area_size` - The size of the bounded area.
/// - `bound` - The bound the bounded area should stay within.
///
/// # Visual explanation
///
/// This makes sure the bounded area does not go outside the bound.
///
/// ```
/// +-------Bound------------+
/// |  +---Bounded Area--+   |
/// |  |                 |   |
/// |  |        x (pos)  |   |
/// |  |                 |   |
/// |  +-----------------+   |
/// +------------------------+
/// ```
fn clamp_area_to_bound(pos: Vec2, bounded_area_size: Vec2, bound: Aabb2d) -> Vec2 {
    let aabb = bound.shrink(bounded_area_size / 2.0);
    pos.clamp(aabb.min, aabb.max)
}

/// Updates all values of the [`ScalingMode`] using the `update` closure.
///
// TODO: This should probably be upstreamed.
fn update_scaling_mode<F>(scaling_mode: ScalingMode, update: F) -> ScalingMode
where
    F: Fn(f32) -> f32,
{
    match scaling_mode {
        ScalingMode::Fixed { width, height } => ScalingMode::Fixed {
            width: update(width),
            height: update(height),
        },
        ScalingMode::WindowSize(scale) => ScalingMode::WindowSize(update(scale)),
        ScalingMode::AutoMin {
            min_width,
            min_height,
        } => ScalingMode::AutoMin {
            min_width: update(min_width),
            min_height: update(min_height),
        },
        ScalingMode::AutoMax {
            max_width,
            max_height,
        } => ScalingMode::AutoMax {
            max_width: update(max_width),
            max_height: update(max_height),
        },
        ScalingMode::FixedVertical(scale) => ScalingMode::FixedVertical(update(scale)),
        ScalingMode::FixedHorizontal(scale) => ScalingMode::FixedHorizontal(update(scale)),
    }
}

/// Returns the [`ScalingMode`] as a Vec2.
///
/// If a mode provides two values they are put into x and y respectively,
/// otherwise x and y will be filled with the same value.
fn scaling_mode_as_vec2(scaling_mode: &ScalingMode) -> Vec2 {
    match scaling_mode {
        ScalingMode::Fixed { width, height } => Vec2::new(*width, *height),
        ScalingMode::AutoMin {
            min_width,
            min_height,
        } => Vec2::new(*min_width, *min_height),
        ScalingMode::AutoMax {
            max_width,
            max_height,
        } => Vec2::new(*max_width, *max_height),
        ScalingMode::WindowSize(scale)
        | ScalingMode::FixedVertical(scale)
        | ScalingMode::FixedHorizontal(scale) => Vec2::new(*scale, *scale),
    }
}

/// This is used to find the maximum safe zoom out/projection
/// scale when we have been provided with minimum and maximum x boundaries for
/// the camera.
fn max_scale_within_bounds(
    bounded_area_size: Vec2,
    proj: &OrthographicProjection,
    window_size: Vec2, //viewport?
) -> Vec2 {
    let mut proj = proj.clone();
    proj.scaling_mode = update_scaling_mode(proj.scaling_mode, |_| 1.0);
    proj.update(window_size.x, window_size.y);
    let base_world_size = proj.area.size();
    bounded_area_size / base_world_size
}

/// Makes sure that the camera projection scale stays in the provided bounds
/// and range.
fn clamp_scale_to_bound(
    proj: &mut OrthographicProjection,
    bounded_area_size: Vec2,
    scale_range: &RangeInclusive<f32>,
    window_size: Vec2,
) {
    proj.scaling_mode = update_scaling_mode(proj.scaling_mode, |v| {
        v.clamp(*scale_range.start(), *scale_range.end())
    });
    // If there is both a min and max boundary, that limits how far we can zoom.
    // Make sure we don't exceed that
    if bounded_area_size.x.is_finite() || bounded_area_size.y.is_finite() {
        let max_safe_scale = max_scale_within_bounds(bounded_area_size, proj, window_size);
        proj.scaling_mode = update_scaling_mode(proj.scaling_mode, |v| {
            v.min(max_safe_scale.x).min(max_safe_scale.y)
        });
    }
}

fn camera_zoom(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query: Query<(
        &EditorCamera2d,
        &Camera,
        &mut OrthographicProjection,
        &mut Transform,
    )>,
) {
    let Ok(window) = primary_window.get_single() else {
        // Log an error message once here?
        return;
    };

    for (e_camera, camera, mut projection, mut transform) in query.iter_mut() {
        if !e_camera.enabled {
            return;
        }

        let scroll_delta = mouse_wheel
            .read()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Pixel => ev.y,
                MouseScrollUnit::Line => ev.y * e_camera.scroll_pixels_per_line,
            })
            .sum::<f32>();

        if scroll_delta == 0.0 {
            continue;
        }

        let viewport_size = camera.logical_viewport_size().unwrap_or(window.size());
        let old_projection_scale = projection.scaling_mode;
        projection.scaling_mode *= 1.0 - scroll_delta * e_camera.zoom_sensitivity;

        clamp_scale_to_bound(
            &mut projection,
            e_camera.bound.size(),
            &e_camera.scale_range(),
            viewport_size,
        );

        let cursor_normalized_viewport_pos = window
            .cursor_position()
            .map(|cursor_pos| {
                let view_pos = camera
                    .logical_viewport_rect()
                    .map(|v| v.min)
                    .unwrap_or(Vec2::ZERO);

                ((cursor_pos - view_pos) / viewport_size) * 2. - Vec2::ONE
            })
            .map(|p| Vec2::new(-p.x, p.y));

        // Move the camera position to normalize the projection window
        let (Some(cursor_normalized_view_pos), true) =
            (cursor_normalized_viewport_pos, e_camera.zoom_to_cursor)
        else {
            continue;
        };

        let proj_size = projection.area.max / scaling_mode_as_vec2(&old_projection_scale);

        let cursor_world_pos = transform.translation.truncate()
            + cursor_normalized_view_pos * proj_size * scaling_mode_as_vec2(&old_projection_scale);

        let proposed_cam_pos = cursor_world_pos
            - cursor_normalized_view_pos
                * proj_size
                * scaling_mode_as_vec2(&projection.scaling_mode);

        // As we zoom out, we don't want the viewport to move beyond the provided
        // boundary. If the most recent change to the camera zoom would move cause
        // parts of the window beyond the boundary to be shown, we need to change the
        // camera position to keep the viewport within bounds.
        transform.translation =
            clamp_area_to_bound(proposed_cam_pos, projection.area.size(), e_camera.aabb())
                .extend(transform.translation.z);
    }
}

#[allow(unused)]
fn camera_pan(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(
        &EditorCamera2d,
        &Camera,
        &OrthographicProjection,
        &mut Transform,
    )>,
    mut prev_mouse_pos: Local<Option<Vec2>>,
) {
    // See https://github.com/johanhelsing/bevy_pancam/blob/main/src/lib.rs#L279
    // for why we are using the mouse position instead of the mouse delta (from
    // the MouseMotion event).
    let Ok(window) = primary_window.get_single() else {
        // Log an error message once here?
        return;
    };
    let window_size = window.size();

    let mouse_pos = match window.cursor_position() {
        Some(p) => Vec2::new(p.x, -p.y),
        None => return,
    };
    // This is the pixels the mouse moved since the last frame in "window space".
    let mouse_delta_pixels = mouse_pos - prev_mouse_pos.unwrap_or(mouse_pos);

    for (e_camera, camera, projection, mut transform) in query.iter_mut() {
        if !e_camera.enabled {
            continue;
        }

        let projection_area_size = projection.area.size();
        let mouse_delta = if !e_camera
            .pan_mouse_buttons
            .iter()
            .any(|btn| mouse_buttons.pressed(*btn) && !mouse_buttons.just_pressed(*btn))
        {
            Vec2::ZERO
        } else {
            // Because the `mouse_delta_pixels` is in "window space" we need to convert
            // it to world space by multiplying it with the ratio of the projection area
            // and the viewport size.
            let viewport_size = camera.logical_viewport_size().unwrap_or(window_size);
            mouse_delta_pixels * projection_area_size / viewport_size
        };

        if mouse_delta == Vec2::ZERO {
            continue;
        }

        let proposed_cam_pos = transform.translation.truncate() - mouse_delta;
        transform.translation =
            clamp_area_to_bound(proposed_cam_pos, projection_area_size, e_camera.aabb())
                .extend(transform.translation.z);
    }

    *prev_mouse_pos = Some(mouse_pos);
}

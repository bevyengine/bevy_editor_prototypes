//! A 2d editor camera controller.
//!
//! This module provides a 2d editor camera controller which can be used to pan and zoom the
//! camera.

// Heavily inspired by https://github.com/johanhelsing/bevy_pancam/blob/main/src/lib.rs#L279

use std::ops::RangeInclusive;

use bevy::{
    input::mouse::AccumulatedMouseScroll, math::bounding::Aabb2d, prelude::*,
    render::camera::ScalingMode, window::PrimaryWindow,
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
    /// The current zoom level of the camera.
    ///
    /// This value is clamped to the [`zoom_range`].
    pub zoom: f32,
    /// The range of zoom the allowed zoom the camera will be clamped to.
    ///
    /// To disable clamping set the range to [`f32::NEG_INFINITY`]..=[`f32::INFINITY`].
    pub zoom_range: RangeInclusive<f32>,
    /// The sensitivity of the mouse wheel input when zooming.
    pub zoom_sensitivity: f32,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
}

impl EditorCamera2d {
    fn aabb(&self) -> Aabb2d {
        Aabb2d {
            min: self.bound.min,
            max: self.bound.max,
        }
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
            zoom: 1.0,
            zoom_range: f32::NEG_INFINITY..=f32::INFINITY,
            zoom_sensitivity: 0.1,
            zoom_to_cursor: true,
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
/// ```text
/// +-------Bound------------+
/// |  +---Bounded Area--+   |
/// |  |                 |   |
/// |  |        x (pos)  |   |
/// |  |                 |   |
/// |  +-----------------+   |
/// +------------------------+
/// ```
fn clamp_area_to_bound(pos: Vec2, bounded_area_size: Vec2, bound: Aabb2d) -> Vec2 {
    // We are manually implementing the `bound.shrink()` functionality
    // because the existing one is unsafe and can panic when the bounded
    // area size does not fit within the bound.
    let aabb = Aabb2d {
        min: bound.min + bounded_area_size / 2.0,
        max: bound.max - bounded_area_size / 2.0,
    };

    if aabb.min.x > aabb.max.x || aabb.min.y > aabb.max.y {
        // Return center of bound
        return (aabb.min + aabb.max) / 2.0;
    }

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

fn camera_zoom(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_wheel: Res<AccumulatedMouseScroll>,
    mut query: Query<(
        &mut EditorCamera2d,
        &Camera,
        &mut OrthographicProjection,
        &mut Transform,
    )>,
) {
    let Ok(window) = primary_window.get_single() else {
        // Log an error message once here?
        return;
    };

    for (mut e_camera, camera, mut projection, mut transform) in query.iter_mut() {
        if !e_camera.enabled {
            return;
        }

        if mouse_wheel.delta.y == 0.0 {
            continue;
        }

        let viewport_size = camera.logical_viewport_size().unwrap_or(window.size());
        let old_projection_scale = projection.scaling_mode;

        let scroll_delta = 1.0 - mouse_wheel.delta.y * e_camera.zoom_sensitivity;
        let zoom = e_camera.zoom / scroll_delta;

        // Clamp the scroll delta so that it will never go beyond the zoom range.
        let zoom_delta = if zoom < *e_camera.zoom_range.start() {
            e_camera.zoom / *e_camera.zoom_range.start()
        } else if zoom > *e_camera.zoom_range.end() {
            e_camera.zoom / *e_camera.zoom_range.end()
        } else {
            scroll_delta
        };

        e_camera.zoom /= zoom_delta;
        projection.scaling_mode *= zoom_delta;

        let cursor_normalized_viewport_pos = window
            .cursor_position()
            .map(|cursor_pos| {
                let view_pos = camera
                    .logical_viewport_rect()
                    .map(|v| v.min)
                    .unwrap_or(Vec2::ZERO);

                ((cursor_pos - view_pos) / viewport_size) * 2. - Vec2::ONE
            })
            .map(|p| {
                Vec2::new(p.x, -p.y)
                    // This is a fix because somehow in window scaling mode everything is reversed.
                    * if let ScalingMode::WindowSize(_) = projection.scaling_mode {
                        -1.0
                    } else {
                        1.0
                    }
            });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_area_to_bound() {
        let bound = Aabb2d {
            min: Vec2::new(-10.0, -10.0),
            max: Vec2::new(10.0, 10.0),
        };

        let bounded_area_size = Vec2::new(5.0, 5.0);

        // Test when the bounded area is within the bound
        let pos = Vec2::new(0.0, 0.0);
        assert_eq!(clamp_area_to_bound(pos, bounded_area_size, bound), pos);

        // Test when the bounded area is outside the bound
        let pos = Vec2::new(-10.0, -10.0);
        assert_eq!(
            clamp_area_to_bound(pos, bounded_area_size, bound),
            Vec2::new(-7.5, -7.5)
        );

        let pos = Vec2::new(10.0, 10.0);
        assert_eq!(
            clamp_area_to_bound(pos, bounded_area_size, bound),
            Vec2::new(7.5, 7.5)
        );

        let pos = Vec2::new(10.0, -10.0);
        assert_eq!(
            clamp_area_to_bound(pos, bounded_area_size, bound),
            Vec2::new(7.5, -7.5)
        );

        let pos = Vec2::new(-10.0, 10.0);
        assert_eq!(
            clamp_area_to_bound(pos, bounded_area_size, bound),
            Vec2::new(-7.5, 7.5)
        );
    }
}

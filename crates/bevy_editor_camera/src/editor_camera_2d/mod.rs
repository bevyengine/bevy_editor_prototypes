//! A 2d editor camera controller.
//!
//! This module provides a 2d editor camera controller which can be used to pan and zoom the
//! camera.

// Heavily inspired by https://github.com/johanhelsing/bevy_pancam/blob/main/src/lib.rs#L279

use std::ops::RangeInclusive;

use bevy::{
    camera::CameraProjection,
    input::mouse::AccumulatedMouseScroll,
    math::bounding::{Aabb2d, BoundingVolume},
    prelude::*,
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
    /// The range of zoom the allowed zoom the camera will be clamped to.
    ///
    /// To disable clamping set the range to [`f32::NEG_INFINITY`]..=[`f32::INFINITY`].
    pub scale_range: RangeInclusive<f32>,
    /// The sensitivity of the mouse wheel input when zooming.
    pub zoom_sensitivity: f32,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// Overrides the viewport. Useful to map the controls correctly
    /// when the camera is rendering to an image.
    pub viewport_override: Option<Rect>,
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
            scale_range: f32::NEG_INFINITY..=f32::INFINITY,
            zoom_sensitivity: 0.1,
            zoom_to_cursor: true,
            viewport_override: None,
        }
    }
}

/// Makes sure that the camera projection scale stays in the provided bounds
/// and range.
fn constrain_proj_scale(
    proj: &mut OrthographicProjection,
    bounded_area_size: Vec2,
    scale_range: &RangeInclusive<f32>,
    window_size: Vec2,
) {
    proj.scale = proj.scale.clamp(*scale_range.start(), *scale_range.end());

    // If there is both a min and max boundary, that limits how far we can zoom.
    // Make sure we don't exceed that
    if bounded_area_size.x.is_finite() || bounded_area_size.y.is_finite() {
        let max_safe_scale = max_scale_within_bounds(bounded_area_size, proj, window_size);
        proj.scale = proj.scale.min(max_safe_scale.x).min(max_safe_scale.y);
    }
}

/// Clamps a camera position to a safe zone. "Safe" means that each screen
/// corner is constrained to the corresponding bound corner.
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
///
/// Since bevy doesn't provide a `shrink` method on a `Rect` yet, we have to
/// operate on `Aabb2d` type.
fn clamp_to_safe_zone(pos: Vec2, aabb: Aabb2d, bounded_area_size: Vec2) -> Vec2 {
    let aabb = aabb.shrink(bounded_area_size / 2.);
    pos.clamp(aabb.min, aabb.max)
}

/// `max_scale_within_bounds` is used to find the maximum safe zoom out/projection
/// scale when we have been provided with minimum and maximum x boundaries for
/// the camera.
fn max_scale_within_bounds(
    bounded_area_size: Vec2,
    proj: &OrthographicProjection,
    window_size: Vec2, //viewport?
) -> Vec2 {
    let mut proj = proj.clone();
    proj.scale = 1.;
    proj.update(window_size.x, window_size.y);
    let base_world_size = proj.area.size();
    bounded_area_size / base_world_size
}

fn camera_zoom(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_wheel: Res<AccumulatedMouseScroll>,
    mut query: Query<(
        &mut EditorCamera2d,
        &Camera,
        &mut Projection,
        &mut Transform,
    )>,
) {
    if mouse_wheel.delta.y == 0.0 {
        return;
    }

    let Ok(window) = primary_window.single() else {
        // Log an error message once here?
        return;
    };

    for (e_camera, camera, mut projection, mut transform) in query.iter_mut() {
        if !e_camera.enabled {
            continue;
        }

        let Projection::Orthographic(projection) = projection.as_mut() else {
            panic!("EditorCamera2d requires an Orthographic projection");
        };

        let viewport_size = camera.logical_viewport_size().unwrap_or(window.size());

        let viewport_rect = e_camera
            .viewport_override
            .unwrap_or(Rect::from_corners(Vec2::ZERO, viewport_size));

        let old_scale = projection.scale;
        projection.scale *= 1. - mouse_wheel.delta.y * e_camera.zoom_sensitivity;

        constrain_proj_scale(
            projection,
            e_camera.bound.size(),
            &e_camera.scale_range,
            viewport_size,
        );

        let cursor_normalized_viewport_pos = window
            .cursor_position()
            .map(|cursor_pos| {
                let view_pos = camera
                    .logical_viewport_rect()
                    .map(|v| v.min)
                    .unwrap_or(Vec2::ZERO);

                ((cursor_pos - (view_pos + viewport_rect.min)) / viewport_rect.size()) * 2.
                    - Vec2::ONE
            })
            .map(|p| Vec2::new(p.x, -p.y));

        // Move the camera position to normalize the projection window
        let (Some(cursor_normalized_view_pos), true) =
            (cursor_normalized_viewport_pos, e_camera.zoom_to_cursor)
        else {
            continue;
        };

        let proj_size = projection.area.max / old_scale;

        let cursor_world_pos =
            transform.translation.truncate() + cursor_normalized_view_pos * projection.area.max;

        let proposed_cam_pos =
            cursor_world_pos - cursor_normalized_view_pos * proj_size * projection.scale;

        // As we zoom out, we don't want the viewport to move beyond the provided
        // boundary. If the most recent change to the camera zoom would move cause
        // parts of the window beyond the boundary to be shown, we need to change the
        // camera position to keep the viewport within bounds.
        transform.translation =
            clamp_to_safe_zone(proposed_cam_pos, e_camera.aabb(), projection.area.size())
                .extend(transform.translation.z);
    }
}

fn camera_pan(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&EditorCamera2d, &Camera, &Projection, &mut Transform)>,
    mut prev_mouse_pos: Local<Option<Vec2>>,
) {
    // See https://github.com/johanhelsing/bevy_pancam/blob/main/src/lib.rs#L279
    // for why we are using the mouse position instead of the mouse delta (from
    // the MouseMotion event).
    let Ok(window) = primary_window.single() else {
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

        let Projection::Orthographic(projection) = projection else {
            panic!("EditorCamera2d requires an Orthographic projection");
        };

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
            clamp_to_safe_zone(proposed_cam_pos, e_camera.aabb(), projection_area_size)
                .extend(transform.translation.z);
    }

    *prev_mouse_pos = Some(mouse_pos);
}

//! Configurable options for the challenge of working with orthographic cameras.

use bevy::camera::prelude::*;
use bevy::ecs::prelude::*;
use bevy::reflect::prelude::*;
use bevy::transform::prelude::*;

use crate::prelude::*;

use self::motion::CurrentMotion;

/// Settings used when the [`EditorCam`] has a perspective [`Projection`].
#[derive(Debug, Clone, Reflect)]
pub struct PerspectiveSettings {
    /// Limits the near clipping plane to always fit inside this range.
    ///
    /// The camera controller will try to make the near clipping plane smaller when you zoom in to
    /// ensure the anchor (the thing you are zooming into) is always within the view frustum
    /// (visible), bounded by this limit.
    ///
    /// Unless the camera is zoomed very close to something, it will spend most of the time at the
    /// high end of this limit - you should treat that like the default near clipping plane. Bevy
    /// defaults to `0.1`, and you should probably use that too unless you have a very good reason
    /// not to. Many rendering effects that rely on depth can break down if the clipping plane is
    /// very far from `0.1`.
    pub near_clip_limits: std::ops::Range<f32>,
    /// When computing the near plane position, the anchor depth is multiplied by this value to
    /// determine the new near clip position. This should be smaller than one, to ensure that the
    /// object you are looking at, which will be located at the anchor position, is bot being
    /// clipped. Some parts of the object may protrude toward the camera, which is what necessitates
    /// this.
    pub near_clip_multiplier: f32,
}

impl Default for PerspectiveSettings {
    fn default() -> Self {
        Self {
            near_clip_limits: 1e-9..0.1,
            near_clip_multiplier: 0.05,
        }
    }
}

/// Updates perspective projection properties of editor cameras.
pub fn update_perspective(mut cameras: Query<(&EditorCam, &mut Projection)>) {
    for (editor_cam, mut projection) in cameras.iter_mut() {
        let Projection::Perspective(ref mut perspective) = *projection else {
            continue;
        };
        let limits = editor_cam.perspective.near_clip_limits.clone();
        let multiplier = editor_cam.perspective.near_clip_multiplier;
        perspective.near = (editor_cam.last_anchor_depth.abs() as f32 * multiplier)
            .clamp(limits.start, limits.end);
    }
}

/// Settings used when the [`EditorCam`] has an orthographic [`Projection`].
#[derive(Debug, Clone, Reflect)]
pub struct OrthographicSettings {
    /// The camera's near clipping plane will move closer and farther from the anchor point during
    /// zoom to maximize precision. The position of the near plane is based on the orthographic
    /// projection `scale`, multiplied by this value.
    pub scale_to_near_clip: f32,
    /// Limits the distance the near clip plane can be to the anchor. The low limit is useful to
    /// prevent geometry clipping when zooming in, while the high limit is useful to prevent the
    /// camera moving too far away from the anchor, causing precision issues.
    pub near_clip_limits: std::ops::Range<f32>,
    /// The far plane is placed opposite the anchor from the near plane, at this multiple of the
    /// distance from the near plane to the anchor. Setting this to 1.0 means the camera frustum is
    /// centered on the anchor. It might be desirable to make this larger to prevent things in the
    /// background from disappearing when zooming in.
    pub far_clip_multiplier: f32,
}

impl Default for OrthographicSettings {
    fn default() -> Self {
        Self {
            scale_to_near_clip: 1_000_000.0,
            near_clip_limits: 1.0..1_000_000.0,
            far_clip_multiplier: 1.0,
        }
    }
}

/// Update the ortho camera projection and position based on the [`OrthographicSettings`].
pub fn update_orthographic(mut cameras: Query<(&mut EditorCam, &mut Projection, &mut Transform)>) {
    for (mut editor_cam, mut projection, mut cam_transform) in cameras.iter_mut() {
        let Projection::Orthographic(ref mut orthographic) = *projection else {
            continue;
        };

        let anchor_dist = editor_cam.last_anchor_depth().abs() as f32;
        let target_dist = (editor_cam.orthographic.scale_to_near_clip * orthographic.scale).clamp(
            editor_cam.orthographic.near_clip_limits.start,
            editor_cam.orthographic.near_clip_limits.end,
        );

        let forward_amount = anchor_dist - target_dist;
        let movement = cam_transform.forward() * forward_amount;

        cam_transform.translation += movement;

        editor_cam.last_anchor_depth += forward_amount as f64;
        if let CurrentMotion::UserControlled { ref mut anchor, .. } = editor_cam.current_motion {
            anchor.z += forward_amount as f64;
        }

        orthographic.near = 0.0;
        orthographic.far = anchor_dist * (1.0 + editor_cam.orthographic.far_clip_multiplier);
    }
}

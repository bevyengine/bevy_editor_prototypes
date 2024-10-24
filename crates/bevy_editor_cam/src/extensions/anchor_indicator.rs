//! A `bevy_editor_cam` extension that draws an indicator in the scene at the location of the
//! anchor. This makes it more obvious to users what point in space the camera is rotating around,
//! making it easier to use and understand.

use crate::prelude::*;

use bevy::app::prelude::*;
use bevy::color::Color;
use bevy::ecs::prelude::*;
use bevy::gizmos::prelude::*;
use bevy::math::prelude::*;
use bevy::reflect::prelude::*;
use bevy::render::prelude::*;
use bevy::transform::prelude::*;

/// See the [module](self) docs.
pub struct AnchorIndicatorPlugin;

impl Plugin for AnchorIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            draw_anchor
                .after(TransformSystem::TransformPropagate)
                .after(bevy::render::camera::CameraUpdateSystem),
        )
        .register_type::<AnchorIndicator>();
    }
}

/// Optional. Configures whether or not an [`EditorCam`] should show an anchor indicator when the
/// camera is orbiting. The indicator will be enabled if this component is not present.
#[derive(Debug, Component, Reflect)]
pub struct AnchorIndicator {
    /// Should the indicator be visible on this camera?
    pub enabled: bool,
}

impl Default for AnchorIndicator {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Use gizmos to draw the camera anchor in world space.
pub fn draw_anchor(
    cameras: Query<(
        &EditorCam,
        &GlobalTransform,
        &Camera,
        Option<&AnchorIndicator>,
    )>,
    mut gizmos: Gizmos,
) {
    for (editor_cam, cam_transform, cam, _) in cameras
        .iter()
        .filter(|(.., anchor_indicator)| anchor_indicator.map(|a| a.enabled).unwrap_or(true))
    {
        let Some(anchor_world) = editor_cam.anchor_world_space(cam_transform) else {
            continue;
        };
        let p1 = cam
            .world_to_viewport(cam_transform, anchor_world.as_vec3())
            .unwrap_or_default();
        let p2 = cam
            .world_to_viewport(
                cam_transform,
                anchor_world.as_vec3() + cam_transform.right().as_vec3(),
            )
            .unwrap_or_default();

        let scale = 8.0 / (p2 - p1).length();

        // Shift the indicator toward the camera to prevent it clipping objects near parallel
        let shift = (cam_transform.translation() - anchor_world.as_vec3()).normalize() * scale;
        let anchor_world = anchor_world.as_vec3() + shift;

        if editor_cam.current_motion.is_orbiting() {
            let gizmo_color = || Color::WHITE;
            let arm_length = 0.4;

            let isometry = Isometry3d::new(
                anchor_world,
                Quat::from_rotation_arc(Vec3::Z, *cam_transform.forward()),
            );

            gizmos.circle(isometry, scale, gizmo_color());
            let offset = 1.5 * scale;
            gizmos.ray(
                anchor_world + offset * cam_transform.left(),
                offset * arm_length * cam_transform.left(),
                gizmo_color(),
            );
            gizmos.ray(
                anchor_world + offset * cam_transform.right(),
                offset * arm_length * cam_transform.right(),
                gizmo_color(),
            );
            gizmos.ray(
                anchor_world + offset * cam_transform.up(),
                offset * arm_length * cam_transform.up(),
                gizmo_color(),
            );
            gizmos.ray(
                anchor_world + offset * cam_transform.down(),
                offset * arm_length * cam_transform.down(),
                gizmo_color(),
            );
        }
    }
}

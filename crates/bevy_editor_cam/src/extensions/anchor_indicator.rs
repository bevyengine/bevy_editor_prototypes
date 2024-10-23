//! A `bevy_editor_cam` extension that draws an indicator in the scene at the location of the
//! anchor. This makes it more obvious to users what point in space the camera is rotating around,
//! making it easier to use and understand.

use crate::prelude::*;

use bevy::{asset::embedded_asset, prelude::*};

/// See the [module](self) docs.
pub struct AnchorIndicatorPlugin;

impl Plugin for AnchorIndicatorPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/tilt-shift.png");

        app.add_systems(
            PostUpdate,
            draw_anchor.after(bevy::render::camera::CameraUpdateSystem),
        )
        .add_observer(
            |trigger: Trigger<OnAdd, EditorCam>,
             mut commands: Commands,
             asset_server: Res<AssetServer>| {
                let image = asset_server
                    .load("embedded://bevy_editor_cam/extensions/assets/tilt-shift.png");

                let id = commands
                    .spawn((
                        UiImage::new(image),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(24.),
                            height: Val::Px(24.),
                            margin: UiRect::all(Val::Px(-12.)),
                            ..default()
                        },
                        TargetCamera(trigger.entity()),
                        PickingBehavior::IGNORE,
                    ))
                    .id();

                commands.entity(trigger.entity()).insert(AnchorRoot(id));
            },
        )
        .add_observer(
            |trigger: Trigger<OnRemove, AnchorRoot>,
             mut commands: Commands,
             anchor_root_query: Query<&AnchorRoot>| {
                if let Ok(anchor_root) = anchor_root_query.get(trigger.entity()) {
                    commands.entity(anchor_root.0).despawn_recursive();
                }
            },
        )
        .register_type::<AnchorIndicator>()
        .register_type::<AnchorRoot>();
    }
}

/// A reference to the image node used for the orbit anchor indicator.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct AnchorRoot(pub Entity);

/// Optional. Configures whether or not an [`EditorCam`] should show an anchor indicator when the
/// camera is orbiting. The indicator will be enabled if this component is not present.
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
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
#[expect(clippy::type_complexity)]
pub fn draw_anchor(
    cameras: Query<(
        Ref<EditorCam>,
        &GlobalTransform,
        &Camera,
        &AnchorRoot,
        Option<Ref<AnchorIndicator>>,
    )>,
    mut node_query: Query<&mut Node>,
) {
    for (editor_cam, cam_transform, cam, anchor_root, anchor_indicator) in &cameras {
        let Ok(mut node) = node_query.get_mut(anchor_root.0) else {
            continue;
        };
        let enabled = anchor_indicator.map(|a| a.enabled).unwrap_or(true);
        let Some(anchor_world) = enabled
            .then(|| editor_cam.anchor_world_space(cam_transform))
            .flatten()
        else {
            node.display = Display::None;
            continue;
        };
        if editor_cam.current_motion.is_orbiting() {
            let position = cam
                .world_to_viewport(cam_transform, anchor_world.as_vec3())
                .unwrap_or_default();

            node.display = Display::Flex;
            node.top = Val::Px(position.y);
            node.left = Val::Px(position.x);
        } else {
            node.display = Display::None;
        }
    }
}

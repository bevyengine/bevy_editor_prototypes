//! A `bevy_editor_cam` extension that provides a skybox rendered by a different camera with a
//! different field of view than the camera it is added to. This allows you to use a very narrow
//! camera FOV, or even orthographic projection, while keeping the appearance of the skybox
//! unchanged.
//!
//! To use it, add a [`IndependentSkybox`] component to a camera.

use bevy::app::prelude::*;
use bevy::asset::Handle;
use bevy::camera::{prelude::*, visibility::RenderLayers};
use bevy::core_pipeline::Skybox;
use bevy::ecs::prelude::*;
use bevy::image::Image;
use bevy::reflect::prelude::*;
use bevy::render::view::Hdr;
use bevy::transform::prelude::*;

/// See the [module](self) docs.
pub struct IndependentSkyboxPlugin;

impl Plugin for IndependentSkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                IndependentSkyboxCamera::spawn,
                IndependentSkyboxCamera::despawn,
                ApplyDeferred,
                IndependentSkyboxCamera::update,
            )
                .chain(),
        );
    }
}

/// Add this to a camera to enable rendering a skybox with these parameters.
#[derive(Debug, Clone, Reflect, Component)]
pub struct IndependentSkybox {
    /// The image to render as a skybox.
    pub skybox: Handle<Image>,
    /// Used to set [`Skybox::brightness`].
    pub brightness: f32,
    /// The [`Camera::order`] of the skybox camera, offset from the camera it is tracking. This
    /// should be lower than the order of the primary camera controller camera. the default value
    /// should be sufficient for most cases. You can override this if you have a more complex use
    /// case with multiple cameras.
    pub skybox_cam_order_offset: isize,
    /// The field of view of the skybox.
    pub fov: SkyboxFov,
    /// The corresponding skybox camera entity.
    skybox_cam: Option<Entity>,
}

impl IndependentSkybox {
    /// Create a new [`IndependentSkybox`] with default settings and the provided skybox image.
    pub fn new(skybox: Handle<Image>, brightness: f32) -> Self {
        Self {
            skybox,
            brightness,
            ..Default::default()
        }
    }
}

impl Default for IndependentSkybox {
    fn default() -> Self {
        Self {
            skybox: Handle::default(),
            brightness: 500.0,
            skybox_cam_order_offset: -1_000,
            fov: Default::default(),
            skybox_cam: Default::default(),
        }
    }
}

/// Field of view setting for the [`IndependentSkybox`]
#[derive(Debug, Clone, Reflect)]
pub enum SkyboxFov {
    /// Match the [`PerspectiveProjection::fov`] of the camera this skybox camera is following.
    Auto,
    /// Use a fixed value for the skybox field of view. This value is equivalent to
    /// [`PerspectiveProjection::fov`].
    Fixed(f32),
}

impl Default for SkyboxFov {
    fn default() -> Self {
        Self::Fixed(PerspectiveProjection::default().fov)
    }
}

/// Used to track the camera that is used to render a skybox, using the [`IndependentSkybox`]
/// component settings placed on a camera.
#[derive(Component)]
#[require(Hdr)]
pub struct IndependentSkyboxCamera {
    /// The camera that this skybox camera is observing.
    driven_by: Entity,
}

impl IndependentSkyboxCamera {
    /// Spawns [`IndependentSkyboxCamera`]s when a [`IndependentSkybox`] exists without a skybox
    /// entity.
    pub fn spawn(
        mut commands: Commands,
        mut editor_cams: Query<(Entity, &mut IndependentSkybox, &mut Camera)>,
        skybox_cams: Query<&IndependentSkyboxCamera>,
    ) {
        for (editor_cam_entity, mut editor_without_skybox, mut camera) in
            editor_cams.iter_mut().filter(|(_, config, ..)| {
                config
                    .skybox_cam
                    .and_then(|e| skybox_cams.get(e).ok())
                    .is_none()
            })
        {
            camera.clear_color = ClearColorConfig::None;

            let entity = commands
                .spawn((
                    Camera3d::default(),
                    Camera {
                        order: camera.order + editor_without_skybox.skybox_cam_order_offset,
                        clear_color: ClearColorConfig::None,
                        ..Default::default()
                    },
                    Projection::Perspective(PerspectiveProjection {
                        fov: match editor_without_skybox.fov {
                            SkyboxFov::Auto => PerspectiveProjection::default().fov,
                            SkyboxFov::Fixed(fov) => fov,
                        },
                        ..Default::default()
                    }),
                    RenderLayers::none(),
                    Skybox {
                        image: editor_without_skybox.skybox.clone(),
                        brightness: editor_without_skybox.brightness,
                        ..Default::default()
                    },
                    IndependentSkyboxCamera {
                        driven_by: editor_cam_entity,
                    },
                ))
                .id();
            editor_without_skybox.skybox_cam = Some(entity);
        }
    }

    /// Despawns [`IndependentSkyboxCamera`]s when their corresponding [`IndependentSkybox`] entity
    /// does not exist.
    pub fn despawn(
        mut commands: Commands,
        skybox_cams: Query<(Entity, &IndependentSkyboxCamera)>,
        editor_cams: Query<&IndependentSkybox>,
    ) {
        for (skybox_entity, skybox) in &skybox_cams {
            if editor_cams.get(skybox.driven_by).is_err() {
                commands.entity(skybox_entity).despawn();
            }
        }
    }

    /// Update the position and projection of this [`IndependentSkyboxCamera`] to copy the camera it
    /// is following.
    #[allow(clippy::type_complexity)]
    pub fn update(
        mut editor_cams: Query<
            (&IndependentSkybox, &Transform, &Projection, &Camera),
            (
                Or<(Changed<IndependentSkybox>, Changed<Transform>)>,
                Without<Self>,
            ),
        >,
        mut skybox_cams: Query<(&mut Transform, &mut Projection, &mut Camera), With<Self>>,
    ) {
        for (editor_cam, editor_transform, editor_projection, camera) in &mut editor_cams {
            let Some(skybox_entity) = editor_cam.skybox_cam else {
                continue;
            };
            let Ok((mut skybox_transform, mut skybox_projection, mut skybox_camera)) =
                skybox_cams.get_mut(skybox_entity)
            else {
                continue;
            };

            skybox_camera.viewport.clone_from(&camera.viewport);

            if let Projection::Perspective(editor_perspective) = editor_projection {
                *skybox_projection = Projection::Perspective(PerspectiveProjection {
                    fov: match editor_cam.fov {
                        SkyboxFov::Auto => editor_perspective.fov,
                        SkyboxFov::Fixed(fov) => fov,
                    },
                    ..editor_perspective.clone()
                });
            }

            *skybox_transform = *editor_transform;
        }
    }
}

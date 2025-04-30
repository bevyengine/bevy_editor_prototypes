//! This module sets up a simple way to spawn a view gizmo that indicates the 3 main axis
//! It's currently hard coded to the top left of the parent `UiNode` it's spawned in.
//! It currently doesn't support any input event to move the camera based on a click.

use bevy::{
    asset::RenderAssetUsages,
    ecs::relationship::RelatedSpawnerCommands,
    prelude::*,
    render::{
        render_resource::{Extent3d, Face, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use bevy_editor_cam::prelude::EditorCam;

// That value was picked arbitrarily
pub const VIEW_GIZMO_TEXTURE_SIZE: u32 = 125;
// TODO we really shouldn't just hardcode view layers like that
pub const VIEW_GIZMO_LAYER: usize = 22;

const GIZMO_CAMERA_ZOOM: f32 = 3.5;

pub struct ViewGizmoPlugin;
impl Plugin for ViewGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_view_gizmo)
            .add_systems(Update, (spawn_view_gizmo_camera, update_view_gizmo));
    }
}

#[derive(Component)]
pub struct ViewGizmoCamera;

#[derive(Component)]
pub struct ViewGizmoCameraTarget(pub Handle<Image>);

pub fn spawn_view_gizmo_target_texture(
    mut images: ResMut<'_, Assets<Image>>,
    parent: &mut RelatedSpawnerCommands<ChildOf>,
) {
    let size = Extent3d {
        width: VIEW_GIZMO_TEXTURE_SIZE,
        height: VIEW_GIZMO_TEXTURE_SIZE,
        ..default()
    };

    let mut target_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    target_texture.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image = images.add(target_texture);

    // TODO don't hardcode it to top left
    // TODO send input events to the image target
    parent.spawn((
        ImageNode::new(image.clone()),
        Node {
            position_type: PositionType::Absolute,
            top: Val::ZERO,
            bottom: Val::ZERO,
            left: Val::ZERO,
            right: Val::ZERO,
            width: Val::Px(VIEW_GIZMO_TEXTURE_SIZE as f32),
            height: Val::Px(VIEW_GIZMO_TEXTURE_SIZE as f32),
            ..default()
        },
        ViewGizmoCameraTarget(image.clone()),
    ));
}

fn setup_view_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    let view_gizmo_pass_layer = RenderLayers::layer(VIEW_GIZMO_LAYER);
    let sphere = meshes.add(Sphere::new(0.2).mesh().uv(32, 18));

    for axis in [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ] {
        let mut gizmo = GizmoAsset::new();
        let color = LinearRgba::from_vec3(axis);
        gizmo.line(Vec3::ZERO, axis, color);
        commands.spawn((
            Gizmo {
                handle: gizmo_assets.add(gizmo),
                line_config: GizmoLineConfig {
                    width: 2.5,
                    ..default()
                },
                ..default()
            },
            Transform::from_xyz(0., 0., 0.),
            view_gizmo_pass_layer.clone(),
        ));
        // TODO react to click on the spheres to snap camera to axis
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color.into(),
                unlit: true,
                ..Default::default()
            })),
            Transform::from_translation(axis),
            view_gizmo_pass_layer.clone(),
        ));
    }
    // Use a sphere for the background
    let sphere = meshes.add(Sphere::new(1.3).mesh().uv(32, 18));
    commands.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: LinearRgba::new(0.0, 0.0, 0.0, 0.5).into(),
            unlit: true,
            // reverse cull mode so it appears behind
            cull_mode: Some(Face::Front),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        view_gizmo_pass_layer.clone(),
    ));
}

fn spawn_view_gizmo_camera(
    mut commands: Commands,
    q: Query<&ViewGizmoCameraTarget, Added<ViewGizmoCameraTarget>>,
) {
    let view_gizmo_pass_layer = RenderLayers::layer(VIEW_GIZMO_LAYER);
    for target in &q {
        commands.spawn((
            Camera3d::default(),
            Camera {
                target: target.0.clone().into(),
                clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Y),
            view_gizmo_pass_layer.clone(),
            ViewGizmoCamera,
        ));
    }
}

fn update_view_gizmo(
    mut view_cube_camera: Query<&mut Transform, (With<ViewGizmoCamera>, With<Camera3d>)>,
    viewport_camera: Query<&Transform, (Without<ViewGizmoCamera>, With<Camera3d>, With<EditorCam>)>,
) {
    for mut transform in &mut view_cube_camera {
        if let Ok(viewport_camera_transform) = viewport_camera.get_single() {
            transform.translation = viewport_camera_transform.back() * GIZMO_CAMERA_ZOOM;
            transform.rotation = viewport_camera_transform.rotation;
        }
    }
}

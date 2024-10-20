//! Renders two cameras to the same window to accomplish "split screen".

use bevy::{
    core_pipeline::tonemapping::Tonemapping, prelude::*, render::camera::Viewport,
    window::WindowResized,
};
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_mod_picking::DefaultPickingPlugins,
            DefaultEditorCamPlugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, set_camera_viewports)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_helmets(27, &asset_server, &mut commands);

    let diffuse_map = asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2");
    let specular_map = asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2");

    // Left Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        EnvironmentMapLight {
            intensity: 1000.0,
            diffuse_map: diffuse_map.clone(),
            specular_map: specular_map.clone(),
        },
        EditorCam::default(),
        bevy_editor_cam::extensions::independent_skybox::IndependentSkybox::new(
            diffuse_map.clone(),
            500.0,
        ),
        LeftCamera,
    ));

    // Right Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(1.0, 1.0, 1.5).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                // Renders the right camera after the left camera, which has a default priority of 0
                order: 10,
                hdr: true,
                // don't clear on the second camera because the first camera already cleared the window
                clear_color: ClearColorConfig::None,
                ..default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 0.01,
                ..default()
            }),
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        EnvironmentMapLight {
            intensity: 1000.0,
            diffuse_map: diffuse_map.clone(),
            specular_map: specular_map.clone(),
        },
        EditorCam::default(),
        bevy_editor_cam::extensions::independent_skybox::IndependentSkybox::new(diffuse_map, 500.0),
        RightCamera,
    ));
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let mut left_camera = left_camera.single_mut();
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });

        let mut right_camera = right_camera.single_mut();
        right_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(window.resolution.physical_width() / 2, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });
    }
}

fn spawn_helmets(n: usize, asset_server: &AssetServer, commands: &mut Commands) {
    let half_width = (((n as f32).powf(1.0 / 3.0) - 1.0) / 2.0) as i32;
    let scene = asset_server.load("models/PlaneEngine/scene.gltf#Scene0");
    let width = -half_width..=half_width;
    for x in width.clone() {
        for y in width.clone() {
            for z in width.clone() {
                commands.spawn((SceneBundle {
                    scene: scene.clone(),
                    transform: Transform::from_translation(IVec3::new(x, y, z).as_vec3() * 2.0)
                        .with_scale(Vec3::splat(1.)),
                    ..default()
                },));
            }
        }
    }
}

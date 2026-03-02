//! Renders two cameras to the same window to accomplish "split screen".

use bevy::{camera::Viewport, prelude::*, render::view::Hdr, window::WindowResized};
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, setup)
        .add_systems(Update, set_camera_viewports)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Left Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera::default(),
        Hdr,
        EditorCam::default(),
        LeftCamera,
    ));

    // Right Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.0, 1.0, 1.5).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            // Renders the right camera after the left camera, which has a default priority of 0
            order: 10,
            // don't clear on the second camera because the first camera already cleared the window
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Hdr,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.01,
            ..OrthographicProjection::default_3d()
        }),
        EditorCam::default(),
        RightCamera,
    ));

    let n = 27;
    let half_width = (((n as f32).powf(1.0 / 3.0) - 1.0) / 2.0) as i32;
    let mesh = meshes.add(Cone::default());
    let material = materials.add(Color::WHITE);
    let width = -half_width..=half_width;
    for x in width.clone() {
        for y in width.clone() {
            for z in width.clone() {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(IVec3::new(x, y, z).as_vec3() * 2.0)
                        .with_scale(Vec3::splat(1.)),
                ));
            }
        }
    }
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let mut left_camera = left_camera.single_mut().unwrap();
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });

        let mut right_camera = right_camera.single_mut().unwrap();
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

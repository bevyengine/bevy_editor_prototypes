//! Demonstrates a pseudo ortho camera - a camera that uses a very narrow perspective projection.
//! This might be useful if certain features are not supported in ortho.

use bevy::{prelude::*, render::view::Hdr};
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, (setup, setup_ui))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1000.0, 1000.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 0.001,
            ..default()
        }),
        Camera::default(),
        Hdr,
        // This component makes the camera controllable with this plugin:
        EditorCam::default(),
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

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(
            "Left Mouse - Pan\n\
            Right Mouse - Orbit\n\
            Scroll - Zoom\n",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

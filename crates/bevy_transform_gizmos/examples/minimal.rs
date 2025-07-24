//! Minimal example.

use bevy::prelude::*;
use bevy_editor_core::prelude::*;
use bevy_transform_gizmos::prelude::*;

fn main() {
    App::new()
        .insert_resource(TransformGizmoSettings {
            // Align the gizmo to a different coordinate system.
            // Leave at the default value to align to the scene's coordinate system.
            alignment_rotation: Quat::from_rotation_y(-0.2),
            ..default()
        })
        .add_plugins((DefaultPlugins, TransformGizmoPlugin))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
        Transform::from_scale(Vec3::splat(5.0)),
        bevy_transform_gizmos::GizmoTransformable,
    ));
    // cube
    let id = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            bevy_transform_gizmos::GizmoTransformable,
        ))
        .id();
    commands.insert_resource(SelectedEntity(Some(id)));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GizmoCamera,
    ));
}

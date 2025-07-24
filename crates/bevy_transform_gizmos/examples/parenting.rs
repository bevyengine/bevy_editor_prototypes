//! Parenting example.

use bevy::prelude::*;
use bevy_editor_core::prelude::*;
use bevy_transform_gizmos::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin::default(),
            TransformGizmoPlugin::default(),
        ))
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
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)).with_scale(Vec3::splat(5.0)),
        bevy_transform_gizmos::GizmoTransformable,
    ));

    let tan = Color::srgb_u8(204, 178, 153);
    let red = Color::srgb_u8(127, 26, 26);

    // cube
    let id = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
            MeshMaterial3d(materials.add(StandardMaterial::from(red))),
            Transform::from_xyz(-1.0, 0.0, 0.0),
            bevy_transform_gizmos::GizmoTransformable,
        ))
        .with_children(|commands| {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
                MeshMaterial3d(materials.add(StandardMaterial::from(tan))),
                Transform::from_xyz(1.0, 0.0, 0.0),
                bevy_transform_gizmos::GizmoTransformable,
            ));
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.0)))),
                MeshMaterial3d(materials.add(StandardMaterial::from(tan))),
                Transform::from_xyz(1.0, 1.0, 0.0),
                bevy_transform_gizmos::GizmoTransformable,
            ));
        })
        .id();

    commands.insert_resource(SelectedEntity(Some(id)));
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GizmoCamera,
    ));
}

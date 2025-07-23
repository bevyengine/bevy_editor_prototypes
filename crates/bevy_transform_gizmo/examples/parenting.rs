use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_transform_gizmo::TransformGizmoPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins,
            TransformGizmoPlugin::new(Quat::default()),
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
        PbrBundle {
            mesh: meshes.add(Plane3d::default()),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            transform: Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))
                .with_scale(Vec3::splat(5.0)),
            ..Default::default()
        },
        bevy_mod_picking::PickableBundle::default(),
        bevy_transform_gizmo::GizmoTransformable,
    ));

    let tan = Color::srgb_u8(204, 178, 153);
    let red = Color::srgb_u8(127, 26, 26);

    // cube
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
                material: materials.add(StandardMaterial::from(red)),
                transform: Transform::from_xyz(-1.0, 0.0, 0.0),
                ..default()
            },
            bevy_mod_picking::PickableBundle::default(),
            bevy_transform_gizmo::GizmoTransformable,
        ))
        .with_children(|commands| {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
                    material: materials.add(StandardMaterial::from(tan)),
                    transform: Transform::from_xyz(1.0, 0.0, 0.0),
                    ..default()
                },
                bevy_mod_picking::PickableBundle::default(),
                bevy_transform_gizmo::GizmoTransformable,
            ));
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
                    material: materials.add(StandardMaterial::from(tan)),
                    transform: Transform::from_xyz(1.0, 1.0, 0.0),
                    ..default()
                },
                bevy_mod_picking::PickableBundle::default(),
                bevy_transform_gizmo::GizmoTransformable,
            ));
        });

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        bevy_transform_gizmo::GizmoPickSource::default(),
    ));
}

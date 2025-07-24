use std::f32::consts::TAU;

use crate::{InteractionKind, InternalGizmoCamera, TransformGizmo};
use bevy::{
    core_pipeline::core_3d::Camera3dDepthLoadOp, pbr::NotShadowCaster, prelude::*,
    render::view::RenderLayers,
};

#[derive(Component)]
pub struct RotationGizmo;

#[derive(Component)]
pub struct ViewTranslateGizmo;

/// Startup system that builds the procedural mesh and materials of the gizmo.
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let axis_length = 1.3;
    let arc_radius = TAU / 4.;
    let plane_size = axis_length * 0.25;
    let plane_offset = plane_size / 2. + axis_length * 0.2;
    // Define gizmo meshes
    let arrow_tail_mesh = meshes.add(Capsule3d {
        radius: 0.04,
        half_length: axis_length * 0.5f32,
    });

    let cone_mesh = meshes.add(Cone {
        height: 0.25,
        radius: 0.10,
    });
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));
    let sphere_mesh = meshes.add(Sphere { radius: 0.2 });
    let rotation_mesh = meshes.add(Mesh::from(
        Torus {
            major_radius: 1.,
            minor_radius: 0.04,
        }
        .mesh()
        .angle_range(0f32..=arc_radius),
    ));

    // Define gizmo materials
    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        }
    }
    let (s, l) = (0.8, 0.6);
    let gizmo_matl_x = materials.add(material(Color::hsl(0.0, s, l)));
    let gizmo_matl_y = materials.add(material(Color::hsl(120.0, s, l)));
    let gizmo_matl_z = materials.add(material(Color::hsl(240.0, s, l)));
    let gizmo_matl_x_sel = materials.add(material(Color::hsl(0.0, s, l)));
    let gizmo_matl_y_sel = materials.add(material(Color::hsl(120.0, s, l)));
    let gizmo_matl_z_sel = materials.add(material(Color::hsl(240.0, s, l)));
    let gizmo_matl_v_sel = materials.add(material(Color::hsl(0., 0.0, l)));

    // Build the gizmo using the variables above.
    commands
        .spawn(TransformGizmo::default())
        .with_children(|parent| {
            // Translation Axes
            parent.spawn((
                Mesh3d(arrow_tail_mesh.clone()),
                MeshMaterial3d(gizmo_matl_x.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    Vec3::new(axis_length / 2.0, 0.0, 0.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(arrow_tail_mesh.clone()),
                MeshMaterial3d(gizmo_matl_y.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, axis_length / 2.0, 0.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(arrow_tail_mesh),
                MeshMaterial3d(gizmo_matl_z.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, 0.0, axis_length / 2.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            // Translation Handles
            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(gizmo_matl_x_sel.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                    Vec3::new(axis_length, 0.0, 0.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(gizmo_matl_x_sel.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                    Vec3::new(0., plane_offset, plane_offset),
                )),
                InteractionKind::TranslatePlane {
                    original: Vec3::X,
                    normal: Vec3::X,
                },
                // NoBackfaceCulling,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(gizmo_matl_y_sel.clone()),
                Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
                InteractionKind::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(gizmo_matl_y_sel.clone()),
                Transform::from_translation(Vec3::new(plane_offset, 0.0, plane_offset)),
                InteractionKind::TranslatePlane {
                    original: Vec3::Y,
                    normal: Vec3::Y,
                },
                // NoBackfaceCulling,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(gizmo_matl_z_sel.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, 0.0, axis_length),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(gizmo_matl_z_sel.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    Vec3::new(plane_offset, plane_offset, 0.0),
                )),
                InteractionKind::TranslatePlane {
                    original: Vec3::Z,
                    normal: Vec3::Z,
                },
                // NoBackfaceCulling,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(gizmo_matl_v_sel.clone()),
                InteractionKind::TranslatePlane {
                    original: Vec3::ZERO,
                    normal: Vec3::Z,
                },
                ViewTranslateGizmo,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            // Rotation Arcs
            parent.spawn((
                Mesh3d(rotation_mesh.clone()),
                MeshMaterial3d(gizmo_matl_x.clone()),
                Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))),
                RotationGizmo,
                InteractionKind::RotateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(rotation_mesh.clone()),
                MeshMaterial3d(gizmo_matl_y.clone()),
                RotationGizmo,
                InteractionKind::RotateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                Mesh3d(rotation_mesh.clone()),
                MeshMaterial3d(gizmo_matl_z.clone()),
                Transform::from_rotation(
                    Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                        * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                ),
                RotationGizmo,
                InteractionKind::RotateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
        });

    commands.spawn((
        Camera3d {
            depth_load_op: Camera3dDepthLoadOp::Clear(0.),
            ..default()
        },
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        InternalGizmoCamera,
        RenderLayers::layer(12),
    ));
}

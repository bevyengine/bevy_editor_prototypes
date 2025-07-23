use crate::{
    gizmo_material::GizmoMaterial, InternalGizmoCamera, PickableGizmo, TransformGizmoBundle,
    TransformGizmoInteraction,
};
use bevy::{
    core_pipeline::core_3d::Camera3dDepthLoadOp, pbr::NotShadowCaster, prelude::*,
    render::view::RenderLayers,
};
use bevy_mod_raycast::prelude::NoBackfaceCulling;

mod cone;
mod truncated_torus;

#[derive(Component)]
pub struct RotationGizmo;

#[derive(Component)]
pub struct ViewTranslateGizmo;

/// Startup system that builds the procedural mesh and materials of the gizmo.
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    let axis_length = 1.3;
    let arc_radius = 1.;
    let plane_size = axis_length * 0.25;
    let plane_offset = plane_size / 2. + axis_length * 0.2;
    // Define gizmo meshes
    let arrow_tail_mesh = meshes.add(Capsule3d {
        radius: 0.04,
        half_length: axis_length * 0.5f32,
    });
    let cone_mesh = meshes.add(cone::Cone {
        height: 0.25,
        radius: 0.10,
        ..Default::default()
    });
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));
    let sphere_mesh = meshes.add(Sphere { radius: 0.2 });
    let rotation_mesh = meshes.add(Mesh::from(truncated_torus::TruncatedTorus {
        radius: arc_radius,
        ring_radius: 0.04,
        ..Default::default()
    }));
    //let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.15 }));
    // Define gizmo materials
    let (s, l) = (0.8, 0.6);
    let gizmo_matl_x = materials.add(GizmoMaterial::from(Color::hsl(0.0, s, l)));
    let gizmo_matl_y = materials.add(GizmoMaterial::from(Color::hsl(120.0, s, l)));
    let gizmo_matl_z = materials.add(GizmoMaterial::from(Color::hsl(240.0, s, l)));
    let gizmo_matl_x_sel = materials.add(GizmoMaterial::from(Color::hsl(0.0, s, l)));
    let gizmo_matl_y_sel = materials.add(GizmoMaterial::from(Color::hsl(120.0, s, l)));
    let gizmo_matl_z_sel = materials.add(GizmoMaterial::from(Color::hsl(240.0, s, l)));
    let gizmo_matl_v_sel = materials.add(GizmoMaterial::from(Color::hsl(0., 0.0, l)));
    /*let gizmo_matl_origin = materials.add(StandardMaterial {
        unlit: true,
        base_color: Color::rgb(0.7, 0.7, 0.7),
        ..Default::default()
    });*/
    // Build the gizmo using the variables above.
    commands
        .spawn(TransformGizmoBundle::default())
        .with_children(|parent| {
            // Translation Axes
            parent.spawn((
                MaterialMeshBundle {
                    mesh: arrow_tail_mesh.clone(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                        Vec3::new(axis_length / 2.0, 0.0, 0.0),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: arrow_tail_mesh.clone(),
                    material: gizmo_matl_y.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, axis_length / 2.0, 0.0),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: arrow_tail_mesh,
                    material: gizmo_matl_z.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, axis_length / 2.0),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            // Translation Handles
            parent.spawn((
                MaterialMeshBundle {
                    mesh: cone_mesh.clone(),
                    material: gizmo_matl_x_sel.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(axis_length, 0.0, 0.0),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: plane_mesh.clone(),
                    material: gizmo_matl_x_sel.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(0., plane_offset, plane_offset),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslatePlane {
                    original: Vec3::X,
                    normal: Vec3::X,
                },
                NoBackfaceCulling,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: cone_mesh.clone(),
                    material: gizmo_matl_y_sel.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: plane_mesh.clone(),
                    material: gizmo_matl_y_sel.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        plane_offset,
                        0.0,
                        plane_offset,
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslatePlane {
                    original: Vec3::Y,
                    normal: Vec3::Y,
                },
                NoBackfaceCulling,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: cone_mesh.clone(),
                    material: gizmo_matl_z_sel.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, axis_length),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: plane_mesh.clone(),
                    material: gizmo_matl_z_sel.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(plane_offset, plane_offset, 0.0),
                    )),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslatePlane {
                    original: Vec3::Z,
                    normal: Vec3::Z,
                },
                NoBackfaceCulling,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            parent.spawn((
                MaterialMeshBundle {
                    mesh: sphere_mesh.clone(),
                    material: gizmo_matl_v_sel.clone(),
                    ..Default::default()
                },
                PickableGizmo::default(),
                TransformGizmoInteraction::TranslatePlane {
                    original: Vec3::ZERO,
                    normal: Vec3::Z,
                },
                ViewTranslateGizmo,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            // Rotation Arcs
            parent.spawn((
                MaterialMeshBundle {
                    mesh: rotation_mesh.clone(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_rotation(Quat::from_axis_angle(
                        Vec3::Z,
                        f32::to_radians(90.0),
                    )),
                    ..Default::default()
                },
                RotationGizmo,
                PickableGizmo::default(),
                TransformGizmoInteraction::RotateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: rotation_mesh.clone(),
                    material: gizmo_matl_y.clone(),
                    ..Default::default()
                },
                RotationGizmo,
                PickableGizmo::default(),
                TransformGizmoInteraction::RotateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
            parent.spawn((
                MaterialMeshBundle {
                    mesh: rotation_mesh.clone(),
                    material: gizmo_matl_z.clone(),
                    transform: Transform::from_rotation(
                        Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                            * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                    ),
                    ..Default::default()
                },
                RotationGizmo,
                PickableGizmo::default(),
                TransformGizmoInteraction::RotateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                },
                NotShadowCaster,
                RenderLayers::layer(12),
            ));
        });

    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                depth_load_op: Camera3dDepthLoadOp::Clear(0.),
                ..default()
            },
            camera: Camera {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..Default::default()
        },
        InternalGizmoCamera,
        RenderLayers::layer(12),
    ));
}

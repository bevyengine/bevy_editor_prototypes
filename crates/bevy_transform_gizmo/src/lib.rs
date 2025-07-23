#![allow(clippy::type_complexity)]

use bevy::asset::load_internal_asset;
use bevy::{prelude::*, render::camera::Projection, transform::TransformSystem};
use bevy_mod_picking::{
    backend::{HitData, PointerHits},
    picking_core::PickSet,
    prelude::{PickingInteraction, PointerId},
    selection::{NoDeselect, PickSelection},
};
use bevy_mod_raycast::prelude::RaycastSystem;
use gizmo_material::GizmoMaterial;
use mesh::{RotationGizmo, ViewTranslateGizmo};
use normalization::*;

mod gizmo_material;
mod mesh;
pub mod normalization;

pub mod picking;

use picking::GizmoRaycastSet;
pub use picking::{GizmoPickSource, PickableGizmo};

#[derive(Resource, Clone, Debug)]
pub struct GizmoSystemsEnabled(pub bool);

pub use normalization::Ui3dNormalization;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum TransformGizmoSystem {
    InputsSet,
    MainSet,
    RaycastSet,
    NormalizeSet,
    UpdateSettings,
    AdjustViewTranslateGizmo,
    Place,
    Hover,
    Grab,
    Drag,
}

#[derive(Debug, Clone, Event)]
pub struct TransformGizmoEvent {
    pub from: GlobalTransform,
    pub to: GlobalTransform,
    pub interaction: TransformGizmoInteraction,
}

#[derive(Component, Default, Clone, Debug)]
pub struct GizmoTransformable;

#[derive(Component, Default, Clone, Debug)]
pub struct InternalGizmoCamera;

#[derive(Resource, Clone, Debug)]
pub struct GizmoSettings {
    pub enabled: bool,
    /// Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    /// coordinate system.
    pub alignment_rotation: Quat,
    pub allow_rotation: bool,
}

#[derive(Default, Debug, Clone)]
pub struct TransformGizmoPlugin {
    // Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    // coordinate system.
    alignment_rotation: Quat,
}

impl TransformGizmoPlugin {
    pub fn new(alignment_rotation: Quat) -> Self {
        TransformGizmoPlugin { alignment_rotation }
    }
}

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            gizmo_material::GIZMO_SHADER_HANDLE,
            "gizmo_material.wgsl",
            Shader::from_wgsl
        );

        let alignment_rotation = self.alignment_rotation;
        app.insert_resource(GizmoSettings {
            enabled: true,
            alignment_rotation,
            allow_rotation: true,
        })
        .insert_resource(GizmoSystemsEnabled(true))
        .add_plugins((
            MaterialPlugin::<GizmoMaterial>::default(),
            picking::GizmoPickingPlugin,
            Ui3dNormalization,
        ))
        .add_event::<TransformGizmoEvent>();

        // Input Set
        app.add_systems(
            PreUpdate,
            (
                update_gizmo_settings.in_set(TransformGizmoSystem::UpdateSettings),
                hover_gizmo
                    .in_set(TransformGizmoSystem::Hover)
                    .in_set(PickSet::Backend)
                    .after(RaycastSystem::UpdateRaycast::<GizmoRaycastSet>),
                grab_gizmo
                    .in_set(TransformGizmoSystem::Grab)
                    .after(PickSet::Focus),
            )
                .chain()
                .in_set(TransformGizmoSystem::InputsSet)
                .run_if(|settings: Res<GizmoSettings>| settings.enabled),
        );

        // Main Set
        app.add_systems(
            PostUpdate,
            (
                drag_gizmo
                    .in_set(TransformGizmoSystem::Drag)
                    .before(TransformSystem::TransformPropagate),
                place_gizmo
                    .in_set(TransformGizmoSystem::Place)
                    .after(TransformSystem::TransformPropagate),
                propagate_gizmo_elements,
                adjust_view_translate_gizmo.in_set(TransformGizmoSystem::Drag),
                gizmo_cam_copy_settings.in_set(TransformGizmoSystem::Drag),
            )
                .chain()
                .in_set(TransformGizmoSystem::MainSet)
                .run_if(|settings: Res<GizmoSettings>| settings.enabled),
        );

        app.add_systems(Startup, mesh::build_gizmo)
            .add_systems(PostStartup, place_gizmo);
    }
}

#[derive(Bundle)]
pub struct TransformGizmoBundle {
    gizmo: TransformGizmo,
    picking_interaction: PickingInteraction,
    picking_blocker: NoDeselect,
    transform: Transform,
    global_transform: GlobalTransform,
    visible: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    normalize: Normalize3d,
}

impl Default for TransformGizmoBundle {
    fn default() -> Self {
        TransformGizmoBundle {
            transform: Transform::from_translation(Vec3::splat(f32::MIN)),
            picking_interaction: PickingInteraction::None,
            picking_blocker: NoDeselect,
            visible: Visibility::Hidden,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            gizmo: TransformGizmo::default(),
            global_transform: GlobalTransform::default(),
            normalize: Normalize3d::new(1.5, 150.0),
        }
    }
}

#[derive(Default, PartialEq, Component)]
pub struct TransformGizmo {
    current_interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
    origin_drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Option<GlobalTransform>,
}

impl TransformGizmo {
    /// Get the gizmo's drag direction.
    pub fn current_interaction(&self) -> Option<TransformGizmoInteraction> {
        self.current_interaction
    }
}

/// Marks the current active gizmo interaction
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub enum TransformGizmoInteraction {
    TranslateAxis { original: Vec3, axis: Vec3 },
    TranslatePlane { original: Vec3, normal: Vec3 },
    RotateAxis { original: Vec3, axis: Vec3 },
    ScaleAxis { original: Vec3, axis: Vec3 },
}

#[derive(Component)]
struct InitialTransform {
    transform: Transform,
    rotation_offset: Vec3,
}

/// Updates the position of the gizmo and selected meshes while the gizmo is being dragged.
#[allow(clippy::type_complexity)]
fn drag_gizmo(
    pick_cam: Query<&GizmoPickSource>,
    mut gizmo_mut: Query<&mut TransformGizmo>,
    mut transform_query: Query<
        (
            &PickSelection,
            Option<&Parent>,
            &mut Transform,
            &InitialTransform,
        ),
        Without<TransformGizmo>,
    >,
    parent_query: Query<&GlobalTransform>,
    gizmo_query: Query<(&GlobalTransform, &PickingInteraction), With<TransformGizmo>>,
) {
    let picking_camera = if let Some(cam) = pick_cam.iter().last() {
        cam
    } else {
        // Not exactly one picking camera.
        return;
    };
    let picking_ray = if let Some(ray) = picking_camera.get_ray() {
        ray
    } else {
        // Picking camera does not have a ray.
        return;
    };
    // Gizmo handle should project mouse motion onto the axis of the handle. Perpendicular motion
    // should have no effect on the handle. We can do this by projecting the vector from the handle
    // click point to mouse's current position, onto the axis of the direction we are dragging. See
    // the wiki article for details: https://en.wikipedia.org/wiki/Vector_projection
    let gizmo_transform =
        if let Ok((transform, &PickingInteraction::Pressed)) = gizmo_query.get_single() {
            transform.to_owned()
        } else {
            return;
        };
    let mut gizmo = if let Ok(g) = gizmo_mut.get_single_mut() {
        g
    } else {
        error!("Number of transform gizmos is != 1");
        return;
    };
    let gizmo_origin = match gizmo.origin_drag_start {
        Some(origin) => origin,
        None => {
            let origin = gizmo_transform.translation();
            gizmo.origin_drag_start = Some(origin);
            origin
        }
    };
    let selected_iter = transform_query
        .iter_mut()
        .filter(|(s, ..)| s.is_selected)
        .map(|(_, parent, local_transform, initial_global_transform)| {
            let parent_global_transform = match parent {
                Some(parent) => match parent_query.get(parent.get()) {
                    Ok(transform) => *transform,
                    Err(_) => GlobalTransform::IDENTITY,
                },
                None => GlobalTransform::IDENTITY,
            };
            let parent_mat = parent_global_transform.compute_matrix();
            let inverse_parent = parent_mat.inverse();
            (inverse_parent, local_transform, initial_global_transform)
        });
    if let Some(interaction) = gizmo.current_interaction {
        if gizmo.initial_transform.is_none() {
            gizmo.initial_transform = Some(gizmo_transform);
        }
        match interaction {
            TransformGizmoInteraction::TranslateAxis { original: _, axis } => {
                let vertical_vector = picking_ray.direction.cross(axis).normalize();
                let plane_normal = axis.cross(vertical_vector).normalize();
                let plane_origin = gizmo_origin;
                let cursor_plane_intersection = if let Some(intesect_point) = picking_camera
                    .get_ray()
                    .and_then(|ray| intersect_plane(ray, plane_normal, plane_origin))
                {
                    intesect_point
                } else {
                    return;
                };

                let cursor_vector: Vec3 = cursor_plane_intersection - plane_origin;
                let cursor_projected_onto_handle = match &gizmo.drag_start {
                    Some(drag_start) => *drag_start,
                    None => {
                        let handle_vector = axis;
                        let cursor_projected_onto_handle = cursor_vector
                            .dot(handle_vector.normalize())
                            * handle_vector.normalize();
                        gizmo.drag_start = Some(cursor_projected_onto_handle + plane_origin);
                        return;
                    }
                };
                let selected_handle_vec = cursor_projected_onto_handle - plane_origin;
                let new_handle_vec = cursor_vector.dot(selected_handle_vec.normalize())
                    * selected_handle_vec.normalize();
                let translation = new_handle_vec - selected_handle_vec;
                selected_iter.for_each(
                    |(inverse_parent, mut local_transform, initial_global_transform)| {
                        let new_transform = Transform {
                            translation: initial_global_transform.transform.translation
                                + translation,
                            rotation: initial_global_transform.transform.rotation,
                            scale: initial_global_transform.transform.scale,
                        };
                        let local = inverse_parent * new_transform.compute_matrix();
                        local_transform.set_if_neq(Transform::from_matrix(local));
                    },
                );
            }
            TransformGizmoInteraction::TranslatePlane { normal, .. } => {
                let plane_origin = gizmo_origin;
                let cursor_plane_intersection = if let Some(intersection) = picking_camera
                    .get_ray()
                    .and_then(|ray| intersect_plane(ray, normal, plane_origin))
                {
                    intersection
                } else {
                    return;
                };
                let drag_start = match gizmo.drag_start {
                    Some(drag_start) => drag_start,
                    None => {
                        gizmo.drag_start = Some(cursor_plane_intersection);
                        return;
                    }
                };
                selected_iter.for_each(
                    |(inverse_parent, mut local_transform, initial_transform)| {
                        let new_transform = Transform {
                            translation: initial_transform.transform.translation
                                + cursor_plane_intersection
                                - drag_start,
                            rotation: initial_transform.transform.rotation,
                            scale: initial_transform.transform.scale,
                        };
                        let local = inverse_parent * new_transform.compute_matrix();
                        local_transform.set_if_neq(Transform::from_matrix(local));
                    },
                );
            }
            TransformGizmoInteraction::RotateAxis { original: _, axis } => {
                let cursor_plane_intersection = if let Some(intersection) = picking_camera
                    .get_ray()
                    .and_then(|ray| intersect_plane(ray, axis.normalize(), gizmo_origin))
                {
                    intersection
                } else {
                    return;
                };
                let cursor_vector = (cursor_plane_intersection - gizmo_origin).normalize();
                let drag_start = match &gizmo.drag_start {
                    Some(drag_start) => *drag_start,
                    None => {
                        gizmo.drag_start = Some(cursor_vector);
                        return; // We just started dragging, no transformation is needed yet, exit early.
                    }
                };
                let dot = drag_start.dot(cursor_vector);
                let det = axis.dot(drag_start.cross(cursor_vector));
                let angle = det.atan2(dot);
                let rotation = Quat::from_axis_angle(axis, angle);
                selected_iter.for_each(
                    |(inverse_parent, mut local_transform, initial_transform)| {
                        let world_space_offset = initial_transform.transform.rotation
                            * initial_transform.rotation_offset;
                        let offset_rotated = rotation * world_space_offset;
                        let offset = world_space_offset - offset_rotated;
                        let new_transform = Transform {
                            translation: initial_transform.transform.translation + offset,
                            rotation: rotation * initial_transform.transform.rotation,
                            scale: initial_transform.transform.scale,
                        };
                        let local = inverse_parent * new_transform.compute_matrix();
                        local_transform.set_if_neq(Transform::from_matrix(local));
                    },
                );
            }
            TransformGizmoInteraction::ScaleAxis {
                original: _,
                axis: _,
            } => (),
        }
    }
}

fn intersect_plane(ray: Ray3d, plane_normal: Vec3, plane_origin: Vec3) -> Option<Vec3> {
    // assuming vectors are all normalized
    let denominator = ray.direction.dot(plane_normal);
    if denominator.abs() > f32::EPSILON {
        let point_to_point = plane_origin - ray.origin;
        let intersect_dist = plane_normal.dot(point_to_point) / denominator;
        let intersect_position = ray.direction * intersect_dist + ray.origin;
        Some(intersect_position)
    } else {
        None
    }
}

fn hover_gizmo(
    gizmo_raycast_source: Query<(Entity, &GizmoPickSource)>,
    mut gizmo_query: Query<(
        Entity,
        &Children,
        &mut TransformGizmo,
        &mut PickingInteraction,
        &Transform,
    )>,
    hover_query: Query<&TransformGizmoInteraction>,
    mut hits: EventWriter<PointerHits>,
) {
    for (gizmo_entity, children, mut gizmo, mut interaction, _transform) in gizmo_query.iter_mut() {
        let Ok((camera, gizmo_raycast_source)) = gizmo_raycast_source.get_single() else {
            warn!("There must be exactly one gizmo raycast source");
            return;
        };

        if let Some((topmost_gizmo_entity, _)) = gizmo_raycast_source.get_nearest_intersection() {
            // Only update the gizmo state if it isn't being clicked (dragged) currently.
            if *interaction != PickingInteraction::Pressed {
                for child in children
                    .iter()
                    .filter(|entity| **entity == topmost_gizmo_entity)
                {
                    *interaction = PickingInteraction::Hovered;
                    if let Ok(gizmo_interaction) = hover_query.get(*child) {
                        gizmo.current_interaction = Some(*gizmo_interaction);
                    }
                }
            }
        } else if *interaction == PickingInteraction::Hovered {
            *interaction = PickingInteraction::None
        }

        if !matches!(*interaction, PickingInteraction::None) {
            // Tell picking backend we're hovering the gizmo, so the `NoDeselect` component takes effect.
            let data = HitData {
                camera,
                depth: 0.,
                position: None,
                normal: None,
            };
            hits.send(PointerHits {
                pointer: PointerId::Mouse,
                picks: vec![(gizmo_entity, data)],
                order: 1000.0,
            });
        }
    }
}

#[derive(Component)]
pub struct RotationOriginOffset(pub Vec3);

/// Tracks when one of the gizmo handles has been clicked on.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn grab_gizmo(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut gizmo_events: EventWriter<TransformGizmoEvent>,
    mut gizmo_query: Query<(
        &mut TransformGizmo,
        &mut PickingInteraction,
        &GlobalTransform,
    )>,
    selected_items_query: Query<(
        &PickSelection,
        &GlobalTransform,
        Entity,
        Option<&RotationOriginOffset>,
    )>,
    initial_transform_query: Query<Entity, With<InitialTransform>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (mut gizmo, interaction, _transform) in gizmo_query.iter_mut() {
            if *interaction == PickingInteraction::Pressed {
                // Dragging has started, store the initial position of all selected meshes
                for (selection, transform, entity, rotation_origin_offset) in
                    selected_items_query.iter()
                {
                    if selection.is_selected {
                        commands.entity(entity).insert(InitialTransform {
                            transform: transform.compute_transform(),
                            rotation_offset: rotation_origin_offset
                                .map(|offset| offset.0)
                                .unwrap_or(Vec3::ZERO),
                        });
                    }
                }
            } else {
                *gizmo = TransformGizmo::default();
                for entity in initial_transform_query.iter() {
                    commands.entity(entity).remove::<InitialTransform>();
                }
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (mut gizmo, mut interaction, transform) in gizmo_query.iter_mut() {
            *interaction = PickingInteraction::None;
            if let (Some(from), Some(interaction)) =
                (gizmo.initial_transform, gizmo.current_interaction())
            {
                let event = TransformGizmoEvent {
                    from,
                    to: *transform,
                    interaction,
                };
                //info!("{:?}", event);
                gizmo_events.send(event);
                *gizmo = TransformGizmo::default();
            }
        }
    }
}

/// Places the gizmo in space relative to the selected entity(s).
#[allow(clippy::type_complexity)]
fn place_gizmo(
    plugin_settings: Res<GizmoSettings>,
    mut queries: ParamSet<(
        Query<
            (
                &PickSelection,
                &GlobalTransform,
                Option<&RotationOriginOffset>,
            ),
            With<GizmoTransformable>,
        >,
        Query<(&mut GlobalTransform, &mut Transform, &mut Visibility), With<TransformGizmo>>,
    )>,
) {
    let selected: Vec<_> = queries
        .p0()
        .iter()
        .filter(|(s, ..)| s.is_selected)
        .map(|(_s, t, offset)| {
            t.translation()
                + offset
                    .map(|o| t.compute_transform().rotation * o.0)
                    .unwrap_or(Vec3::ZERO)
        })
        .collect();
    let n_selected = selected.len();
    let transform_sum = selected.iter().fold(Vec3::ZERO, |acc, t| acc + *t);
    let centroid = transform_sum / n_selected as f32;
    // Set the gizmo's position and visibility
    if let Ok((mut g_transform, mut transform, mut visible)) = queries.p1().get_single_mut() {
        let gt = g_transform.compute_transform();
        *g_transform = Transform {
            translation: centroid,
            rotation: plugin_settings.alignment_rotation,
            ..gt
        }
        .into();
        transform.translation = centroid;
        transform.rotation = plugin_settings.alignment_rotation;
        if n_selected > 0 {
            *visible = Visibility::Inherited;
        } else {
            *visible = Visibility::Hidden;
        }
    } else {
        error!("Number of gizmos is != 1");
    }
}

fn propagate_gizmo_elements(
    gizmo: Query<(&GlobalTransform, &Children), With<TransformGizmo>>,
    mut gizmo_parts_query: Query<(&Transform, &mut GlobalTransform), Without<TransformGizmo>>,
) {
    if let Ok((gizmo_pos, gizmo_parts)) = gizmo.get_single() {
        for &entity in gizmo_parts.iter() {
            let (transform, mut g_transform) = gizmo_parts_query.get_mut(entity).unwrap();
            *g_transform = gizmo_pos.mul_transform(*transform);
        }
    }
}

fn update_gizmo_settings(
    plugin_settings: Res<GizmoSettings>,
    mut interactions: Query<&mut TransformGizmoInteraction, Without<ViewTranslateGizmo>>,
    mut rotations: Query<&mut Visibility, With<RotationGizmo>>,
) {
    if !plugin_settings.is_changed() {
        return;
    }
    let rotation = plugin_settings.alignment_rotation;
    for mut interaction in interactions.iter_mut() {
        if let Some(rotated_interaction) = match *interaction {
            TransformGizmoInteraction::TranslateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::TranslateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            TransformGizmoInteraction::TranslatePlane {
                original,
                normal: _,
            } => Some(TransformGizmoInteraction::TranslatePlane {
                original,
                normal: rotation.mul_vec3(original),
            }),
            TransformGizmoInteraction::RotateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::RotateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            TransformGizmoInteraction::ScaleAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::ScaleAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
        } {
            *interaction = rotated_interaction;
        }
    }

    for mut visibility in rotations.iter_mut() {
        if plugin_settings.allow_rotation {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[allow(clippy::type_complexity)]
fn adjust_view_translate_gizmo(
    mut gizmo: Query<
        (&mut GlobalTransform, &mut TransformGizmoInteraction),
        (With<ViewTranslateGizmo>, Without<GizmoPickSource>),
    >,
    camera: Query<&Transform, With<GizmoPickSource>>,
) {
    let (mut global_transform, mut interaction) = match gizmo.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    let cam_transform = match camera.get_single() {
        Ok(x) => x,
        Err(_) => return,
    };

    let direction = cam_transform.local_z();
    *interaction = TransformGizmoInteraction::TranslatePlane {
        original: Vec3::ZERO,
        normal: *direction,
    };
    let rotation = Quat::from_mat3(&Mat3::from_cols(
        direction.cross(*cam_transform.local_y()),
        *direction,
        *cam_transform.local_y(),
    ));
    *global_transform = Transform {
        rotation,
        ..global_transform.compute_transform()
    }
    .into();
}

fn gizmo_cam_copy_settings(
    main_cam: Query<(Ref<Camera>, Ref<GlobalTransform>, Ref<Projection>), With<GizmoPickSource>>,
    mut gizmo_cam: Query<
        (&mut Camera, &mut GlobalTransform, &mut Projection),
        (With<InternalGizmoCamera>, Without<GizmoPickSource>),
    >,
) {
    let (main_cam, main_cam_pos, main_proj) = if let Ok(x) = main_cam.get_single() {
        x
    } else {
        error!("No `GizmoPickSource` found! Insert the `GizmoPickSource` component onto your primary 3d camera");
        return;
    };
    let (mut gizmo_cam, mut gizmo_cam_pos, mut proj) = gizmo_cam.single_mut();
    if main_cam_pos.is_changed() {
        *gizmo_cam_pos = *main_cam_pos;
    }
    if main_cam.is_changed() {
        *gizmo_cam = main_cam.clone();
        gizmo_cam.order += 10;
    }
    if main_proj.is_changed() {
        *proj = main_proj.clone();
    }
}

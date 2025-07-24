//! Transform

use bevy::picking::{backend::ray::RayMap, pointer::PointerId};
use bevy::{prelude::*, render::camera::Projection, transform::TransformSystems};
use bevy_editor_core::prelude::SelectedEntity;
use mesh::{RotationGizmo, ViewTranslateGizmo};

use normalization::*;

mod mesh;
pub mod normalization;

/// Crate prelude.
pub mod prelude {
    pub use crate::{GizmoCamera, TransformGizmoPlugin};
}

/// Resource that controls whether the transform gizmo is active.
#[derive(Resource, Clone, Debug)]
pub struct GizmoSystemsEnabled(pub bool);

/// Set enum for the systems relating to transform gizmos.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum TransformGizmoSystems {
    InputsSet,
    MainSet,
    NormalizeSet,
    UpdateSettings,
    Place,
    Hover,
    Grab,
    Drag,
}

#[derive(Debug, Clone, Event, BufferedEvent)]
pub struct TransformGizmoEvent {
    pub from: GlobalTransform,
    pub to: GlobalTransform,
    pub interaction: InteractionKind,
}

#[derive(Component, Default, Clone, Debug)]
pub struct GizmoTransformable;

/// Marker component for the camera that displays the gizmo.
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
        let alignment_rotation = self.alignment_rotation;
        app.insert_resource(GizmoSettings {
            enabled: true,
            alignment_rotation,
            allow_rotation: true,
        })
        .insert_resource(GizmoSystemsEnabled(true))
        .add_plugins((Ui3dNormalization,))
        .add_event::<TransformGizmoEvent>()
        .add_observer(
            |trigger: On<Pointer<Press>>,
             target_query: Query<(&InteractionKind, &ChildOf)>,
             mut query: Query<&mut TransformGizmo>,
             selection: Res<SelectedEntity>,
             items_query: Query<(&GlobalTransform, Entity, Option<&RotationOriginOffset>)>,
             mut commands: Commands| {
                if trigger.button != PointerButton::Primary {
                    return;
                }
                let Ok((interaction, child_of)) = target_query.get(trigger.target()) else {
                    return;
                };

                // Activate the interaction.
                query.get_mut(child_of.parent()).unwrap().interaction =
                    Some(TransformGizmoInteraction {
                        kind: *interaction,
                        pointer_id: trigger.pointer_id,
                    });

                // Dragging has started, store the initial position of all selected meshes
                for (transform, entity, rotation_origin_offset) in items_query.iter() {
                    if selection.contains(entity) {
                        commands.entity(entity).insert(InitialTransform {
                            transform: transform.compute_transform(),
                            rotation_offset: rotation_origin_offset
                                .map(|offset| offset.0)
                                .unwrap_or(Vec3::ZERO),
                        });
                    }
                }
            },
        )
        .add_observer(
            |trigger: On<Pointer<Release>>,
             mut query: Query<(&mut TransformGizmo, &GlobalTransform)>,
             mut gizmo_events: EventWriter<TransformGizmoEvent>,
             mut commands: Commands,
             initial_transform_query: Query<Entity, With<InitialTransform>>| {
                if trigger.button != PointerButton::Primary {
                    return;
                }
                let (mut gizmo, transform) = query.single_mut().unwrap();

                if let (Some(from), Some(interaction)) =
                    (gizmo.initial_transform, gizmo.interaction)
                {
                    let event = TransformGizmoEvent {
                        from,
                        to: *transform,
                        interaction: interaction.kind,
                    };
                    gizmo_events.write(event);
                    *gizmo = TransformGizmo::default();
                }

                *gizmo = default();

                for entity in initial_transform_query.iter() {
                    commands.entity(entity).remove::<InitialTransform>();
                }
            },
        );

        // Input Set
        app.add_systems(
            PreUpdate,
            update_gizmo_settings
                .in_set(TransformGizmoSystems::UpdateSettings)
                .run_if(|settings: Res<GizmoSettings>| settings.enabled),
        );

        // Main Set
        app.add_systems(
            PostUpdate,
            (
                drag_gizmo
                    .in_set(TransformGizmoSystems::Drag)
                    .before(TransformSystems::Propagate),
                place_gizmo
                    .in_set(TransformGizmoSystems::Place)
                    .after(TransformSystems::Propagate),
                propagate_gizmo_elements,
                adjust_view_translate_gizmo.in_set(TransformGizmoSystems::Drag),
                gizmo_cam_copy_settings.in_set(TransformGizmoSystems::Drag),
            )
                .chain()
                .in_set(TransformGizmoSystems::MainSet)
                .run_if(|settings: Res<GizmoSettings>| settings.enabled),
        );

        app.add_systems(Startup, mesh::build_gizmo)
            .add_systems(PostStartup, place_gizmo);
    }
}

/// Component for the transform gizmo itself. Holds dat about the current interaction.
#[derive(Default, PartialEq, Component)]
#[require(Transform, Visibility::Hidden, Normalize3d::new(1.5, 150.0))]
pub struct TransformGizmo {
    /// The active gizmo interaction.
    interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
    origin_drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Option<GlobalTransform>,
}

impl TransformGizmo {
    /// Get the gizmo's ongoing interaction.
    pub fn interaction(&self) -> Option<TransformGizmoInteraction> {
        self.interaction
    }
}

/// Describes an ongoing transform gizmo interation.
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub struct TransformGizmoInteraction {
    /// The kind of interaction we are currently performing.
    kind: InteractionKind,
    /// The pointer that started this interaction.
    pointer_id: PointerId,
}

/// The kind of [`TransformGizmoInteraction`] that is happening.
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub enum InteractionKind {
    /// Translating along an axis.
    TranslateAxis {
        /// Starting position.
        original: Vec3,
        /// The axis were translating along.
        axis: Vec3,
    },
    /// Translating across a plane.
    TranslatePlane {
        /// Starting position.
        original: Vec3,
        /// The plane were translating across.
        normal: Vec3,
    },
    /// Rotating on an axis.
    RotateAxis {
        /// Starting direction.
        original: Vec3,
        /// The axis were rotating on.
        axis: Vec3,
    },
}

/// Stores the inital transform of entities involved in a [`TransformGizmoInteraction`].
#[derive(Component, Clone, Debug)]
struct InitialTransform {
    transform: Transform,
    rotation_offset: Vec3,
}

/// Marker component for the camera that display and control the transform gizmo.
#[derive(Component, Default, Clone, Debug)]
pub struct GizmoCamera;

/// Updates the position of the gizmo and selected meshes while the gizmo is being dragged.
#[allow(clippy::type_complexity)]
fn drag_gizmo(
    // pick_cam: Query<&GizmoCamera>,
    raymap: Res<RayMap>,
    selection: Res<SelectedEntity>,
    mut transform_query: Query<
        (
            Entity,
            // &PickSelection,
            Option<&ChildOf>,
            &mut Transform,
            &InitialTransform,
        ),
        Without<TransformGizmo>,
    >,
    parent_query: Query<&GlobalTransform>,
    mut gizmo_query: Query<(&mut TransformGizmo, &GlobalTransform)>,
) {
    // Gizmo handle should project mouse motion onto the axis of the handle. Perpendicular motion
    // should have no effect on the handle. We can do this by projecting the vector from the handle
    // click point to mouse's current position, onto the axis of the direction we are dragging. See
    // the wiki article for details: https://en.wikipedia.org/wiki/Vector_projection
    let Ok((mut gizmo, &gizmo_transform)) = gizmo_query.single_mut() else {
        return;
    };
    let Some(TransformGizmoInteraction { kind, pointer_id }) = gizmo.interaction else {
        return;
    };
    let Some((_, &picking_ray)) = raymap
        .iter()
        .filter(|(ray_id, _)| ray_id.pointer == pointer_id)
        .next()
    else {
        // Picking camera does not have a ray.
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
        .filter(|(entity, ..)| selection.contains(*entity))
        .map(|(_, parent, local_transform, initial_global_transform)| {
            let parent_global_transform = parent
                .and_then(|child_of| parent_query.get(child_of.parent()).ok())
                .unwrap_or(&GlobalTransform::IDENTITY);
            let parent_mat = parent_global_transform.to_matrix();
            let inverse_parent = parent_mat.inverse();
            (inverse_parent, local_transform, initial_global_transform)
        });
    if gizmo.initial_transform.is_none() {
        gizmo.initial_transform = Some(gizmo_transform);
    }
    match kind {
        InteractionKind::TranslateAxis { original: _, axis } => {
            let vertical_vector = picking_ray.direction.cross(axis).normalize();
            let plane_normal = axis.cross(vertical_vector).normalize();
            let plane_origin = gizmo_origin;
            let Some(cursor_plane_intersection) =
                intersect_plane(picking_ray, plane_normal, plane_origin)
            else {
                return;
            };

            let cursor_vector: Vec3 = cursor_plane_intersection - plane_origin;
            let cursor_projected_onto_handle = match &gizmo.drag_start {
                Some(drag_start) => *drag_start,
                None => {
                    let handle_vector = axis;
                    let cursor_projected_onto_handle =
                        cursor_vector.dot(handle_vector.normalize()) * handle_vector.normalize();
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
                        translation: initial_global_transform.transform.translation + translation,
                        rotation: initial_global_transform.transform.rotation,
                        scale: initial_global_transform.transform.scale,
                    };
                    let local = inverse_parent * new_transform.to_matrix();
                    local_transform.set_if_neq(Transform::from_matrix(local));
                },
            );
        }
        InteractionKind::TranslatePlane { normal, .. } => {
            let plane_origin = gizmo_origin;
            let Some(cursor_plane_intersection) =
                intersect_plane(picking_ray, normal, plane_origin)
            else {
                return;
            };
            let drag_start = match gizmo.drag_start {
                Some(drag_start) => drag_start,
                None => {
                    gizmo.drag_start = Some(cursor_plane_intersection);
                    return;
                }
            };
            selected_iter.for_each(|(inverse_parent, mut local_transform, initial_transform)| {
                let new_transform = Transform {
                    translation: initial_transform.transform.translation
                        + cursor_plane_intersection
                        - drag_start,
                    rotation: initial_transform.transform.rotation,
                    scale: initial_transform.transform.scale,
                };
                let local = inverse_parent * new_transform.to_matrix();
                local_transform.set_if_neq(Transform::from_matrix(local));
            });
        }
        InteractionKind::RotateAxis { original: _, axis } => {
            let Some(cursor_plane_intersection) =
                intersect_plane(picking_ray, axis.normalize(), gizmo_origin)
            else {
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
            selected_iter.for_each(|(inverse_parent, mut local_transform, initial_transform)| {
                let world_space_offset =
                    initial_transform.transform.rotation * initial_transform.rotation_offset;
                let offset_rotated = rotation * world_space_offset;
                let offset = world_space_offset - offset_rotated;
                let new_transform = Transform {
                    translation: initial_transform.transform.translation + offset,
                    rotation: rotation * initial_transform.transform.rotation,
                    scale: initial_transform.transform.scale,
                };
                let local = inverse_parent * new_transform.to_matrix();
                local_transform.set_if_neq(Transform::from_matrix(local));
            });
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

#[derive(Component)]
pub struct RotationOriginOffset(pub Vec3);

/// Places the gizmo in space relative to the selected entity(s).
#[allow(clippy::type_complexity)]
fn place_gizmo(
    plugin_settings: Res<GizmoSettings>,
    selection: Res<SelectedEntity>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                // &PickSelection,
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
        .filter(|(entity, ..)| selection.contains(*entity))
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
    if let Ok((mut g_transform, mut transform, mut visible)) = queries.p1().single_mut() {
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
    if let Ok((gizmo_pos, gizmo_parts)) = gizmo.single() {
        for entity in gizmo_parts.iter() {
            let (transform, mut g_transform) = gizmo_parts_query.get_mut(entity).unwrap();
            *g_transform = gizmo_pos.mul_transform(*transform);
        }
    }
}

fn update_gizmo_settings(
    plugin_settings: Res<GizmoSettings>,
    mut interactions: Query<&mut InteractionKind, Without<ViewTranslateGizmo>>,
    mut rotations: Query<&mut Visibility, With<RotationGizmo>>,
) {
    if !plugin_settings.is_changed() {
        return;
    }
    let rotation = plugin_settings.alignment_rotation;
    for mut interaction in interactions.iter_mut() {
        if let Some(rotated_interaction) = match *interaction {
            InteractionKind::TranslateAxis { original, axis: _ } => {
                Some(InteractionKind::TranslateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            InteractionKind::TranslatePlane {
                original,
                normal: _,
            } => Some(InteractionKind::TranslatePlane {
                original,
                normal: rotation.mul_vec3(original),
            }),
            InteractionKind::RotateAxis { original, axis: _ } => {
                Some(InteractionKind::RotateAxis {
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
        (&mut GlobalTransform, &mut InteractionKind),
        (With<ViewTranslateGizmo>, Without<GizmoCamera>),
    >,
    camera: Query<&Transform, With<GizmoCamera>>,
) {
    let (mut global_transform, mut interaction) = match gizmo.single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    let cam_transform = match camera.single() {
        Ok(x) => x,
        Err(_) => return,
    };

    let direction = cam_transform.local_z();
    *interaction = InteractionKind::TranslatePlane {
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
    main_cam: Query<(Ref<Camera>, Ref<GlobalTransform>, Ref<Projection>), With<GizmoCamera>>,
    mut gizmo_cam: Query<
        (&mut Camera, &mut GlobalTransform, &mut Projection),
        (With<InternalGizmoCamera>, Without<GizmoCamera>),
    >,
) {
    let (main_cam, main_cam_pos, main_proj) = if let Ok(x) = main_cam.single() {
        x
    } else {
        error!("No `GizmoPickSource` found! Insert the `GizmoPickSource` component onto your primary 3d camera");
        return;
    };
    let (mut gizmo_cam, mut gizmo_cam_pos, mut proj) = gizmo_cam.single_mut().unwrap();
    if main_cam_pos.is_changed() {
        *gizmo_cam_pos = *main_cam_pos;
    }
    if main_cam.is_changed() {
        *gizmo_cam = main_cam.clone();
        gizmo_cam.order += 1;
    }
    if main_proj.is_changed() {
        *proj = main_proj.clone();
    }
}

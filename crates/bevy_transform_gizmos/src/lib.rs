//! A crate for transform gizmos. Transform gizmos are UI elements that
//! allow you to manipulate the transforms of entities.
//!
//! # Usage
//!
//! You must add the [`TransformGizmoPlugin`] to the app.
//!
//! Then you can add the [`GizmoCamera`] marker component to a camera,
//! and the [`GizmoTransformable`] marker to the entities you want to manipulate.
//!
//! Then, when these entities are selected via [`bevy_editor_core::selection`] the
//! transform gizmo will appear and allow you to move and rotate your selection.

use bevy::picking::{backend::ray::RayMap, pointer::PointerId};
use bevy::{prelude::*, render::camera::Projection, transform::TransformSystems};
use bevy_editor_core::selection::SelectedEntity;
use mesh::{RotationGizmo, ViewTranslateGizmo};

use normalization::*;

mod mesh;
pub mod normalization;

/// Crate prelude.
pub mod prelude {
    pub use crate::{GizmoCamera, TransformGizmoPlugin, TransformGizmoSettings};
}

/// Set enum for the systems relating to transform gizmos.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum TransformGizmoSystems {
    /// Input set.
    Input,
    /// Main set.
    Main,
    /// Normalize set.
    Normalize,
    /// Update settings set.
    UpdateSettings,
    /// Transform gizmo place set.
    Place,
    /// Transform gizmo drag set.
    Drag,
}

/// Event thats sent when a [`TransformGizmoInteraction`] finishes.
#[derive(Debug, Clone, Event, BufferedEvent)]
pub struct TransformGizmoEvent {
    /// The starting position of the gizmo before the interaction.
    pub from: GlobalTransform,
    /// The end position of the gizmo after the interaction.
    pub to: GlobalTransform,
    /// The kind of interaction that was performed.
    pub kind: InteractionKind,
}

/// Marker component for entities that can be transformed by the transform gizmo.
#[derive(Component, Default, Clone, Debug)]
pub struct GizmoTransformable;

/// Marker component for the camera that displays the gizmo.
#[derive(Component, Default, Clone, Debug)]
pub struct InternalGizmoCamera;

/// Settings for the [`TransformGizmoPlugin`].
#[derive(Resource, Clone, Debug)]
pub struct TransformGizmoSettings {
    /// Control whether the transform gizmo is active.
    pub enabled: bool,
    /// Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    /// coordinate system.
    pub alignment_rotation: Quat,
    /// Control whether the gizmo allows rotation.
    pub enable_rotation: bool,
}

impl Default for TransformGizmoSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            alignment_rotation: default(),
            enable_rotation: true,
        }
    }
}

/// The transform gizmo plugin.
#[derive(Default, Debug, Clone)]
pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MeshPickingPlugin>() {
            app.add_plugins(MeshPickingPlugin);
        }
        app.init_resource::<TransformGizmoSettings>()
            .add_plugins(Ui3dNormalizationPlugin)
            .add_event::<TransformGizmoEvent>()
            .add_observer(on_transform_gizmo_pointer_press)
            .add_observer(on_transform_gizmo_pointer_release);

        // Settings Set
        app.add_systems(
            PreUpdate,
            update_gizmo_settings
                .in_set(TransformGizmoSystems::UpdateSettings)
                .run_if(|settings: Res<TransformGizmoSettings>| settings.enabled),
        );

        // Main Set
        app.add_systems(
            PostUpdate,
            (
                (
                    drag_gizmo.before(TransformSystems::Propagate),
                    place_gizmo.after(TransformSystems::Propagate),
                )
                    .in_set(TransformGizmoSystems::Place),
                propagate_gizmo_elements,
                (adjust_view_translate_gizmo, gizmo_cam_copy_settings)
                    .chain()
                    .in_set(TransformGizmoSystems::Drag),
            )
                .chain()
                .in_set(TransformGizmoSystems::Main)
                .run_if(|settings: Res<TransformGizmoSettings>| settings.enabled),
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
    // Initial transform of the gizmo
    initial_transform: Option<GlobalTransform>,
}

impl TransformGizmo {
    /// Get the gizmo's ongoing interaction.
    pub fn interaction(&self) -> Option<TransformGizmoInteraction> {
        self.interaction
    }
}

/// Describes an ongoing transform gizmo interaction.
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub struct TransformGizmoInteraction {
    /// The kind of interaction we are currently performing.
    kind: InteractionKind,
    /// The pointer that started this interaction.
    pointer_id: PointerId,
    origin: Vec3,
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

/// Stores the initial transform of entities involved in a [`TransformGizmoInteraction`].
#[derive(Component, Clone, Debug)]
struct InitialTransform {
    transform: Transform,
    rotation_offset: Vec3,
}

/// Marker component for the camera that display and control the transform gizmo.
#[derive(Component, Default, Clone, Debug)]
pub struct GizmoCamera;

fn on_transform_gizmo_pointer_press(
    trigger: On<Pointer<Press>>,
    target_query: Query<(&InteractionKind, &ChildOf)>,
    mut query: Query<(&mut TransformGizmo, &GlobalTransform)>,
    selection: Res<SelectedEntity>,
    items_query: Query<(&GlobalTransform, Entity, Option<&TransformGizmoOffset>)>,
    mut commands: Commands,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }
    let Ok((interaction, child_of)) = target_query.get(trigger.target()) else {
        return;
    };
    let Ok((mut gizmo, transform)) = query.get_mut(child_of.parent()) else {
        return;
    };

    // Activate the interaction.
    gizmo.interaction = Some(TransformGizmoInteraction {
        kind: *interaction,
        pointer_id: trigger.pointer_id,
        origin: transform.translation(),
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
}

fn on_transform_gizmo_pointer_release(
    trigger: On<Pointer<Release>>,
    mut query: Query<(&mut TransformGizmo, &GlobalTransform)>,
    mut gizmo_events: EventWriter<TransformGizmoEvent>,
    mut commands: Commands,
    initial_transform_query: Query<Entity, With<InitialTransform>>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }
    let Ok((mut gizmo, transform)) = query.single_mut() else {
        return;
    };

    if let (Some(from), Some(interaction)) = (gizmo.initial_transform, gizmo.interaction) {
        let event = TransformGizmoEvent {
            from,
            to: *transform,
            kind: interaction.kind,
        };
        gizmo_events.write(event);
        *gizmo = TransformGizmo::default();
    }

    *gizmo = default();

    for entity in &initial_transform_query {
        commands.entity(entity).remove::<InitialTransform>();
    }
}

/// Updates the position of the gizmo and selected meshes while the gizmo is being dragged.
fn drag_gizmo(
    raymap: Res<RayMap>,
    selection: Res<SelectedEntity>,
    mut transform_query: Query<
        (Entity, Option<&ChildOf>, &mut Transform, &InitialTransform),
        Without<TransformGizmo>,
    >,
    parent_query: Query<&GlobalTransform>,
    mut gizmo_query: Query<(&mut TransformGizmo, &GlobalTransform)>,
) {
    // Gizmo handle should project mouse motion onto the axis of the handle. Perpendicular motion
    // should have no effect on the handle. We can do this by projecting the vector from the handle
    // click point to mouse's current position, onto the axis of the direction we are dragging. See
    // the wiki article for details: https://en.wikipedia.org/wiki/Vector_projection
    if let Ok((mut gizmo, &gizmo_transform)) = gizmo_query.single_mut()
        && let Some(TransformGizmoInteraction {
            kind,
            pointer_id,
            origin,
        }) = gizmo.interaction
        && let Some((_, &ray)) = raymap.iter().find(|(id, _)| id.pointer == pointer_id)
    {
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
                let vertical_vector = ray.direction.cross(axis).normalize();
                let plane_normal = axis.cross(vertical_vector).normalize();
                let Some(cursor_plane_intersection) = intersect_plane(ray, plane_normal, origin)
                else {
                    return;
                };

                let cursor_vector: Vec3 = cursor_plane_intersection - origin;
                let Some(cursor_projected_onto_handle) = &gizmo.drag_start else {
                    let handle_vector = axis;
                    let cursor_projected_onto_handle =
                        cursor_vector.dot(handle_vector.normalize()) * handle_vector.normalize();
                    gizmo.drag_start = Some(cursor_projected_onto_handle + origin);
                    return;
                };
                let selected_handle_vec = cursor_projected_onto_handle - origin;
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
                        let local = inverse_parent * new_transform.to_matrix();
                        local_transform.set_if_neq(Transform::from_matrix(local));
                    },
                );
            }
            InteractionKind::TranslatePlane { normal, .. } => {
                let Some(cursor_plane_intersection) = intersect_plane(ray, normal, origin) else {
                    return;
                };
                let Some(drag_start) = gizmo.drag_start else {
                    gizmo.drag_start = Some(cursor_plane_intersection);
                    return;
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
                        let local = inverse_parent * new_transform.to_matrix();
                        local_transform.set_if_neq(Transform::from_matrix(local));
                    },
                );
            }
            InteractionKind::RotateAxis { original: _, axis } => {
                let Some(cursor_plane_intersection) =
                    intersect_plane(ray, axis.normalize(), origin)
                else {
                    return;
                };
                let cursor_vector = (cursor_plane_intersection - origin).normalize();
                let Some(drag_start) = &gizmo.drag_start else {
                    gizmo.drag_start = Some(cursor_vector);
                    return;
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
                        let local = inverse_parent * new_transform.to_matrix();
                        local_transform.set_if_neq(Transform::from_matrix(local));
                    },
                );
            }
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

/// Offsets where the origin is for an entity transformed by the transform gizmo.
#[derive(Component)]
pub struct TransformGizmoOffset(pub Vec3);

/// Places the gizmo in space relative to the selected entity(s).
fn place_gizmo(
    plugin_settings: Res<TransformGizmoSettings>,
    selection: Res<SelectedEntity>,
    mut queries: ParamSet<(
        Query<(Entity, &GlobalTransform, Option<&TransformGizmoOffset>), With<GizmoTransformable>>,
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
        if n_selected > 0 {
            let gt = g_transform.compute_transform();
            *g_transform = Transform {
                translation: centroid,
                rotation: plugin_settings.alignment_rotation,
                ..gt
            }
            .into();
            transform.translation = centroid;
            transform.rotation = plugin_settings.alignment_rotation;
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
            let Ok((transform, mut g_transform)) = gizmo_parts_query.get_mut(entity) else {
                error!("Malformed transform gizmo");
                continue;
            };
            *g_transform = gizmo_pos.mul_transform(*transform);
        }
    }
}

fn update_gizmo_settings(
    plugin_settings: Res<TransformGizmoSettings>,
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
        if plugin_settings.enable_rotation {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn adjust_view_translate_gizmo(
    mut gizmo: Query<
        (&mut GlobalTransform, &mut InteractionKind),
        (With<ViewTranslateGizmo>, Without<GizmoCamera>),
    >,
    camera: Query<&Transform, With<GizmoCamera>>,
) {
    if let Ok((mut global_transform, mut interaction)) = gizmo.single_mut()
        && let Ok(cam_transform) = camera.single()
    {
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
}

fn gizmo_cam_copy_settings(
    main_cam: Query<(Ref<Camera>, Ref<GlobalTransform>, Ref<Projection>), With<GizmoCamera>>,
    mut gizmo_cam: Query<
        (&mut Camera, &mut GlobalTransform, &mut Projection),
        (With<InternalGizmoCamera>, Without<GizmoCamera>),
    >,
) {
    if let Ok((main_cam, main_cam_pos, main_proj)) = main_cam.single()
        && let Ok((mut gizmo_cam, mut gizmo_cam_pos, mut proj)) = gizmo_cam.single_mut()
    {
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
}

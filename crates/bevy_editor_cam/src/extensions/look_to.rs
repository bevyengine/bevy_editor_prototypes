//! A `bevy_editor_cam` extension that adds the ability to smoothly rotate the camera about its
//! anchor point until it is looking in the specified direction.

use std::{f32::consts::PI, time::Duration};

use bevy::app::prelude::*;
use bevy::ecs::prelude::*;
use bevy::math::{DQuat, DVec3, prelude::*};
use bevy::platform::collections::HashMap;
use bevy::platform::time::Instant;
use bevy::reflect::prelude::*;
use bevy::transform::prelude::*;
use bevy::window::RequestRedraw;

use crate::prelude::*;

/// See the [module](self) docs.
pub struct LookToPlugin;

impl Plugin for LookToPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LookTo>()
            .add_message::<LookToTrigger>()
            .add_systems(
                PreUpdate,
                LookTo::update.before(EditorCam::update_camera_positions),
            )
            .add_systems(PostUpdate, LookToTrigger::receive); // In PostUpdate so we don't miss users sending this in Update. LookTo::update will catch the changes next frame.
    }
}

/// Send this event to rotate the camera about its anchor until it is looking in the given direction
/// with the given up direction. Animation speed is configured with the [`LookTo`] resource.
#[derive(Debug, Event, Message)]
pub struct LookToTrigger {
    /// The new direction to face.
    pub target_facing_direction: Dir3,
    /// The camera's "up" direction when finished moving.
    pub target_up_direction: Dir3,
    /// The camera to update.
    pub camera: Entity,
}

impl LookToTrigger {
    /// Constructs a [`LookToTrigger`] with the up direction automatically selected.
    ///
    /// If the camera is set to [`OrbitConstraint::Fixed`], the fixed up direction will be used, as
    /// long as it is not parallel to the facing direction. If set to [`OrbitConstraint::Free`] or
    /// the facing direction is parallel to the fixed up direction, the up direction will be
    /// automatically selected by choosing the axis that results in the least amount of rotation.
    pub fn auto_snap_up_direction(
        facing: Dir3,
        cam_entity: Entity,
        cam_transform: &Transform,
        cam_editor: &EditorCam,
    ) -> Self {
        const EPSILON: f32 = 0.01;
        let constraint = match cam_editor.orbit_constraint {
            OrbitConstraint::Fixed { up, .. } => Some(up),
            OrbitConstraint::Free => None,
        }
        .filter(|up| {
            let angle = facing.angle_between(*up).abs();
            angle > EPSILON && angle < PI - EPSILON
        });

        let up = constraint.unwrap_or_else(|| {
            let current = cam_transform.rotation;
            let options = [
                Vec3::X,
                Vec3::NEG_X,
                Vec3::Y,
                Vec3::NEG_Y,
                Vec3::Z,
                Vec3::NEG_Z,
            ];
            *options
                .iter()
                .map(|d| (d, Transform::default().looking_to(*facing, *d).rotation))
                .map(|(d, rot)| (d, rot.angle_between(current).abs()))
                .reduce(|acc, this| if this.1 < acc.1 { this } else { acc })
                .map(|nearest| nearest.0)
                .unwrap_or(&Vec3::Y)
        });

        LookToTrigger {
            target_facing_direction: facing,
            target_up_direction: Dir3::new_unchecked(up.normalize()),
            camera: cam_entity,
        }
    }
}

impl LookToTrigger {
    fn receive(
        mut events: MessageReader<Self>,
        mut state: ResMut<LookTo>,
        mut cameras: Query<(&mut EditorCam, &Transform)>,
        mut redraw: MessageWriter<RequestRedraw>,
    ) {
        for event in events.read() {
            let Ok((mut controller, transform)) = cameras.get_mut(event.camera) else {
                continue;
            };
            redraw.write(RequestRedraw);

            state
                .map
                .entry(event.camera)
                .and_modify(|e| {
                    e.start = Instant::now();
                    e.initial_facing_direction = transform.forward();
                    e.initial_up_direction = transform.up();
                    e.target_facing_direction = event.target_facing_direction;
                    e.target_up_direction = event.target_up_direction;
                    e.complete = false;
                })
                .or_insert(LookToEntry {
                    start: Instant::now(),
                    initial_facing_direction: transform.forward(),
                    initial_up_direction: transform.up(),
                    target_facing_direction: event.target_facing_direction,
                    target_up_direction: event.target_up_direction,
                    complete: false,
                });

            controller.end_move();
            controller.current_motion = motion::CurrentMotion::Stationary;
        }
    }
}

struct LookToEntry {
    start: Instant,
    initial_facing_direction: Dir3,
    initial_up_direction: Dir3,
    target_facing_direction: Dir3,
    target_up_direction: Dir3,
    complete: bool,
}

/// Stores settings and state for the dolly zoom plugin.
#[derive(Resource, Reflect)]
pub struct LookTo {
    /// The duration of the "look to" transition animation.
    pub animation_duration: Duration,
    /// The cubic curve used to animate the camera during a "look to".
    #[reflect(ignore)]
    pub animation_curve: CubicSegment<Vec2>,
    #[reflect(ignore)]
    map: HashMap<Entity, LookToEntry>,
}

impl Default for LookTo {
    fn default() -> Self {
        Self {
            animation_duration: Duration::from_millis(400),
            animation_curve: CubicSegment::new_bezier_easing((0.25, 0.0), (0.25, 1.0)),
            map: Default::default(),
        }
    }
}

impl LookTo {
    fn update(
        mut state: ResMut<Self>,
        mut cameras: Query<(&mut Transform, &EditorCam)>,
        mut redraw: MessageWriter<RequestRedraw>,
    ) {
        let animation_duration = state.animation_duration;
        let animation_curve = state.animation_curve;
        for (
            camera,
            LookToEntry {
                start,
                initial_facing_direction,
                initial_up_direction,
                target_facing_direction,
                target_up_direction,
                complete,
            },
        ) in state.map.iter_mut()
        {
            let Ok((mut transform, controller)) = cameras.get_mut(*camera) else {
                *complete = true;
                continue;
            };
            let progress_t =
                (start.elapsed().as_secs_f32() / animation_duration.as_secs_f32()).clamp(0.0, 1.0);
            let progress = animation_curve.ease(progress_t);

            let rotate_around = |transform: &mut Transform, point: DVec3, rotation: DQuat| {
                // Following lines are f64 versions of Transform::rotate_around
                transform.translation =
                    (point + rotation * (transform.translation.as_dvec3() - point)).as_vec3();
                transform.rotation = (rotation * transform.rotation.as_dquat())
                    .as_quat()
                    .normalize();
            };

            let anchor_view_space = controller.anchor_view_space().unwrap_or(DVec3::new(
                0.0,
                0.0,
                controller.last_anchor_depth(),
            ));

            let anchor_world = {
                let (r, t) = (transform.rotation, transform.translation);
                r.as_dquat() * anchor_view_space + t.as_dvec3()
            };

            let rot_init = Transform::default()
                .looking_to(**initial_facing_direction, **initial_up_direction)
                .rotation;
            let rot_target = Transform::default()
                .looking_to(**target_facing_direction, **target_up_direction)
                .rotation;

            let rot_next = rot_init.slerp(rot_target, progress);
            let rot_last = transform.rotation;
            let rot_delta = rot_next * rot_last.inverse();

            rotate_around(&mut transform, anchor_world, rot_delta.as_dquat());

            if progress_t >= 1.0 {
                *complete = true;
            }
            redraw.write(RequestRedraw);
        }
        state.map.retain(|_, v| !v.complete);
    }
}

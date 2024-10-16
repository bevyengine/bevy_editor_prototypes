use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use camera_controller::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraControllerPlugin, InfiniteGridPlugin))
        .add_systems(Startup, setup_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(InfiniteGridBundle::default());

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 4.37, 14.77),
            ..default()
        },
        CameraController::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 15. + Vec3::Y * 20.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let mat = standard_materials.add(StandardMaterial::default());

    // cube
    commands.spawn(PbrBundle {
        material: mat.clone(),
        mesh: meshes.add(Cuboid::from_size(Vec3::ONE).mesh()),
        transform: Transform {
            translation: Vec3::new(3., 4., 0.),
            rotation: Quat::from_rotation_arc(Vec3::Y, Vec3::ONE.normalize()),
            scale: Vec3::splat(1.5),
        },
        ..default()
    });

    commands.spawn(PbrBundle {
        material: mat.clone(),
        mesh: meshes.add(Cuboid::from_size(Vec3::ONE).mesh()),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });
}

// This is a simplified version of the camera controller used in bevy examples
mod camera_controller {
    use bevy::{input::mouse::MouseMotion, prelude::*};

    use std::f32::consts::*;

    pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

    #[derive(Component)]
    pub struct CameraController {
        pub pitch: f32,
        pub yaw: f32,
        pub velocity: Vec3,
    }

    impl Default for CameraController {
        fn default() -> Self {
            Self {
                pitch: 0.0,
                yaw: 0.0,
                velocity: Vec3::ZERO,
            }
        }
    }

    pub struct CameraControllerPlugin;

    impl Plugin for CameraControllerPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, camera_controller);
        }
    }

    fn camera_controller(
        time: Res<Time>,
        mut mouse_events: EventReader<MouseMotion>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        key_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
    ) {
        let dt = time.delta_seconds();

        if let Ok((mut transform, mut state)) = query.get_single_mut() {
            // Handle key input
            let mut axis_input = Vec3::ZERO;
            if key_input.pressed(KeyCode::KeyW) {
                axis_input.z += 1.0;
            }
            if key_input.pressed(KeyCode::KeyS) {
                axis_input.z -= 1.0;
            }
            if key_input.pressed(KeyCode::KeyD) {
                axis_input.x += 1.0;
            }
            if key_input.pressed(KeyCode::KeyA) {
                axis_input.x -= 1.0;
            }
            if key_input.pressed(KeyCode::KeyE) {
                axis_input.y += 1.0;
            }
            if key_input.pressed(KeyCode::KeyQ) {
                axis_input.y -= 1.0;
            }

            // Apply movement update
            if axis_input != Vec3::ZERO {
                let max_speed = if key_input.pressed(KeyCode::ShiftLeft) {
                    15.0
                } else {
                    5.0
                };
                state.velocity = axis_input.normalize() * max_speed;
            } else {
                state.velocity *= 0.5; // friction
                if state.velocity.length_squared() < 1e-6 {
                    state.velocity = Vec3::ZERO;
                }
            }
            let forward = *transform.forward();
            let right = *transform.right();
            transform.translation += state.velocity.x * dt * right
                + state.velocity.y * dt * Vec3::Y
                + state.velocity.z * dt * forward;

            // Handle mouse input
            let mut mouse_delta = Vec2::ZERO;
            if mouse_button_input.pressed(MouseButton::Left) {
                for mouse_event in mouse_events.read() {
                    mouse_delta += mouse_event.delta;
                }
            }
            if mouse_delta != Vec2::ZERO {
                // Apply look update
                state.pitch =
                    (state.pitch - mouse_delta.y * RADIANS_PER_DOT).clamp(-PI / 2., PI / 2.);
                state.yaw -= mouse_delta.x * RADIANS_PER_DOT;
                transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, state.yaw, state.pitch);
            }
        }
    }
}

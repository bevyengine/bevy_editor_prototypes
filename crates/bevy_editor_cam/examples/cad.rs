//! Editor camera example with an expanded control scheme fit for CAD software.

use bevy::{
    anti_alias::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing},
    pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
    prelude::*,
    render::camera::TemporalJitter,
};
use bevy_editor_cam::{
    extensions::{dolly_zoom::DollyZoomTrigger, look_to::LookToTrigger},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultEditorCamPlugins,
            TemporalAntiAliasPlugin,
        ))
        // The camera controller works with reactive rendering:
        // .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_projection,
                projection_specific_render_config,
                toggle_constraint,
                switch_direction,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cone::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    let cam_trans = Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y);

    commands.spawn((
        Camera3d::default(),
        cam_trans,
        Bloom::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Free,
            last_anchor_depth: cam_trans.translation.length() as f64,
            ..default()
        },
    ));

    setup_ui(commands);
}

fn projection_specific_render_config(
    mut commands: Commands,
    mut cam: Query<(Entity, &Projection, &mut Msaa), With<EditorCam>>,
) {
    let (entity, proj, mut msaa) = cam.single_mut().unwrap();
    match proj {
        Projection::Perspective(_) => {
            *msaa = Msaa::Off;
            commands
                .entity(entity)
                .insert(TemporalAntiAliasing::default())
                .insert(ScreenSpaceAmbientOcclusion::default());
        }
        Projection::Orthographic(_) => {
            *msaa = Msaa::Sample4;
            commands
                .entity(entity)
                .remove::<TemporalJitter>()
                .remove::<ScreenSpaceAmbientOcclusion>();
        }
        _ => {}
    }
}

fn toggle_projection(
    keys: Res<ButtonInput<KeyCode>>,
    mut dolly: MessageWriter<DollyZoomTrigger>,
    cam: Query<Entity, With<EditorCam>>,
    mut toggled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        *toggled = !*toggled;
        let target_projection = if *toggled {
            Projection::Orthographic(OrthographicProjection::default_3d())
        } else {
            Projection::Perspective(PerspectiveProjection::default())
        };
        dolly.write(DollyZoomTrigger {
            target_projection,
            camera: cam.single().unwrap(),
        });
    }
}

fn toggle_constraint(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam: Query<(Entity, &Transform, &mut EditorCam)>,
    mut look_to: MessageWriter<LookToTrigger>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        let (entity, transform, mut editor) = cam.single_mut().unwrap();
        match editor.orbit_constraint {
            OrbitConstraint::Fixed { .. } => editor.orbit_constraint = OrbitConstraint::Free,
            OrbitConstraint::Free => {
                editor.orbit_constraint = OrbitConstraint::Fixed {
                    up: Vec3::Y,
                    can_pass_tdc: false,
                };

                look_to.write(LookToTrigger::auto_snap_up_direction(
                    transform.forward(),
                    entity,
                    transform,
                    editor.as_ref(),
                ));
            }
        };
    }
}

fn switch_direction(
    keys: Res<ButtonInput<KeyCode>>,
    mut look_to: MessageWriter<LookToTrigger>,
    cam: Query<(Entity, &Transform, &EditorCam)>,
) {
    let (camera, transform, editor) = cam.single().unwrap();
    if keys.just_pressed(KeyCode::Digit1) {
        look_to.write(LookToTrigger::auto_snap_up_direction(
            Dir3::X,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit2) {
        look_to.write(LookToTrigger::auto_snap_up_direction(
            Dir3::Z,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit3) {
        look_to.write(LookToTrigger::auto_snap_up_direction(
            Dir3::NEG_X,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit4) {
        look_to.write(LookToTrigger::auto_snap_up_direction(
            Dir3::NEG_Z,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit5) {
        look_to.write(LookToTrigger::auto_snap_up_direction(
            Dir3::Y,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit6) {
        look_to.write(LookToTrigger::auto_snap_up_direction(
            Dir3::NEG_Y,
            camera,
            transform,
            editor,
        ));
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(
            "Left Mouse - Pan\n\
            Right Mouse - Orbit\n\
            Scroll - Zoom\n\
            P - Toggle projection\n\
            C - Toggle orbit constraint\n\
            1-6 - Switch direction",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

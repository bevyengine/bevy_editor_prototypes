use std::time::Duration;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        tonemapping::Tonemapping,
    },
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    render::{camera::TemporalJitter, primitives::Aabb},
    utils::Instant,
    window::RequestRedraw,
};
use bevy_editor_cam::{
    extensions::{dolly_zoom::DollyZoomTrigger, look_to::LookToTrigger},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_mod_picking::DefaultPickingPlugins,
            DefaultEditorCamPlugins,
            TemporalAntiAliasPlugin,
        ))
        // The camera controller works with reactive rendering:
        // .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_projection,
                projection_specific_render_config,
                toggle_constraint,
                explode,
                switch_direction,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let diffuse_map = asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2");
    let specular_map = asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2");

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/PlaneEngine/scene.gltf#Scene0"),
        transform: Transform::from_scale(Vec3::splat(2.0)),
        ..Default::default()
    });

    let cam_trans = Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y);

    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: cam_trans,
                tonemapping: Tonemapping::AcesFitted,
                ..default()
            },
            BloomSettings::default(),
            EnvironmentMapLight {
                intensity: 1000.0,
                diffuse_map: diffuse_map.clone(),
                specular_map: specular_map.clone(),
            },
            EditorCam {
                orbit_constraint: OrbitConstraint::Free,
                last_anchor_depth: cam_trans.translation.length() as f64,
                ..Default::default()
            },
        ))
        .id();

    setup_ui(commands, camera);
}

fn projection_specific_render_config(
    mut commands: Commands,
    cam: Query<(Entity, &Projection), With<EditorCam>>,
    mut msaa: ResMut<Msaa>,
) {
    let (entity, proj) = cam.single();
    match proj {
        Projection::Perspective(_) => {
            *msaa = Msaa::Off;
            commands
                .entity(entity)
                .insert(TemporalAntiAliasBundle::default())
                .insert(ScreenSpaceAmbientOcclusionBundle::default());
        }
        Projection::Orthographic(_) => {
            *msaa = Msaa::Sample4;
            commands
                .entity(entity)
                .remove::<TemporalJitter>()
                .remove::<ScreenSpaceAmbientOcclusionBundle>();
        }
    }
}

fn toggle_projection(
    keys: Res<ButtonInput<KeyCode>>,
    mut dolly: EventWriter<DollyZoomTrigger>,
    cam: Query<Entity, With<EditorCam>>,
    mut toggled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        *toggled = !*toggled;
        let target_projection = if *toggled {
            Projection::Orthographic(OrthographicProjection::default())
        } else {
            Projection::Perspective(PerspectiveProjection::default())
        };
        dolly.send(DollyZoomTrigger {
            target_projection,
            camera: cam.single(),
        });
    }
}

fn toggle_constraint(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam: Query<(Entity, &Transform, &mut EditorCam)>,
    mut look_to: EventWriter<LookToTrigger>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        let (entity, transform, mut editor) = cam.single_mut();
        match editor.orbit_constraint {
            OrbitConstraint::Fixed { .. } => editor.orbit_constraint = OrbitConstraint::Free,
            OrbitConstraint::Free => {
                editor.orbit_constraint = OrbitConstraint::Fixed {
                    up: Vec3::Y,
                    can_pass_tdc: false,
                };

                look_to.send(LookToTrigger::auto_snap_up_direction(
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
    mut look_to: EventWriter<LookToTrigger>,
    cam: Query<(Entity, &Transform, &EditorCam)>,
) {
    let (camera, transform, editor) = cam.single();
    if keys.just_pressed(KeyCode::Digit1) {
        look_to.send(LookToTrigger::auto_snap_up_direction(
            Dir3::X,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit2) {
        look_to.send(LookToTrigger::auto_snap_up_direction(
            Dir3::Z,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit3) {
        look_to.send(LookToTrigger::auto_snap_up_direction(
            Dir3::NEG_X,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit4) {
        look_to.send(LookToTrigger::auto_snap_up_direction(
            Dir3::NEG_Z,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit5) {
        look_to.send(LookToTrigger::auto_snap_up_direction(
            Dir3::Y,
            camera,
            transform,
            editor,
        ));
    }
    if keys.just_pressed(KeyCode::Digit6) {
        look_to.send(LookToTrigger::auto_snap_up_direction(
            Dir3::NEG_Y,
            camera,
            transform,
            editor,
        ));
    }
}

fn setup_ui(mut commands: Commands, camera: Entity) {
    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands
        .spawn((
            TargetCamera(camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_sections(vec![
                    TextSection::new("Left Mouse - Pan\n", style.clone()),
                    TextSection::new("Right Mouse - Orbit\n", style.clone()),
                    TextSection::new("Scroll - Zoom\n", style.clone()),
                    TextSection::new("P - Toggle projection\n", style.clone()),
                    TextSection::new("C - Toggle orbit constraint\n", style.clone()),
                    TextSection::new("E - Toggle explode\n", style.clone()),
                    TextSection::new("1-6 - Switch direction\n", style.clone()),
                ])
                .with_style(Style { ..default() }),
            );
        });
}

#[derive(Component)]
struct StartPos(f32);

#[allow(clippy::type_complexity)]
fn explode(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut toggle: Local<Option<(bool, Instant, f32)>>,
    mut explode_amount: Local<f32>,
    mut redraw: EventWriter<RequestRedraw>,
    mut parts: Query<(Entity, &mut Transform, &Aabb, Option<&StartPos>), With<Handle<Mesh>>>,
    mut matls: ResMut<Assets<StandardMaterial>>,
) {
    let animation = Duration::from_millis(2000);
    if keys.just_pressed(KeyCode::KeyE) {
        let new = if let Some((last, ..)) = *toggle {
            !last
        } else {
            true
        };
        *toggle = Some((new, Instant::now(), *explode_amount));
    }
    if let Some((toggled, start, start_amount)) = *toggle {
        let goal_amount = toggled as usize as f32;
        let t = (start.elapsed().as_secs_f32() / animation.as_secs_f32()).clamp(0.0, 1.0);
        let progress = CubicSegment::new_bezier((0.25, 0.1), (0.25, 1.0)).ease(t);
        *explode_amount = start_amount + (goal_amount - start_amount) * progress;
        for (part, mut transform, aabb, start) in &mut parts {
            let start = if let Some(start) = start {
                start.0
            } else {
                let start = aabb.max().y;
                commands.entity(part).insert(StartPos(start));
                start
            };
            transform.translation.y = *explode_amount * (start) * 2.0;
        }
        if t < 1.0 {
            redraw.send(RequestRedraw);
        }
    }
    for (_, matl) in matls.iter_mut() {
        matl.perceptual_roughness = matl.perceptual_roughness.clamp(0.1, 1.0)
    }
}

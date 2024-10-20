//! A minimal example demonstrating setting zoom limits and zooming through objects.

use bevy::prelude::*;
use bevy_color::palettes;
use bevy_editor_cam::{extensions::dolly_zoom::DollyZoomTrigger, prelude::*};
use zoom::ZoomLimits;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_mod_picking::DefaultPickingPlugins,
            DefaultEditorCamPlugins,
        ))
        .add_systems(Startup, (setup_camera, setup_scene, setup_ui))
        .add_systems(Update, (toggle_projection, toggle_zoom))
        .run();
}

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle::default(),
        EditorCam {
            zoom_limits: ZoomLimits {
                min_size_per_pixel: 0.0001,
                max_size_per_pixel: 0.01,
                zoom_through_objects: true,
            },
            ..default()
        },
        EnvironmentMapLight {
            intensity: 1000.0,
            diffuse_map: asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2"),
        },
    ));
}

fn toggle_zoom(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam: Query<&mut EditorCam>,
    mut text: Query<&mut Text>,
) {
    if keys.just_pressed(KeyCode::KeyZ) {
        let mut editor = cam.single_mut();
        editor.zoom_limits.zoom_through_objects = !editor.zoom_limits.zoom_through_objects;
        let mut text = text.single_mut();
        text.sections.last_mut().unwrap().value = if editor.zoom_limits.zoom_through_objects {
            "Zoom Through: Enabled".into()
        } else {
            "Zoom Through: Disabled".into()
        };
    }
}

//
// --- The below code is not important for the example ---
//

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(Color::srgba(0.1, 0.1, 0.9, 0.5));
    let mesh = meshes.add(Cuboid::from_size(Vec3::new(1.0, 1.0, 0.1)));

    for i in 1..5 {
        commands.spawn(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, -2.0 * i as f32),
            ..default()
        });
    }
}

fn setup_ui(mut commands: Commands) {
    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands
        .spawn((
            // TargetCamera(camera),
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
                    TextSection::new("Z - Toggle zoom through object setting\n", style.clone()),
                    TextSection::new(
                        "Zoom Through: Enabled\n",
                        TextStyle {
                            font_size: 20.0,
                            color: palettes::basic::YELLOW.into(),
                            ..default()
                        },
                    ),
                ])
                .with_style(Style { ..default() }),
            );
        });
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

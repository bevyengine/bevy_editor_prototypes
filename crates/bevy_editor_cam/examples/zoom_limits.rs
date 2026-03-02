//! A minimal example demonstrating setting zoom limits and zooming through objects.

use bevy::color::palettes;
use bevy::prelude::*;
use bevy_editor_cam::{extensions::dolly_zoom::DollyZoomTrigger, prelude::*};
use zoom::ZoomLimits;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, (setup_camera, setup_scene, setup_ui))
        .add_systems(Update, (toggle_projection, toggle_zoom))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            zoom_limits: ZoomLimits {
                min_size_per_pixel: 0.0001,
                max_size_per_pixel: 0.01,
                zoom_through_objects: true,
            },
            ..default()
        },
    ));
}

fn toggle_zoom(
    keys: Res<ButtonInput<KeyCode>>,
    mut editor_cam: Single<&mut EditorCam>,
    mut text: Single<&mut TextSpan>,
) {
    if keys.just_pressed(KeyCode::KeyZ) {
        editor_cam.zoom_limits.zoom_through_objects ^= true;
        text.0 = if editor_cam.zoom_limits.zoom_through_objects {
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
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 0.0, -2.0 * i as f32),
        ));
    }
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Text::new(
                "Left Mouse - Pan\n\
                    Right Mouse - Orbit\n\
                    Scroll - Zoom\n\
                    P - Toggle projection\n\
                    Z - Toggle zoom through object setting\n",
            ),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("Zoom Through: Enabled\n"),
            TextColor(palettes::basic::YELLOW.into()),
        ));
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

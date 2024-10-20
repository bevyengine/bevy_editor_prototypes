//! A minimal example showing the steps needed to get started with the plugin.

use bevy::prelude::*;
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultEditorCamPlugins, // Step 1: Add camera controller plugin
        ))
        .add_systems(Startup, (setup_camera, setup_scene))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        EditorCam::default(), // Step 2: add camera controller component to any cameras
    ));
}

//
// --- The below code is not important for the example ---
//

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cone::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));

    commands.spawn((
        Text::new(
            "Left Mouse - Pan\n\
            Right Mouse - Orbit\n\
            Scroll - Zoom\n",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

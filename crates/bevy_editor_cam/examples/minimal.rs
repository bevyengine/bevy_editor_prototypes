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

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        EditorCam::default(), // Step 2: add camera controller component to any cameras
        EnvironmentMapLight {
            // Unrelated to camera controller, needed for lighting:
            intensity: 1000.0,
            diffuse_map: asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2"),
            ..default()
        },
    ));
}

//
// --- The below code is not important for the example ---
//

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(asset_server.load("models/PlaneEngine/scene.gltf#Scene0")),
        Transform::from_xyz(0.0, -0.5, -2.0),
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

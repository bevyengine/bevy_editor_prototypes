//! A minimal example showing the steps needed to get started with the plugin.

use bevy::prelude::*;
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_mod_picking::DefaultPickingPlugins, // Prerequisite: Use picking plugin
            DefaultEditorCamPlugins,                 // Step 1: Add camera controller plugin
        ))
        .add_systems(Startup, (setup_camera, setup_scene))
        .run();
}

fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle::default(),
        EditorCam::default(), // Step 2: add camera controller component to any cameras
        EnvironmentMapLight {
            // Unrelated to camera controller, needed for lighting:
            intensity: 1000.0,
            diffuse_map: asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2"),
        },
    ));
}

//
// --- The below code is not important for the example ---
//

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/PlaneEngine/scene.gltf#Scene0"),
        transform: Transform::from_xyz(0.0, -0.5, -2.0),
        ..Default::default()
    });

    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands.spawn(
        TextBundle::from_sections(vec![
            TextSection::new("Left Mouse - Pan\n", style.clone()),
            TextSection::new("Right Mouse - Orbit\n", style.clone()),
            TextSection::new("Scroll - Zoom\n", style.clone()),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

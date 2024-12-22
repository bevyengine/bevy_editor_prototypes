//! This example demonstrates how to use an editor camera 2d.
//! It creates a simple sprite for visual reference and allows
//! for panning / scaling of the camera.
//!
//! # Controls
//!
//! - `Mouse Left/Middle/Right Click`: Pan the camera.
//! - `Mouse Up/Down Scroll`: Zoom the camera.

use bevy::prelude::*;
use bevy_editor_camera::editor_camera_2d::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditorCamera2dPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        EditorCamera2d {
            pan_mouse_buttons: vec![MouseButton::Left, MouseButton::Middle, MouseButton::Right],
            bound: Rect {
                min: Vec2::new(-1000.0, -1000.0),
                max: Vec2::new(1000.0, 1000.0),
            },
            scale_range: 0.4..=10.0,
            ..Default::default()
        },
    ));

    commands.spawn(Sprite {
        image: asset_server.load("bevy_bird.png"),
        ..Default::default()
    });
}

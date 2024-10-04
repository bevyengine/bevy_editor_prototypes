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
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(EditorCamera2d {
            pan_mouse_buttons: vec![MouseButton::Left, MouseButton::Middle, MouseButton::Right],
            bound: Rect {
                min: Vec2::new(-1000.0, -1000.0),
                max: Vec2::new(1000.0, 1000.0),
            },
            zoom_range: 0.4..=10.0,
            zoom: 0.4, // Set the initial zoom level.
            ..Default::default()
        });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("bevy_bird.png"),
        ..Default::default()
    });
}

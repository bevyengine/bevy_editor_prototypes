//! An infinite grid with a simple scene and a 2d camera.

use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InfiniteGridPlugin))
        .add_systems(Startup, setup_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 100.,
            dot_fadeout_strength: 0.,
            z_axis_color: Color::srgb(0.2, 8., 0.3),
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
    ));

    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::splat(100.)),
            ..default()
        },
        Transform::from_xyz(70., 100., 0.),
    ));
}

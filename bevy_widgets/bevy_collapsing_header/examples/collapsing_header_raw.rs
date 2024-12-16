//! This example demonstrates how to use the collapsing header widget.

use bevy::prelude::*;
use bevy_collapsing_header::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CollapsingHeaderPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    commands
        .spawn((
            CollapsingHeader::new("Hello, collapsing header!"),
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .with_children(|cmd| {
            cmd.spawn((Text::new(""), CollapsingHeaderText));

            cmd.spawn((CollapsingHeaderContent, Node::default()))
                .with_children(|cmd| {
                    cmd.spawn(Text::new("Content 1"));
                    cmd.spawn(Text::new("Content 2"));
                    cmd.spawn(Text::new("Content 3"));
                });
        });
}

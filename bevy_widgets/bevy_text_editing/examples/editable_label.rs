//! This example shows how to create editable label with bevy_text_editing

use bevy::prelude::*;
use bevy_text_editing::editable_text_line::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditableTextLinePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    commands
        .spawn(
            (Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            }),
        )
        .with_children(|cmd| {
            cmd.spawn((
                EditableTextLine::new("Hello, World!"),
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(25.0),
                    ..Default::default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)), // We can use any background color (or any borders/border color)
            ));
        });
}

//! This example shows how to create editable label with `bevy_text_editing`

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin},
    prelude::*,
};
use bevy_text_editing::editable_text_line::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TabNavigationPlugin)
        .add_plugins(EditableTextLinePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            TabGroup::default(),
        ))
        .with_children(|cmd| {
            cmd.spawn((
                EditableTextLine::new("Hello, World!"),
                Node {
                    // We need to manually set the width and height for the editable text line
                    // It is limitation of current implementation
                    width: Val::Px(300.0),
                    height: Val::Px(25.0),
                    ..Default::default()
                },
                TabIndex(0),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)), // We can use any background color (or any borders/border color)
            ));
            cmd.spawn((
                EditableTextLine::new("Hello, World!"),
                Node {
                    // We need to manually set the width and height for the editable text line
                    // It is limitation of current implementation
                    width: Val::Px(300.0),
                    height: Val::Px(25.0),
                    ..Default::default()
                },
                TabIndex(0),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)), // We can use any background color (or any borders/border color)
            ));
        });
}

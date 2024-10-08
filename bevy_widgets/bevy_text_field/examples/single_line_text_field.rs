//! This example shows how to use single line text field

use bevy::prelude::*;
use bevy_text_field::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LineTextFieldPlugin)
        .add_plugins(bevy_editor_styles::StylesPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            // color from bevy website background
            clear_color: ClearColorConfig::Custom(Color::srgb(
                34.0 / 255.0,
                34.0 / 255.0,
                34.0 / 255.0,
            )),
            ..default()
        },
        ..default()
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                display: Display::Flex,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|cmd| {
            cmd.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    ..Default::default()
                },
                LineTextField::new("Bevy Engine"),
            ));

            cmd.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(600.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    ..Default::default()
                },
                LineTextField::new("Bevy Engine 2"),
            ));

            cmd.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    ..Default::default()
                },
                LineTextField::new("Bevy Engine 3"),
            ));
        });
}

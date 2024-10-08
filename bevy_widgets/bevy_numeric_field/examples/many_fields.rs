//! Contains example for spawning many numeric fields

use bevy::prelude::*;
use bevy_numeric_field::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultNumericFieldPlugin)
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
                NumericField::<f32>::new(0.0),
            ));
        });
}

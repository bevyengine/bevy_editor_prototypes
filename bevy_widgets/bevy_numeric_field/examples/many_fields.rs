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
    commands.spawn(Camera {
        clear_color: ClearColorConfig::Custom(Color::srgb(
            34.0 / 255.0,
            34.0 / 255.0,
            34.0 / 255.0,
        )),
        ..Default::default()
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                display: Display::Flex,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // Unsigned integers column
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Grid,
                        grid_template_columns: vec![GridTrack::auto(), GridTrack::auto()],
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|column| {
                    spawn_numeric_field::<u8>(column, "u8");
                    spawn_numeric_field::<u16>(column, "u16");
                    spawn_numeric_field::<u32>(column, "u32");
                    spawn_numeric_field::<u64>(column, "u64");
                    spawn_numeric_field::<u128>(column, "u128");
                });

            // Signed integers column
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Grid,
                        grid_template_columns: vec![GridTrack::auto(), GridTrack::auto()],
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|column| {
                    spawn_numeric_field::<i8>(column, "i8");
                    spawn_numeric_field::<i16>(column, "i16");
                    spawn_numeric_field::<i32>(column, "i32");
                    spawn_numeric_field::<i64>(column, "i64");
                    spawn_numeric_field::<i128>(column, "i128");
                });

            // Floating-point numbers column
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Grid,
                        grid_template_columns: vec![GridTrack::auto(), GridTrack::auto()],
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|column| {
                    spawn_numeric_field::<f32>(column, "f32");
                    spawn_numeric_field::<f64>(column, "f64");
                });
        });
}

fn spawn_numeric_field<T: NumericFieldValue + Default>(row: &mut ChildBuilder, label: &str) {
    row.spawn(Text(label.to_string()));
    row.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(30.0),
                margin: UiRect::left(Val::Px(10.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        NumericField::<T>::new(T::default()),
    ));
}

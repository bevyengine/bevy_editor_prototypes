//! This example demonstrates how to use the `ValidatedInputFieldPlugin` to create a validated input field for a character name.

use bevy::{input_focus::tab_navigation::TabGroup, prelude::*};
use bevy_field_forms::{
    drag_input::{DragInput, Draggable},
    input_field::{InputField, Validable},
    validate_highlight::SimpleBorderHighlight,
    FieldFormsPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FieldFormsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: vec![
                    RepeatedGridTrack::min_content(1),
                    RepeatedGridTrack::auto(1),
                    RepeatedGridTrack::min_content(1),
                    RepeatedGridTrack::auto(1),
                ],
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            TabGroup::default(),
        ))
        .with_children(move |cmd| {
            spawn_numeric_field::<i8>(cmd, "i8");
            spawn_numeric_field::<u8>(cmd, "u8");
            spawn_numeric_field::<i16>(cmd, "i16");
            spawn_numeric_field::<u16>(cmd, "u16");
            spawn_numeric_field::<i32>(cmd, "i32");
            spawn_numeric_field::<u32>(cmd, "u32");
            spawn_numeric_field::<i64>(cmd, "i64");
            spawn_numeric_field::<u64>(cmd, "u64");
            spawn_numeric_field::<i128>(cmd, "i128");
            spawn_numeric_field::<u128>(cmd, "u128");
            spawn_numeric_field::<f32>(cmd, "f32");
            spawn_numeric_field::<f64>(cmd, "f64");
        });
}

fn spawn_numeric_field<T: Validable + Draggable>(cmd: &mut ChildSpawnerCommands, label: &str) {
    cmd.spawn((
        Text::new(format!("{}:", label)),
        Node {
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
    ));
    cmd.spawn((
        Node {
            width: Val::Px(100.0),
            height: Val::Px(25.0),
            border: UiRect::all(Val::Px(1.0)),
            margin: UiRect::all(Val::Px(5.0)),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        InputField::<T>::default(),
        SimpleBorderHighlight::default(),
        DragInput::<T>::default(),
    ));
}

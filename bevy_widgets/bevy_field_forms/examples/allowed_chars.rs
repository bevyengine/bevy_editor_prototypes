//! This example demonstrates how to use the `AllowedCharsFilter` to restrict the input to allowed characters in a text field.

use bevy::{prelude::*, utils::HashSet};
use bevy_field_forms::{
    allowed_chars_filter::{AllowedCharsFilter, AllowedCharsFilterPlugin},
    text_event_mirror::{TextEventMirror, TextEventMirrorPlugin},
};
use bevy_i_cant_believe_its_not_bsn::WithChild;
use bevy_text_editing::{
    child_traversal::FirstChildTraversalPlugin, EditableTextLine, EditableTextLinePlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditableTextLinePlugin)
        .add_plugins(TextEventMirrorPlugin)
        .add_plugins(AllowedCharsFilterPlugin)
        .add_plugins(FirstChildTraversalPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|cmd| {
            cmd.spawn((
                TextEventMirror,
                WithChild((
                    // Allow only 'a', 'b' and 'c' characters
                    AllowedCharsFilter::new(HashSet::from(['a', 'b', 'c'])),
                    WithChild((
                        EditableTextLine::controlled("abc"),
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(25.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    )),
                )),
            ));
        });
}

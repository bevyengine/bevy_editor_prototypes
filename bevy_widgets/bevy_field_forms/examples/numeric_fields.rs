//! This example demonstrates how to use the `ValidatedInputFieldPlugin` to create a validated input field for a character name.

use bevy::{prelude::*, utils::HashSet};
use bevy_field_forms::{
    input_field::{InputField, InputFieldPlugin, Validable, ValidationChanged, ValidationState},
    validate_highlight::{SimpleBorderHighlight, SimpleBorderHighlightPlugin},
};
use bevy_focus::{FocusPlugin, Focusable};
use bevy_i_cant_believe_its_not_bsn::WithChild;
use bevy_text_editing::{
    child_traversal::FirstChildTraversalPlugin, EditableTextLine, EditableTextLinePlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditableTextLinePlugin)
        .add_plugins(InputFieldPlugin::<i8>::default())
        .add_plugins(FirstChildTraversalPlugin)
        .add_plugins(SimpleBorderHighlightPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    let text_msg_entity = commands
        .spawn((
            Text::new(""),
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            TextFont {
                font_size: 12.0,
                ..Default::default()
            },
        ))
        .id();

    commands
        .spawn(
            (Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            }),
        )
        .with_children(move |cmd| {
            cmd.spawn(Text::new("Nickname:"));
            cmd.spawn((
                Node {
                    border: UiRect::all(Val::Px(1.0)),
                    width: Val::Px(300.0),
                    height: Val::Px(25.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(5.0)),
                BorderColor(Color::WHITE),
                InputField::new(CharacterName(String::new())),
                SimpleBorderHighlight::default(),
                CharacterValidator {
                    msg_text: text_msg_entity,
                },
            ));
        })
        .add_child(text_msg_entity);
}

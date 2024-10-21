//! This example demonstrates how to use the `ValidatedInputFieldPlugin` to create a validated input field for a character name.

use bevy::{prelude::*, utils::HashSet};
use bevy_field_forms::{
    validate_highlight::{SimpleBorderHighlight, SimpleBorderHighlightPlugin},
    validated_input_field::{
        Validable, ValidatedInputField, ValidatedInputFieldPlugin, ValidationChanged,
        ValidationState,
    },
};
use bevy_focus::Focusable;
use bevy_i_cant_believe_its_not_bsn::WithChild;
use bevy_text_editing::{
    child_traversal::FirstChildTraversalPlugin, EditableTextLine, EditableTextLinePlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditableTextLinePlugin)
        .add_plugins(ValidatedInputFieldPlugin::<CharacterName>::default())
        .add_plugins(FirstChildTraversalPlugin)
        .add_plugins(SimpleBorderHighlightPlugin)
        .add_observer(on_validation_changed)
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
                    ..default()
                },
                BorderRadius::all(Val::Px(5.0)),
                BorderColor(Color::WHITE),
                ValidatedInputField::new(CharacterName(String::new())),
                SimpleBorderHighlight::default(),
                Focusable,
                CharacterValidator {
                    msg_text: text_msg_entity,
                },
                WithChild((
                    EditableTextLine::new(""),
                    Node {
                        // We need to set the width and height of the editable area because limitations
                        width: Val::Px(300.0),
                        height: Val::Px(25.0),
                        ..default()
                    },
                )),
            ));
        })
        .add_child(text_msg_entity);
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct CharacterName(pub String);

impl Validable for CharacterName {
    fn validate(text: &str) -> Result<Self, String> {
        let allowed_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"
            .chars()
            .collect::<HashSet<_>>();
        if text.chars().all(|c| allowed_chars.contains(&c)) {
            Ok(CharacterName(text.to_string()))
        } else {
            let invalid_chars: String = text
                .chars()
                .filter(|c| !allowed_chars.contains(c))
                .collect();
            Err(format!("Invalid character name. The following characters are not allowed: '{}'. Only letters, numbers, and underscores can be used.", invalid_chars))
        }
    }
}

impl ToString for CharacterName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Component)]
pub struct CharacterValidator {
    pub msg_text: Entity,
}

fn on_validation_changed(
    mut trigger: Trigger<ValidationChanged>,
    mut commands: Commands,
    q_character_validator: Query<&CharacterValidator>,
) {
    let entity = trigger.entity();
    let Ok(character_validator) = q_character_validator.get(entity) else {
        return;
    };

    match &trigger.0 {
        ValidationState::Valid => {
            commands
                .entity(character_validator.msg_text)
                .insert(Text::new(""));
        }
        ValidationState::Invalid(msg) => {
            commands
                .entity(character_validator.msg_text)
                .insert(Text::new(msg));
        }
        ValidationState::Unchecked => {
            commands
                .entity(character_validator.msg_text)
                .insert(Text::new(""));
        }
    }
}

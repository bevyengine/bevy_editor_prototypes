//! This example demonstrates how to create a password input field using the `EditableTextLine` component.
//!
//! The password field is created with the following features:
//! - Masked input (characters are displayed as asterisks)
//! - A custom style to visually represent a password field
//! - A `Password` component to store the actual password value
//!
//! Run this example with:
//! ```
//! cargo run --example password
//! ```

use bevy::{input_focus::tab_navigation::TabGroup, prelude::*};
use bevy_text_editing::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditableTextLinePlugin)
        .add_systems(Startup, setup)
        .add_observer(update_password)
        .add_systems(Update, show_password)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let show_password_id = commands.spawn(Text::new("")).id();

    let password_id = commands
        .spawn((
            Password::default(),
            EditableTextLine::controlled(""),
            Node {
                width: Val::Px(300.0),
                height: Val::Px(25.0),
                ..Default::default()
            },
            BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
            ShowPassword(show_password_id),
        ))
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            TabGroup::default(),
        ))
        .add_child(password_id)
        .with_child(Text::new("Password:"))
        .add_child(show_password_id);
}

#[derive(Component, Default)]
struct Password {
    val: String,
}

#[derive(Component)]
struct ShowPassword(pub Entity);

fn update_password(
    trigger: On<TextChanged>,
    mut commands: Commands,
    mut q_passwords: Query<&mut Password>,
) {
    let entity = trigger.event().event_target();
    let Ok(mut password) = q_passwords.get_mut(entity) else {
        return;
    };

    info!("Text changed: {:?}", trigger.change);

    trigger.change.apply(&mut password.val);

    info!("Password: {:?}", password.val);

    let asterisks = "*".repeat(password.val.chars().count());
    commands.trigger(SetText {
        entity,
        text: asterisks,
    });
}

fn show_password(
    q_password_changed: Query<(&Password, &ShowPassword), Changed<Password>>,
    mut q_texts: Query<&mut Text>,
) {
    for (password, show_password) in q_password_changed.iter() {
        let Ok(mut text) = q_texts.get_mut(show_password.0) else {
            return;
        };
        text.0 = password.val.clone();
    }
}

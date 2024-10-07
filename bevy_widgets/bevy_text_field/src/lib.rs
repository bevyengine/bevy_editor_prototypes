//! This library provides a input text field widget for Bevy.
//!
//! Currently only single line text input is supported. (multiline text input is coming some time)

pub mod cursor;
pub mod input;
pub mod render;

// These colors are taken from the Bevy website's text field in examples page
const BORDER_COLOR: Color = Color::srgb(56.0 / 255.0, 56.0 / 255.0, 56.0 / 255.0);
const HIGHLIGHT_COLOR: Color = Color::srgb(107.0 / 255.0, 107.0 / 255.0, 107.0 / 255.0);
const BACKGROUND_COLOR: Color = Color::srgb(43.0 / 255.0, 44.0 / 255.0, 47.0 / 255.0);

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_focus::Focus;
use cursor::Cursor;
use render::RenderTextField;

pub struct LineTextFieldPlugin;

impl Plugin for LineTextFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RenderTextField>();

        app.observe(render::render_text_field);
        app.observe(input::text_field_on_over);
        app.observe(input::text_field_on_out);
        app.observe(input::text_field_on_click);

        app.add_systems(PreUpdate, input::keyboard_input);

        app.add_systems(
            PreUpdate,
            (spawn_render_text_field, render::trigger_render_on_change).chain(),
        );

        app.add_plugins(cursor::CursorPlugin);
    }
}

/// Single line text field
#[derive(Component, Default, Reflect)]
pub struct LineTextField {
    /// Text in the text field
    pub text: String,
    /// Cursor position
    pub cursor_position: Option<usize>,
}

#[derive(Component)]
struct LineTextFieldLinks {
    canvas: Entity,
    text: Entity,
    text_right: Entity,
    cursor: Entity,
}

impl LineTextField {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            cursor_position: None,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

fn spawn_render_text_field(
    mut commands: Commands,
    q_text_fields: Query<(Entity, &LineTextField), Added<LineTextField>>,
) {
    for (entity, text_field) in q_text_fields.iter() {
        let canvas = commands
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                },
                background_color: BackgroundColor(BACKGROUND_COLOR.clone()),
                border_color: BorderColor(BORDER_COLOR.clone()),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                ..Default::default()
            })
            .id();

        let text_field = commands
            .spawn(TextBundle::from_section(
                text_field.text.clone(),
                TextStyle::default(),
            ))
            .id();

        let text_field_right = commands
            .spawn(TextBundle::from_section("", TextStyle::default()))
            .id();
        let cursor = commands.spawn(NodeBundle::default()).id();

        commands.entity(entity).add_child(canvas);
        commands.entity(canvas).add_child(text_field);
        commands.entity(canvas).add_child(cursor);
        commands.entity(canvas).add_child(text_field_right);
        commands.entity(entity).insert(Pickable {
            is_hoverable: true,
            should_block_lower: true,
        });
        commands.entity(entity).insert(Interaction::default());

        let links = LineTextFieldLinks {
            canvas,
            text: text_field,
            text_right: text_field_right,
            cursor: cursor,
        };
        commands.entity(entity).insert(links);
    }
}

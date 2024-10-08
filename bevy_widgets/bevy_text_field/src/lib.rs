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
use bevy_focus::{Focus, FocusPlugin, Focusable};
use cursor::Cursor;
use render::RenderTextField;

pub struct LineTextFieldPlugin;

impl Plugin for LineTextFieldPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FocusPlugin>() {
            app.add_plugins(FocusPlugin);
        }

        app.add_event::<RenderTextField>();

        app.add_systems(
            PreUpdate,
            (
                render::check_cursor_overflow,
                render::update_skip_cursor_check,
            )
                .chain(),
        );
        app.observe(render::render_text_field);
        app.observe(input::text_field_on_over);
        app.observe(input::text_field_on_out);
        app.observe(input::text_field_on_click);
        app.observe(input::lost_focus);

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
pub(crate) struct LineTextFieldLinks {
    canvas: Entity,
    sub_canvas: Entity,
    text: Entity,
    text_right: Entity,
    cursor: Entity,
}

#[derive(Component, Default, Reflect)]
pub(crate) struct InnerFieldParams {
    /// Text shift (used for text longer than the text field)
    pub(crate) text_shift: f32,
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
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    overflow: Overflow::clip(),
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

        // Move text field left/right in the canvas for text longer than the text field
        let shifting_canvas = commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    width: Val::Auto,
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Start,
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();

        commands.entity(entity).add_child(canvas);
        commands.entity(canvas).add_child(shifting_canvas);
        commands.entity(shifting_canvas).add_child(text_field);
        commands.entity(shifting_canvas).add_child(cursor);
        commands.entity(shifting_canvas).add_child(text_field_right);
        commands
            .entity(entity)
            .insert(Pickable {
                is_hoverable: true,
                should_block_lower: true,
            })
            .insert(InnerFieldParams::default())
            .insert(Focusable);

        commands.entity(entity).insert(Interaction::default());

        let links = LineTextFieldLinks {
            canvas,
            sub_canvas: shifting_canvas,
            text: text_field,
            text_right: text_field_right,
            cursor: cursor,
        };
        commands.entity(entity).insert(links);
    }
}
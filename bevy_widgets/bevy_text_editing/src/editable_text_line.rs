//! This file contains the [`EditableTextLine`] component which allow create editable text by keyboard and mouse

mod input;
mod render;

use bevy::{
    prelude::*,
    text::{cosmic_text::Buffer, TextLayoutInfo},
    ui::experimental::GhostNode,
    utils::HashSet,
};
use bevy_clipboard::ClipboardPlugin;
use bevy_focus::{FocusPlugin, Focusable, SetFocus};

use crate::{
    cursor::{Cursor, CursorPlugin},
    text_change::TextChange,
    CharPosition, SetCursorPosition, SetText, TextChanged, TEXT_SELECTION_COLOR,
};

use input::*;
use render::*;

pub struct EditableTextLinePlugin;

impl Plugin for EditableTextLinePlugin {
    fn build(&self, app: &mut App) {
        // Check that our required plugins are loaded.
        if !app.is_plugin_added::<CursorPlugin>() {
            app.add_plugins(CursorPlugin);
        }
        if !app.is_plugin_added::<FocusPlugin>() {
            app.add_plugins(FocusPlugin);
        }
        if !app.is_plugin_added::<ClipboardPlugin>() {
            app.add_plugins(ClipboardPlugin);
        }

        app.add_event::<SetText>();
        app.add_event::<TextChanged>();
        app.add_event::<RenderWidget>();
        app.add_event::<SetCursorPosition>();

        app.add_systems(
            PreUpdate,
            (
                spawn_system,
                keyboard_input,
                check_cursor_overflow,
                set_cursor_pos,
            ),
        );

        app.add_observer(set_text_trigger);
        app.add_observer(on_click);
        app.add_observer(render_system);
        app.add_observer(on_focus_lost);
        app.add_observer(on_set_cursor_position);
    }
}




#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Node, Focusable)]
pub struct EditableTextLine {
    /// Text content
    pub text: String,
    /// Cursor position. Measured in characters
    pub cursor_position: Option<CharPosition>,
    /// Selection start. Measured in characters
    pub selection_start: Option<CharPosition>,
    /// Allowed chars
    pub allowed_chars: Option<HashSet<char>>,
    /// Controlled widgets do not update their state by themselves,
    /// while uncontrolled widgets can edit their own state.
    pub controlled_widget: bool,
}

impl EditableTextLine {
    /// Create uncontrolled editable text line
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    /// Create controlled editable text line
    pub fn controlled(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            controlled_widget: true,
            ..default()
        }
    }

    /// Create uncontrolled editable text line
    pub fn uncontrolled(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    /// Change mode to controlled
    pub fn with_controlled(mut self) -> Self {
        self.controlled_widget = true;
        self
    }

    /// Get selection char range
    pub fn selection_range(&self) -> Option<(CharPosition, CharPosition)> {
        if let Some(selection_start) = self.selection_start {
            if let Some(cursor_position) = self.cursor_position {
                if selection_start < cursor_position {
                    Some((selection_start, cursor_position))
                } else {
                    Some((cursor_position, selection_start))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_text_range(&self, range: (CharPosition, CharPosition)) -> Option<String> {
        if range.0 > range.1 {
            return None;
        }

        if range.0 .0 > self.text.chars().count() || range.1 .0 > self.text.chars().count() {
            return None;
        }

        let start_byte_pos = self.get_byte_position(range.0);
        let end_byte_pos = self.get_byte_position(range.1);
        Some(self.text[start_byte_pos..end_byte_pos].to_string())
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if let Some(range) = self.selection_range() {
            self.get_text_range(range)
        } else {
            None
        }
    }

    pub fn get_byte_position(&self, char_position: CharPosition) -> usize {
        if char_position.0 < self.text.chars().count() {
            self.text.char_indices().nth(char_position.0).unwrap().0
        } else {
            self.text.len()
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EditableTextInner {
    fake_cursor_text: Entity,
    cursor: Entity,
    text: Entity,
    canvas: Entity,

    fake_text_before_selection: Entity,
    fake_selection_text: Entity,

    /// Canvas shift to the left to keep cursor visible in the text field
    text_shift: f32,
}

#[derive(Event, Default, Clone)]
pub struct RenderWidget {
    /// Make cursor immediately visible and reset cursor blinking timer
    pub show_cursor: bool,
}

impl RenderWidget {
    /// Make cursor immediately visible and reset cursor blinking timer
    pub fn show_cursor() -> Self {
        Self { show_cursor: true }
    }
}

fn spawn_system(
    mut commands: Commands,
    mut q_texts: Query<(Entity, &EditableTextLine, &mut Node), Without<EditableTextInner>>,
) {
    for (e, text, mut node) in q_texts.iter_mut() {
        // Set important properties of the node
        node.overflow = Overflow::clip();

        let cursor = commands
            .spawn((
                Node {
                    width: Val::Px(2.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Visibility::Hidden,
            ))
            .id();

        let fake_cursor_text = commands
            .spawn((
                Text::new("".to_string()),
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                Node { ..default() },
                TextLayout::new_with_no_wrap(),
            ))
            .id();

        let cursor_canvas = commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_wrap: FlexWrap::NoWrap,
                ..default()
            })
            .id();

        commands.entity(cursor_canvas).add_child(fake_cursor_text);
        commands.entity(cursor_canvas).add_child(cursor);

        let fake_text_before_selection = commands
            .spawn((
                Text::new("".to_string()),
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                Node { ..default() },
                TextLayout::new_with_no_wrap(),
            ))
            .id();

        let fake_selection_text = commands
            .spawn((
                Text::new("".to_string()),
                BackgroundColor(TEXT_SELECTION_COLOR),
                Visibility::Hidden,
                Node { ..default() },
                TextLayout::new_with_no_wrap(),
            ))
            .id();

        let selection_canvas = commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .id();

        commands
            .entity(selection_canvas)
            .add_child(fake_text_before_selection);
        commands
            .entity(selection_canvas)
            .add_child(fake_selection_text);

        let text = commands
            .spawn((
                Text::new(text.text.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                TextLayout::new_with_no_wrap(),
            ))
            .id();

        let canvas = commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            })
            .id();

        commands.entity(canvas).add_child(selection_canvas);
        commands.entity(canvas).add_child(text);
        commands.entity(canvas).add_child(cursor_canvas);

        commands
            .entity(e)
            .insert(EditableTextInner {
                fake_cursor_text,
                cursor,
                text,
                canvas,
                fake_text_before_selection,
                fake_selection_text,
                text_shift: 0.0,
            })
            .add_child(canvas);
    }
}

fn set_text_trigger(
    trigger: Trigger<SetText>,
    mut commands: Commands,
    mut q_texts: Query<&mut EditableTextLine>,
) {
    let entity = trigger.entity();
    let Ok(mut line) = q_texts.get_mut(entity) else {
        return;
    };

    line.text = trigger.0.clone();
    info!("Set text for {} to {}", entity, trigger.0);

    // Trigger rerender
    commands.trigger_targets(RenderWidget::default(), entity);
}

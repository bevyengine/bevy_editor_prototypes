//! This file contains the [`EditableTextLine`] component which allow create editable text by keyboard and mouse

mod input;
mod render;

use bevy::{
    input_focus::{
        InputDispatchPlugin,
        tab_navigation::{TabIndex, TabNavigationPlugin},
    },
    platform::collections::HashSet,
    prelude::*,
};
use bevy_clipboard::ClipboardPlugin;

use crate::{
    CharPosition, HasFocus, SetCursorPosition, SetText, TEXT_SELECTION_COLOR, TextChanged,
    cursor::{Cursor, CursorPlugin},
};

use input::*;
use render::*;

/// Plugin for editable text line
pub struct EditableTextLinePlugin;

impl Plugin for EditableTextLinePlugin {
    fn build(&self, app: &mut App) {
        // Check that our required plugins are loaded.
        if !app.is_plugin_added::<CursorPlugin>() {
            app.add_plugins(CursorPlugin);
        }
        if !app.is_plugin_added::<InputDispatchPlugin>() {
            app.add_plugins(InputDispatchPlugin);
        }
        if !app.is_plugin_added::<TabNavigationPlugin>() {
            app.add_plugins(TabNavigationPlugin);
        }
        if !app.is_plugin_added::<ClipboardPlugin>() {
            app.add_plugins(ClipboardPlugin);
        }

        app.add_systems(
            PreUpdate,
            (
                spawn_system,
                update_has_focus,
                check_cursor_overflow,
                set_cursor_pos,
                propagate_text_font,
            ),
        );

        app.add_observer(set_text_trigger);
        app.add_observer(on_click);
        app.add_observer(render_system);
        app.add_observer(on_key_input);
        app.add_observer(on_set_cursor_position);
    }
}

/// A component representing an editable line of text.
///
/// This component provides functionality for text editing, including cursor positioning,
/// text selection, and character filtering. It can be used in both controlled and
/// uncontrolled modes, allowing for flexible integration with different UI paradigms.
///
/// # Fields
///
/// * `text`: The current text content of the editable line.
/// * `cursor_position`: The current position of the cursor, measured in characters.
/// * `selection_start`: The starting position of the text selection, if any.
/// * `allowed_chars`: An optional set of characters that are allowed to be entered.
/// * `controlled_widget`: A flag indicating whether the widget is controlled externally
///   or manages its own state.
///
/// # Examples
///
/// ```
/// use bevy_text_editing::EditableTextLine;
///
/// // Create an uncontrolled editable text line
/// let uncontrolled = EditableTextLine::new("Hello, World!");
///
/// // Create a controlled editable text line
/// let controlled = EditableTextLine::controlled("Editable Text");
///
/// // Create an editable text line with allowed characters
/// let mut numeric = EditableTextLine::new("12345");
/// numeric.allowed_chars = Some(('0'..='9').collect());
/// ```

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Node, HasFocus, TabIndex)]
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

    /// Returns the text within the specified character range.
    ///
    /// # Arguments
    ///
    /// * `range` - A tuple of `(start, end)` positions, where both are `CharPosition`.
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The substring within the specified range.
    /// * `None` - If the range is invalid (start > end) or out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_text_editing::EditableTextLine;
    /// use bevy_text_editing::CharPosition;
    ///
    /// let text_line = EditableTextLine::new("Hello, world!");
    /// let range = (CharPosition(0), CharPosition(5));
    /// assert_eq!(text_line.get_text_range(range), Some("Hello".to_string()));
    /// ```
    pub fn get_text_range(&self, range: (CharPosition, CharPosition)) -> Option<String> {
        if range.0 > range.1 {
            return None;
        }

        if range.0.0 > self.text.chars().count() || range.1.0 > self.text.chars().count() {
            return None;
        }

        let start_byte_pos = self.get_byte_position(range.0);
        let end_byte_pos = self.get_byte_position(range.1);
        Some(self.text[start_byte_pos..end_byte_pos].to_string())
    }

    /// Returns the selected text within the current selection range.
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The selected text.
    /// * `None` - If there is no selection.
    pub fn get_selected_text(&self) -> Option<String> {
        if let Some(range) = self.selection_range() {
            self.get_text_range(range)
        } else {
            None
        }
    }

    /// Get the byte position of a character placed at a given position in a string
    pub fn get_byte_position(&self, char_position: CharPosition) -> usize {
        if char_position.0 < self.text.chars().count() {
            self.text.char_indices().nth(char_position.0).unwrap().0
        } else {
            self.text.len()
        }
    }
}

/// Hidden component for storing inner entities of editable text line
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

    /// One frame cursor overflow check skip
    /// We need this, because we need to wait `ComputedNode` to be computed after cursor was spawned
    skip_cursor_overflow_check: bool,
}

/// Event for rendering editable text line
#[derive(Clone, EntityEvent)]
pub struct RenderWidget {
    /// The entity to propagate to
    pub entity: Entity,
    /// Make cursor immediately visible and reset cursor blinking timer
    pub show_cursor: bool,
}

impl RenderWidget {
    /// Make cursor immediately invisible and reset cursor blinking timer
    pub fn hide_cursor(entity: Entity) -> Self {
        Self {
            entity,
            show_cursor: false,
        }
    }

    /// Make cursor immediately visible and reset cursor blinking timer
    pub fn show_cursor(entity: Entity) -> Self {
        Self {
            entity,
            show_cursor: true,
        }
    }
}

fn spawn_system(
    mut commands: Commands,
    mut q_texts: Query<
        (Entity, &EditableTextLine, &mut Node, Option<&TextFont>),
        Without<EditableTextInner>,
    >,
) {
    for (e, text, mut node, font) in q_texts.iter_mut() {
        // Set important properties of the node
        node.overflow = Overflow::clip();

        let font = font.cloned().unwrap_or_default();

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
                font.clone(),
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
                font.clone(),
            ))
            .id();

        let fake_selection_text = commands
            .spawn((
                Text::new("".to_string()),
                BackgroundColor(TEXT_SELECTION_COLOR),
                Visibility::Hidden,
                Node { ..default() },
                TextLayout::new_with_no_wrap(),
                font.clone(),
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
                font.clone(),
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
                skip_cursor_overflow_check: false,
            })
            .add_child(canvas);
    }
}

fn set_text_trigger(
    trigger: On<SetText>,
    mut commands: Commands,
    mut q_texts: Query<&mut EditableTextLine>,
) {
    let entity = trigger.event().event_target();
    let Ok(mut line) = q_texts.get_mut(entity) else {
        return;
    };

    line.text = trigger.text.clone();
    // info!("Set text for {} to {}", entity, trigger.text);

    // Trigger rerender
    commands.trigger(RenderWidget::hide_cursor(entity));
}

fn propagate_text_font(
    mut commands: Commands,
    q_texts: Query<(&TextFont, &EditableTextInner), Or<(Added<TextFont>, Changed<TextFont>)>>,
) {
    for (font, inner) in q_texts.iter() {
        commands.entity(inner.fake_cursor_text).insert(font.clone());
        commands
            .entity(inner.fake_text_before_selection)
            .insert(font.clone());
        commands
            .entity(inner.fake_selection_text)
            .insert(font.clone());
        commands.entity(inner.text).insert(font.clone());
    }
}

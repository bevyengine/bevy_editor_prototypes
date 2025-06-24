//! A library for editing text in Bevy.
//!
//! This library provides a way to edit text using the keyboard and mouse.
//! It is designed to be used with Bevy's UI system.

mod char_position;
pub mod child_traversal;
pub(crate) mod cursor;
pub mod editable_text_line;
pub mod text_change;

use bevy::prelude::*;
pub use char_position::*;
use child_traversal::CachedFirstChild;
pub use editable_text_line::*;
use text_change::TextChange;

/// Color of text selection
pub const TEXT_SELECTION_COLOR: Color = Color::srgb(0.0 / 255.0, 122.0 / 255.0, 1.0);

/// An event used to set the text of an editable text widget.
/// Will be propagated to the first child of the entity it's sent to.
#[derive(Clone, Component, Event)]
pub struct SetText(pub String);

impl EntityEvent for SetText {
    type Traversal = &'static CachedFirstChild;
    const AUTO_PROPAGATE: bool = true;
}

/// Event emitted when the text in an editable text component changes
#[derive(Clone, Component, Event)]
pub struct TextChanged {
    /// The specific change that occurred to the text
    pub change: TextChange,
    /// The new text content after the change (can be not valid if editable text widget shows different from source-of-truth text)
    pub new_text: String,
    /// Old cursor position
    pub old_cursor_position: Option<CharPosition>,
    /// New cursor position
    pub new_cursor_position: Option<CharPosition>,
}

impl EntityEvent for TextChanged {
    type Traversal = &'static ChildOf;

    const AUTO_PROPAGATE: bool = true;
}

/// An event used to set the cursor position of an editable text widget.
/// Will be propagated to the first child of the entity it's sent to.
#[derive(Clone, Event, Component)]
pub struct SetCursorPosition(pub CharPosition);

impl EntityEvent for SetCursorPosition {
    type Traversal = &'static CachedFirstChild;
    const AUTO_PROPAGATE: bool = true;
}

/// A component holding a boolean for whether an entity has focus.
#[derive(Component, Copy, Clone, Default, Eq, PartialEq, Debug, Reflect)]
#[reflect(Component, Default, PartialEq, Debug, Clone)]
#[component(immutable)]
pub struct HasFocus(pub bool);

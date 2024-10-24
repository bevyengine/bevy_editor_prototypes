//! A library for editing text in Bevy.
//!
//! This library provides a way to edit text using the keyboard and mouse.
//! It is designed to be used with Bevy's UI system.

mod char_poistion;
pub mod child_traversal;
pub(crate) mod cursor;
pub mod editable_text_line;
pub mod text_change;

use bevy::prelude::*;
pub use char_poistion::*;
use child_traversal::CachedFirsChild;
pub use editable_text_line::*;
use text_change::TextChange;

/// Color of text selection
pub const TEXT_SELECTION_COLOR: Color = Color::srgb(0.0 / 255.0, 122.0 / 255.0, 1.0);

/// An event used to set the text of an editable text widget.
/// Will be propagated to the first child of the entity it's sent to.
#[derive(Clone, Component)]
pub struct SetText(pub String);

impl Event for SetText {
    type Traversal = &'static CachedFirsChild;
    const AUTO_PROPAGATE: bool = true;
}

/// Event emitted when the text in an editable text component changes
#[derive(Clone, Component)]
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

impl Event for TextChanged {
    type Traversal = &'static Parent;

    const AUTO_PROPAGATE: bool = true;
}

/// An event used to set the cursor position of an editable text widget.
/// Will be propagated to the first child of the entity it's sent to.
#[derive(Clone, Component)]
pub struct SetCursorPosition(pub CharPosition);

impl Event for SetCursorPosition {
    type Traversal = &'static CachedFirsChild;
    const AUTO_PROPAGATE: bool = true;
}

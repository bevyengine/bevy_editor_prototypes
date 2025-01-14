//! Defines the `TextChange` struct and related functionality for representing and manipulating text changes.
//!
//! This module provides a way to represent text changes as atomic operations, which can be useful
//! for implementing undo/redo functionality, collaborative editing, or any scenario where
//! tracking and applying text modifications is necessary.
//!
//! The main struct in this module is `TextChange`, which encapsulates a single text change operation,
//! including the range of text to be modified and the new text to be inserted.

use crate::{get_byte_position, CharPosition};

/// Represents a single text change operation.
/// Any text change can be represented as a series of these operations.
#[derive(Debug, Clone, Default)]
pub struct TextChange {
    /// The range of characters to be replaced, specified as (start, end) positions.
    pub range: (CharPosition, CharPosition),
    /// The new text to insert in place of the specified range.
    pub new_text: String,
}

impl TextChange {
    /// Creates a new `TextChange` instance.
    ///
    /// # Arguments
    /// * `range` - The range of characters to be replaced.
    /// * `new_text` - The new text to insert.
    pub fn new(range: (CharPosition, CharPosition), new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    /// Creates a `TextChange` that removes text within the specified range.
    ///
    /// # Arguments
    /// * `range` - The range of characters to be removed.
    pub fn remove_change(range: (CharPosition, CharPosition)) -> Self {
        Self {
            range,
            new_text: "".to_string(),
        }
    }

    /// Creates a `TextChange` that inserts new text at a specific position.
    ///
    /// # Arguments
    /// * `pos` - The position at which to insert the new text.
    /// * `new_text` - The text to be inserted.
    pub fn insert_change(pos: CharPosition, new_text: impl Into<String>) -> Self {
        Self {
            range: (pos, pos),
            new_text: new_text.into(),
        }
    }

    /// Creates a `TextChange` that does nothing
    pub fn nop_change() -> Self {
        Self::new((CharPosition(0), CharPosition(0)), "")
    }

    /// Applies the text change to the given string.
    ///
    /// # Arguments
    /// * `text` - The string to modify.
    pub fn apply(&self, text: &mut String) {
        let start_byte_pos = get_byte_position(text, self.range.0);
        let end_byte_pos = get_byte_position(text, self.range.1);
        if start_byte_pos != end_byte_pos {
            // Replace the text in the specified range
            text.replace_range(start_byte_pos..end_byte_pos, &self.new_text);
        } else {
            // Insert new text at the specified position
            text.insert_str(start_byte_pos, &self.new_text);
        }
    }

    /// Returns true if the text change is not changing the text
    pub fn is_nop(&self) -> bool {
        self.range.0 == self.range.1 && self.new_text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_replace() {
        let mut text = String::from("Hello, world!");
        let change = TextChange::new((CharPosition(7), CharPosition(12)), "Rust");
        change.apply(&mut text);
        assert_eq!(text, "Hello, Rust!");
    }

    #[test]
    fn test_apply_remove() {
        let mut text = String::from("Hello, world!");
        let change = TextChange::remove_change((CharPosition(5), CharPosition(12)));
        change.apply(&mut text);
        assert_eq!(text, "Hello!");
    }

    #[test]
    fn test_apply_insert() {
        let mut text = String::from("Hello, world!");
        let change = TextChange::insert_change(CharPosition(7), "beautiful ");
        change.apply(&mut text);
        assert_eq!(text, "Hello, beautiful world!");
    }
}

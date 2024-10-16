//! This file contains the [`EditableTextLine`] component which allow create editable text by keyboard and mouse

use bevy::{prelude::*, text::cosmic_text::Buffer, utils::HashSet};


#[derive(Component)]
pub struct EditableTextLine {
     /// Cursor position
     pub(crate) cursor_position: Option<usize>,
     /// Selection start
     pub(crate) selection_start: Option<usize>,
}
//! A library for editing text in Bevy.
//!
//! This library provides a way to edit text using the keyboard and mouse.
//! It is designed to be used with Bevy's UI system.

pub mod editable_text_line;
pub(crate) mod cursor;
mod char_poistion;

use bevy::prelude::*;
pub use char_poistion::*;

/// Color of text selection
pub const TEXT_SELECTION_COLOR: Color = Color::srgb(0.0 / 255.0, 122.0 / 255.0, 1.0);


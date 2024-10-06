//! This crate contains input focus management for Bevy widgets.

use bevy::prelude::*;

/// Component which indicates that a widget is focused and can receive input events.
#[derive(Component)]
pub struct Focus;

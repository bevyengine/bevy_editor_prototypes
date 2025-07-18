//! This crate provides core functionality for the Bevy Engine Editor.

pub mod selection;
pub mod utils;

use bevy::prelude::*;

use crate::{selection::SelectionPlugin, utils::CoreUtilsPlugin};

/// Core plugin for the editor.
#[derive(Default)]
pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SelectionPlugin, CoreUtilsPlugin));
    }
}

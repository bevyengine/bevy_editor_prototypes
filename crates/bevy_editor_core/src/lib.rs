//! This crate provides core functionality for the Bevy Engine Editor.

pub mod actions;
pub mod selection;
pub mod utils;

use bevy::prelude::*;

use crate::{actions::ActionsPlugin, selection::SelectionPlugin, utils::CoreUtilsPlugin};

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        actions::{ActionAppExt, ActionWorldExt},
        selection::EditorSelection,
        utils::IntoBoxedScene,
    };
}

/// Core plugin for the editor.
#[derive(Default)]
pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ActionsPlugin, SelectionPlugin, CoreUtilsPlugin));
    }
}

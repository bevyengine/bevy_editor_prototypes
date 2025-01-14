//! Contains simple clipboard functionality for text fields based on arboard library

use arboard::Clipboard;
use bevy::prelude::*;

/// Plugin for clipboard functionality
pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyClipboard>();
    }
}

/// Contains clipboard api instance
#[derive(Resource, Deref, DerefMut)]
pub struct BevyClipboard(pub Clipboard);

impl Default for BevyClipboard {
    fn default() -> Self {
        Self(Clipboard::new().unwrap())
    }
}

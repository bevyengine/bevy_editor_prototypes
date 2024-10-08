use arboard::Clipboard;
use bevy::prelude::*;

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyClipboard>();
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct BevyClipboard(pub Clipboard);

impl Default for BevyClipboard {
    fn default() -> Self {
        Self(Clipboard::new().unwrap())
    }
}

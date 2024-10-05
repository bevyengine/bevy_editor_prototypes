//! Palette plugin for the Bevy Editor. This plugin provides a color palette for the editor's UI.
use bevy::prelude::*;

/// The Pallet Plugin.
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>();
    }
}

/// The core resource for the editor's color palette. This resource is used to store the current theme of the editor.
/// All colors in the editor should be derived from this resource.
/// All colors should use OKLCH color space. This ensures that the colors are perceptually uniform and work well for accessibility such as color blind adjustments.
#[derive(Resource)]
pub struct Theme {
    pub background_color: Color,
    pub text_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background_color: Color::oklch(0.209, 0.0, 0.0),
            text_color: Color::oklch(0.9219, 0.0, 0.0),
        }
    }
}

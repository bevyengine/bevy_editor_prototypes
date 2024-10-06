//! Palette plugin for the Bevy Editor. This plugin provides a color palette for the editor's UI.
use bevy::prelude::*;

/// The Pallet Plugin.
pub struct StylesPlugin;

impl Plugin for StylesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>();
    }
}

/// The core resource for the editor's color palette. This resource is used to store the current theme of the editor.
/// All colors in the editor should be derived from this resource.
/// All colors should use OKLCH color space, use <https://oklch.com/> to get colors. This ensures that the colors are perceptually uniform and work well for accessibility such as color blind adjustments.
#[derive(Resource)]
pub struct Theme {
    /// The background color of the editor.
    pub background_color: BackgroundColor,
    /// The background color of the panes in the editor.
    pub pane_background_color: BackgroundColor,
    /// The text color of the editor.
    pub text_color: Color,
    /// The color of the menu bar.
    pub menu_bar_color: BackgroundColor,
    /// The Common Border Radius for the Editor.
    pub border_radius: BorderRadius,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background_color: BackgroundColor(Color::oklch(0.209, 0.0, 0.0)),
            pane_background_color: BackgroundColor(Color::oklch(0.45, 0.0, 0.0)),
            text_color: Color::oklch(0.9219, 0.0, 0.0),
            menu_bar_color: BackgroundColor(Color::oklch(0.209, 0.0, 0.0)),
            border_radius: BorderRadius {
                top_left: Val::Px(8.0),
                top_right: Val::Px(8.0),
                bottom_left: Val::Px(8.0),
                bottom_right: Val::Px(8.0),
            },
        }
    }
}

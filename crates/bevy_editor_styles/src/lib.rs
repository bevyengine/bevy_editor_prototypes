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
    pub pane_header_background_color: BackgroundColor,
    /// The background color of the content area of panes.
    pub pane_area_background_color: BackgroundColor,
    /// The text color of the editor.
    pub text_color: Color,
    /// The color of the menu bar.
    pub menu_bar_color: BackgroundColor,
    /// The Common Border Radius for the Editor.
    pub border_radius: BorderRadius,
    /// Pane header Border Radius
    pub pane_header_border_radius: BorderRadius,
    /// The default button Border Radius.
    pub button_border_radius: BorderRadius,
    /// The hover color for interactions.
    pub hover_color: Color,
    /// The background color of the context menu.
    pub context_menu_background_color: BackgroundColor,
    /// The background color of the context menu options when hovered.
    pub context_menu_button_hover_background_color: BackgroundColor,

    /// The background color of the viewports.
    pub viewport_background_color: Color,

    /// The text color for low priority text. Text that the user doesn't need to see but is convenient to have visible.
    pub low_priority_text_color: Color,
    /// The text color for high priority text. Text that the user needs to see asap.
    pub high_priority_text_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // General styles
            border_radius: BorderRadius::all(Val::Px(8.)),
            background_color: BackgroundColor(Color::oklch(0.209, 0.0, 0.0)),
            button_border_radius: BorderRadius::all(Val::Px(3.)),

            // Pane layout styles
            pane_header_background_color: BackgroundColor(Color::oklch(0.3407, 0.0, 0.0)),
            pane_area_background_color: BackgroundColor(Color::oklch(0.3677, 0.0, 0.0)),
            pane_header_border_radius: BorderRadius::top(Val::Px(8.)),

            // Menu bar styles
            menu_bar_color: BackgroundColor(Color::oklch(0.209, 0.0, 0.0)),

            // Context menu styles
            context_menu_background_color: Color::oklch(0.209, 0., 0.).into(),
            context_menu_button_hover_background_color: Color::oklch(0.3677, 0., 0.).into(),

            // Viewport styles
            viewport_background_color: Color::oklch(0.3677, 0.0, 0.0),

            // Text styles
            low_priority_text_color: Color::oklch(0.50, 0.0, 0.0),
            text_color: Color::oklch(0.9219, 0.0, 0.0),
            high_priority_text_color: Color::oklch(0.209, 0.0, 0.0),
            hover_color: Color::oklch(0.7693, 0.11687714331662057, 268.01929959201783),
        }
    }
}

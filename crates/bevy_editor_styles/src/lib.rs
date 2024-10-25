//! Palette plugin for the Bevy Editor. This plugin provides a color palette for the editor's UI.
use bevy::{asset::embedded_asset, prelude::*};

/// The Pallet Plugin.
pub struct StylesPlugin;

impl Plugin for StylesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/fonts/Inter-Regular.ttf");
        app.init_resource::<Theme>();
    }
}

/// The core resource for the editor's color palette and fonts. This resource is used to store the current theme of the editor.
/// All colors in the editor should be derived from this resource.
/// All colors should use OKLCH color space, use <https://oklch.com/> to get colors. This ensures that the colors are perceptually uniform and work well for accessibility such as color blind adjustments.
#[derive(Resource)]
pub struct Theme {
    /// The general styles for the editor.
    pub general: GeneralStyles,
    /// The styles for buttons in the editor.
    pub button: ButtonStyles,
    /// The text styles for the editor.
    pub text: TextStyles,
    /// The styles for panes in the editor.
    pub pane: PaneStyles,
    /// The styles for menus in the editor.
    pub menu: MenuStyles,
    /// The styles for context menus in the editor.
    pub context_menu: ContextMenuStyles,
    /// The styles for viewports in the editor.
    pub viewport: ViewportStyles,
    /// The styles for scroll boxes in the editor.
    pub scroll_box: ScrollBoxStyles,
}

/// The general styles for the editor.
pub struct GeneralStyles {
    /// The common border radius for elements in the editor.
    pub border_radius: BorderRadius,
    /// The common background color of the editor.
    pub background_color: BackgroundColor,
}

/// The styles for buttons in the editor.
pub struct ButtonStyles {
    /// The border radius of the buttons.
    pub border_radius: BorderRadius,
    /// The background color of the buttons.
    pub background_color: BackgroundColor,
    /// The hover color of the buttons.
    pub hover_color: Color,
}

/// The text styles for the editor.
pub struct TextStyles {
    /// The color of low priority text.
    pub low_priority: Color,
    /// The color of normal text.
    pub text_color: Color,
    /// The color of high priority text.
    pub high_priority: Color,
    /// The font for the text.
    pub font: Handle<Font>,
}

/// The styles for panes in the editor.
pub struct PaneStyles {
    /// The background color of the header of the pane.
    pub header_background_color: BackgroundColor,
    /// The background color of the content area of the pane.
    pub area_background_color: BackgroundColor,
    /// The border radius of the header of the pane.
    pub header_border_radius: BorderRadius,
}

/// The styles for menus in the editor.
pub struct MenuStyles {
    /// The background color of the menu.
    pub background_color: Color,
}

/// The styles for context menus in the editor.
pub struct ContextMenuStyles {
    /// The background color of the context menu.
    pub background_color: BackgroundColor,
    /// The hover color of the context menu.
    pub hover_color: BackgroundColor,
}

/// The styles for viewports in the editor.
pub struct ViewportStyles {
    /// The background color of the viewports.
    pub background_color: Color,
}

/// The styles for the scroll boxes in the editor.
pub struct ScrollBoxStyles {
    /// The background color of the scroll box.
    pub background_color: BackgroundColor,
    /// The color of the scroll handle.
    pub handle_color: Color,
    /// The border radius of the scroll box.
    pub border_radius: BorderRadius,
}

impl FromWorld for Theme {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Theme {
            general: GeneralStyles {
                border_radius: BorderRadius::all(Val::Px(8.)),
                background_color: BackgroundColor(Color::oklch(0.209, 0.0, 0.0)),
            },
            button: ButtonStyles {
                border_radius: BorderRadius::all(Val::Px(3.)),
                background_color: BackgroundColor(Color::oklch(0.2768, 0.0, 0.0)),
                hover_color: Color::oklch(0.7693, 0.116_877_146, 268.019_3),
            },
            text: TextStyles {
                low_priority: Color::oklch(0.50, 0.0, 0.0),
                text_color: Color::oklch(0.9219, 0.0, 0.0),
                high_priority: Color::oklch(0.48, 0.1926, 0.2243),
                font: asset_server
                    .load("embedded://bevy_editor_styles/assets/fonts/Inter-Regular.ttf"),
            },
            pane: PaneStyles {
                header_background_color: BackgroundColor(Color::oklch(0.3407, 0.0, 0.0)),
                area_background_color: BackgroundColor(Color::oklch(0.3677, 0.0, 0.0)),
                header_border_radius: BorderRadius::top(Val::Px(8.)),
            },
            menu: MenuStyles {
                background_color: Color::oklch(0.209, 0.0, 0.0),
            },
            context_menu: ContextMenuStyles {
                background_color: BackgroundColor(Color::oklch(0.209, 0., 0.)),
                hover_color: BackgroundColor(Color::oklch(0.3677, 0., 0.)),
            },
            viewport: ViewportStyles {
                background_color: Color::oklch(0.3677, 0.0, 0.0),
            },
            scroll_box: ScrollBoxStyles {
                background_color: BackgroundColor(Color::oklch(0.4, 0.0, 0.0)),
                handle_color: Color::oklch(0.325, 0.0, 0.0),
                border_radius: BorderRadius::all(Val::Px(8.)),
            },
        }
    }
}

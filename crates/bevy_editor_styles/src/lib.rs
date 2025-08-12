//! Palette plugin for the Bevy Editor. This plugin provides a color palette for the editor's UI.
use bevy::{asset::embedded_asset, prelude::*};

pub mod icons;

/// The Pallet Plugin.
pub struct StylesPlugin;

impl Plugin for StylesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/fonts/Inter-Regular.ttf");
        embedded_asset!(app, "assets/icons/Lucide.ttf");
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
    /// The icon styles for the editor.
    pub icon: IconStyles,
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

/// The icon styles for the editor.
pub struct IconStyles {
    /// The font for the icons.
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
    /// The border radius of the context menu options.
    pub option_border_radius: BorderRadius,
}

/// The styles for viewports in the editor.
pub struct ViewportStyles {
    /// The background color of the viewports.
    pub background_color: Color,
    /// The color of the x-axis.
    pub x_axis_color: Color,
    /// The color of the y-axis.
    pub y_axis_color: Color,
    /// The color of the z-axis.
    pub z_axis_color: Color,
    /// The color of the major grid lines.
    pub grid_major_line_color: Color,
    /// The color of the minor grid lines.
    pub grid_minor_line_color: Color,
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
                // CSS: #1F1F24 - Main editor background
                background_color: BackgroundColor(Color::srgb(0.122, 0.122, 0.141)),
            },
            button: ButtonStyles {
                border_radius: BorderRadius::all(Val::Px(5.)),
                // CSS: #36373B - Button background
                background_color: BackgroundColor(Color::srgb(0.212, 0.216, 0.231)),
                // CSS: #206EC9 - Active/hover blue
                hover_color: Color::srgb(0.125, 0.431, 0.788),
            },
            text: TextStyles {
                // CSS: #838385 - Low priority text
                low_priority: Color::srgb(0.514, 0.514, 0.522),
                // CSS: #ECECEC - Primary text
                text_color: Color::srgb(0.925, 0.925, 0.925),
                // CSS: #206EC9 - High priority/accent text  
                high_priority: Color::srgb(0.125, 0.431, 0.788),
                font: asset_server
                    .load("embedded://bevy_editor_styles/assets/fonts/Inter-Regular.ttf"),
            },
            icon: IconStyles {
                font: asset_server.load("embedded://bevy_editor_styles/assets/icons/Lucide.ttf"),
            },
            pane: PaneStyles {
                // CSS: #1F1F24 - Pane header
                header_background_color: BackgroundColor(Color::srgb(0.122, 0.122, 0.141)),
                // CSS: #2A2A2E - Pane content area  
                area_background_color: BackgroundColor(Color::srgb(0.165, 0.165, 0.180)),
                header_border_radius: BorderRadius::top(Val::Px(6.)),
            },
            menu: MenuStyles {
                // CSS: #1F1F24
                background_color: Color::srgb(0.122, 0.122, 0.141),
            },
            context_menu: ContextMenuStyles {
                // CSS: #1F1F24
                background_color: BackgroundColor(Color::srgb(0.122, 0.122, 0.141)),
                // CSS: #36373B - Hover background
                hover_color: BackgroundColor(Color::srgb(0.212, 0.216, 0.231)),
                option_border_radius: BorderRadius::all(Val::Px(4.)),
            },
            viewport: ViewportStyles {
                // CSS: #2A2A2E - Viewport background
                background_color: Color::srgb(0.165, 0.165, 0.180),
                // Transform component X (red): #AB4051
                x_axis_color: Color::srgb(0.671, 0.251, 0.318),
                // Transform component Y (green): #5D8D0A  
                y_axis_color: Color::srgb(0.365, 0.553, 0.039),
                // Transform component Z (blue): #2160A3
                z_axis_color: Color::srgb(0.129, 0.376, 0.639),
                // CSS: #414142 - Border colors
                grid_major_line_color: Color::srgb(0.255, 0.255, 0.259),
                grid_minor_line_color: Color::srgb(0.188, 0.188, 0.188),
            },
            scroll_box: ScrollBoxStyles {
                // CSS: #36373B
                background_color: BackgroundColor(Color::srgb(0.212, 0.216, 0.231)),
                handle_color: Color::srgb(0.180, 0.180, 0.184),
                border_radius: BorderRadius::all(Val::Px(5.)),
            },
        }
    }
}

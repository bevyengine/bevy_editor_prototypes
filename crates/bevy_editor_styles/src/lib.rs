//! Styles and theming system for the Bevy Editor.
//! 
//! This crate provides a consistent theming system for the editor UI components.
//! All colors are derived from the Figma design specification and are centralized
//! in the [`colors`] module for easy maintenance.
//!
//! ## Usage
//!
//! Add the [`StylesPlugin`] to your app and access the [`Theme`] resource:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_editor_styles::{StylesPlugin, Theme, colors::EditorColors};
//!
//! fn setup(theme: Res<Theme>) {
//!     // Use theme from resource
//!     let bg_color = theme.general.background_color;
//!     
//!     // Or use color constants directly
//!     let button_color = EditorColors::BUTTON_DEFAULT;
//! }
//! ```
use bevy::{asset::embedded_asset, prelude::*};

pub mod icons;
pub mod colors;

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
        use colors::EditorColors;
        
        Theme {
            general: GeneralStyles {
                border_radius: BorderRadius::all(Val::Px(8.)),
                background_color: BackgroundColor(EditorColors::BACKGROUND),
            },
            button: ButtonStyles {
                border_radius: BorderRadius::all(Val::Px(5.)),
                background_color: BackgroundColor(EditorColors::BUTTON_DEFAULT),
                hover_color: EditorColors::ACCENT_BLUE,
            },
            text: TextStyles {
                low_priority: EditorColors::TEXT_MUTED,
                text_color: EditorColors::TEXT_PRIMARY,
                high_priority: EditorColors::ACCENT_BLUE,
                font: asset_server
                    .load("embedded://bevy_editor_styles/assets/fonts/Inter-Regular.ttf"),
            },
            icon: IconStyles {
                font: asset_server.load("embedded://bevy_editor_styles/assets/icons/Lucide.ttf"),
            },
            pane: PaneStyles {
                header_background_color: BackgroundColor(EditorColors::BACKGROUND),
                area_background_color: BackgroundColor(EditorColors::PANEL_BACKGROUND),
                header_border_radius: BorderRadius::top(Val::Px(6.)),
            },
            menu: MenuStyles {
                background_color: EditorColors::BACKGROUND,
            },
            context_menu: ContextMenuStyles {
                background_color: BackgroundColor(EditorColors::BACKGROUND),
                hover_color: BackgroundColor(EditorColors::BUTTON_DEFAULT),
                option_border_radius: BorderRadius::all(Val::Px(4.)),
            },
            viewport: ViewportStyles {
                background_color: EditorColors::PANEL_BACKGROUND,
                x_axis_color: EditorColors::AXIS_X,
                y_axis_color: EditorColors::AXIS_Y,
                z_axis_color: EditorColors::AXIS_Z,
                grid_major_line_color: EditorColors::GRID_MAJOR,
                grid_minor_line_color: EditorColors::GRID_MINOR,
            },
            scroll_box: ScrollBoxStyles {
                background_color: BackgroundColor(EditorColors::BUTTON_DEFAULT),
                handle_color: EditorColors::BORDER,
                border_radius: BorderRadius::all(Val::Px(5.)),
            },
        }
    }
}

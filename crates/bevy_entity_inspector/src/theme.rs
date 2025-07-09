//! Theme configuration for the entity inspector

use bevy::prelude::*;

/// Inspector theme configuration
#[derive(Resource, Clone)]
pub struct InspectorTheme {
    /// Background color for the inspector panel
    pub background_color: Color,
    /// Border color for the inspector panel
    pub border_color: Color,
    /// Primary text color
    pub text_color: Color,
    /// Secondary text color (for values)
    pub text_secondary_color: Color,
    /// Selected item background color
    pub selected_color: Color,
    /// Hovered item background color
    pub hover_color: Color,
    /// Disclosure triangle color
    pub disclosure_color: Color,
    /// Header background color
    pub header_color: Color,
    /// Header text color
    pub header_text_color: Color,
    /// Font size for normal text
    pub font_size: f32,
    /// Font size for headers
    pub header_font_size: f32,
    /// Padding for container elements
    pub container_padding: f32,
    /// Indent size for tree hierarchy
    pub indent_size: f32,
    /// Height of tree nodes
    pub node_height: f32,
    /// Size of disclosure triangles
    pub disclosure_size: f32,
}

impl Default for InspectorTheme {
    fn default() -> Self {
        Self {
            background_color: Color::srgb(0.1, 0.1, 0.1),
            border_color: Color::srgb(0.3, 0.3, 0.3),
            text_color: Color::srgb(0.9, 0.9, 0.9),
            text_secondary_color: Color::srgb(0.7, 0.7, 0.7),
            selected_color: Color::srgb(0.2, 0.4, 0.8),
            hover_color: Color::srgb(0.15, 0.15, 0.15),
            disclosure_color: Color::srgb(0.7, 0.7, 0.7),
            header_color: Color::srgb(0.2, 0.2, 0.2),
            header_text_color: Color::srgb(1.0, 1.0, 1.0),
            font_size: 14.0,
            header_font_size: 16.0,
            container_padding: 8.0,
            indent_size: 20.0,
            node_height: 24.0,
            disclosure_size: 12.0,
        }
    }
}

/// Dark theme for the inspector
pub fn create_dark_inspector_theme() -> InspectorTheme {
    InspectorTheme {
        background_color: Color::srgb(0.08, 0.08, 0.08),
        border_color: Color::srgb(0.25, 0.25, 0.25),
        text_color: Color::srgb(0.95, 0.95, 0.95),
        text_secondary_color: Color::srgb(0.7, 0.7, 0.7),
        selected_color: Color::srgb(0.2, 0.4, 0.8),
        hover_color: Color::srgb(0.12, 0.12, 0.12),
        disclosure_color: Color::srgb(0.8, 0.8, 0.8),
        header_color: Color::srgb(0.15, 0.15, 0.15),
        header_text_color: Color::srgb(1.0, 1.0, 1.0),
        font_size: 13.0,
        header_font_size: 15.0,
        container_padding: 12.0,
        indent_size: 18.0,
        node_height: 26.0,
        disclosure_size: 14.0,
    }
}

/// Light theme for the inspector
pub fn create_light_inspector_theme() -> InspectorTheme {
    InspectorTheme {
        background_color: Color::srgb(0.98, 0.98, 0.98),
        border_color: Color::srgb(0.7, 0.7, 0.7),
        text_color: Color::srgb(0.1, 0.1, 0.1),
        text_secondary_color: Color::srgb(0.4, 0.4, 0.4),
        selected_color: Color::srgb(0.3, 0.5, 0.9),
        hover_color: Color::srgb(0.93, 0.93, 0.93),
        disclosure_color: Color::srgb(0.3, 0.3, 0.3),
        header_color: Color::srgb(0.9, 0.9, 0.9),
        header_text_color: Color::srgb(0.1, 0.1, 0.1),
        font_size: 13.0,
        header_font_size: 15.0,
        container_padding: 12.0,
        indent_size: 18.0,
        node_height: 26.0,
        disclosure_size: 14.0,
    }
}

/// Component that marks an entity as themed by the inspector
#[derive(Component)]
pub struct InspectorThemedComponent;

/// System to apply theme changes to inspector components
pub fn update_inspector_theme(
    mut query: Query<
        (
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextColor,
            &mut TextFont,
        ),
        (
            With<InspectorThemedComponent>,
            Or<(
                Changed<BackgroundColor>,
                Changed<BorderColor>,
                Changed<TextColor>,
                Changed<TextFont>,
            )>,
        ),
    >,
    theme: Res<InspectorTheme>,
) {
    if theme.is_changed() {
        for (mut bg_color, mut border_color, mut text_color, mut text_font) in query.iter_mut() {
            bg_color.0 = theme.background_color;
            *border_color = BorderColor::all(theme.border_color);
            text_color.0 = theme.text_color;
            text_font.font_size = theme.font_size;
        }
    }
}

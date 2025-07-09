//! Property panel for displaying detailed component information.
//!
//! This module provides a property panel that displays the reflected component data
//! for the currently selected entity or component in a clean, organized format.

use crate::events::EntityInspectorRows;
use crate::theme::InspectorTheme;
use bevy::prelude::*;

/// Component marker for the property panel container
#[derive(Component)]
pub struct PropertyPanel;

/// Component marker for the property panel content area (scrollable section)
#[derive(Component)]
pub struct PropertyPanelContent;

/// Properties for creating the property panel
pub struct PropertyPanelProps {
    /// Width of the property panel
    pub width: Val,
    /// Whether to show component type headers
    pub show_headers: bool,
    /// Whether to use alternating row colors
    pub alternating_rows: bool,
}

impl Default for PropertyPanelProps {
    fn default() -> Self {
        Self {
            width: Val::Percent(60.0), // Take 60% of the width (complement to 40% tree)
            show_headers: true,
            alternating_rows: true,
        }
    }
}

/// Creates the property panel UI structure
pub fn create_property_panel(
    commands: &mut Commands,
    props: PropertyPanelProps,
    theme: &InspectorTheme,
) -> Entity {
    commands
        .spawn((
            Node {
                width: props.width,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::left(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(8.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            PropertyPanel,
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)), // Slightly lighter than tree background
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|parent| {
            // Header
            parent.spawn((
                Text::new("Properties"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(theme.text_color),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Scrollable content area
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                PropertyPanelContent, // Add marker component
                BackgroundColor(Color::NONE),
            ));
        })
        .id()
}

/// Updates the property panel selection based on the selected tree node
/// This is a simplified stub that creates basic content
pub fn update_property_panel_selection(
    _parent: &(),
    _selected_node_id: &str,
    _inspector_data: &EntityInspectorRows,
    _theme: &InspectorTheme,
) {
    // TODO: Implement property panel content updates
    // For now, this is handled by the update_property_panel system
}

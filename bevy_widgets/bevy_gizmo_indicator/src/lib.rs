//! A gizmo mode indicator widget for the Bevy Editor.
//!
//! Displays the current transform gizmo mode and available hotkeys.

use bevy::prelude::*;
use bevy_editor_styles::Theme;
use bevy_transform_gizmos::{GizmoMode, TransformGizmoSettings};

/// Plugin for the gizmo mode indicator.
pub struct GizmoIndicatorPlugin;

impl Plugin for GizmoIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gizmo_indicator)
            .add_systems(Update, update_gizmo_indicator);
    }
}

/// Marker component for the gizmo mode indicator.
#[derive(Component)]
pub struct GizmoModeIndicator;

/// Marker component for the mode text display.
#[derive(Component)]
pub struct GizmoModeText;

/// Marker component for the snap status text display.
#[derive(Component)]
pub struct GizmoSnapText;

fn setup_gizmo_indicator(mut commands: Commands, theme: Res<Theme>) {
    commands
        .spawn((
            GizmoModeIndicator,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            // Semi-transparent dark background
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 0.8)),
            BorderRadius::all(Val::Px(6.0)),
            // Add subtle shadow effect with z-index
            ZIndex(1000),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text("Transform Gizmo".to_string()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Current mode
            parent.spawn((
                Text("Mode: Translate (W)".to_string()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.125, 0.431, 0.788)), // CSS: #206EC9 - Active blue
                GizmoModeText,
            ));

            // Snap status
            parent.spawn((
                Text("Snap: ON (Ctrl to toggle)".to_string()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.36, 0.7, 0.05)), // Green for enabled snap
                GizmoSnapText,
            ));

            // Hotkey help
            parent
                .spawn((Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },))
                .with_children(|help| {
                    help.spawn((
                        Text("Hotkeys:".to_string()),
                        TextFont {
                            font: theme.text.font.clone(),
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                    help.spawn((
                        Text("W - Translate | E - Rotate | R - Scale".to_string()),
                        TextFont {
                            font: theme.text.font.clone(),
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    help.spawn((
                        Text("Ctrl - Toggle Snapping".to_string()),
                        TextFont {
                            font: theme.text.font.clone(),
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });
        });
}

fn update_gizmo_indicator(
    gizmo_settings: Res<TransformGizmoSettings>,
    mut mode_text: Query<&mut Text, (With<GizmoModeText>, Without<GizmoSnapText>)>,
    mut snap_query: Query<
        (&mut Text, &mut TextColor),
        (With<GizmoSnapText>, Without<GizmoModeText>),
    >,
) {
    if gizmo_settings.is_changed() {
        // Update mode text
        if let Ok(mut text) = mode_text.single_mut() {
            text.0 = match gizmo_settings.mode {
                GizmoMode::Translate => "Mode: Translate (W)".to_string(),
                GizmoMode::Rotate => "Mode: Rotate (E)".to_string(),
                GizmoMode::Scale => "Mode: Scale (R)".to_string(),
            };
        }

        // Update snap text with color
        if let Ok((mut text, mut color)) = snap_query.single_mut() {
            if gizmo_settings.snap_enabled {
                text.0 = "Snap: ON (Ctrl to toggle)".to_string();
                color.0 = Color::srgb(0.36, 0.7, 0.05); // Green for enabled
            } else {
                text.0 = "Snap: OFF (Ctrl to toggle)".to_string();
                color.0 = Color::srgb(0.8, 0.3, 0.3); // Red for disabled
            }
        }
    }
}

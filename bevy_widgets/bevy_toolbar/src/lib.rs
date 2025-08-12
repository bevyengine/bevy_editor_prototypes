//! A toolbar widget for Bevy applications.
//!
//! Toolbars are a common UI element in many applications, providing quick access to frequently used commands,
//! and typically display small icons with on-hover tooltips.

use bevy::prelude::*;
use bevy_editor_core::selection::EditorSelection;
use bevy_editor_styles::Theme;
use bevy_transform_gizmos::{TransformGizmoSettings, GizmoMode};

/// Plugin for the editor toolbar.
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_toolbar.in_set(ToolbarSet))
            .add_systems(Update, (handle_toolbar_actions, update_button_colors, sync_gizmo_mode))
            .init_resource::<ActiveTool>();
    }
}

/// System set for toolbar operations.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolbarSet;

/// Root node for the toolbar.
#[derive(Component)]
pub struct ToolbarNode;

/// Marker for toolbar tool buttons.
#[derive(Component)]
pub struct ToolbarButton {
    /// The tool associated with this button.
    pub tool: EditorTool,
}

/// Available editor tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    /// Selection tool for picking entities.
    Select,
    /// Move tool for translating entities.
    Move,
    /// Rotate tool for rotating entities.
    Rotate,
    /// Scale tool for resizing entities.
    Scale,
    /// Tool for creating new entities.
    NewEntity,
    /// Save scene to file.
    Save,
    /// Load scene from file.
    Load,
    /// Undo last action.
    Undo,
    /// Redo last undone action.
    Redo,
    /// Start playback/simulation.
    Play,
    /// Pause playback/simulation.
    Pause,
    /// Stop playback/simulation.
    Stop,
}

/// Current active tool resource.
#[derive(Resource, Default)]
pub struct ActiveTool(pub EditorTool);

impl Default for EditorTool {
    fn default() -> Self {
        Self::Select
    }
}

/// Marker component for the gizmo mode status text.
#[derive(Component)]
pub struct GizmoModeStatus;

fn setup_toolbar(
    mut commands: Commands, 
    theme: Res<Theme>,
    toolbar_query: Query<Entity, With<ToolbarNode>>,
) {
    let toolbar_tools = [
        (EditorTool::Select, "Select"),
        (EditorTool::Move, "Move (W)"),
        (EditorTool::Rotate, "Rotate (E)"),
        (EditorTool::Scale, "Scale (R)"),
        (EditorTool::NewEntity, "New"),
        (EditorTool::Save, "Save"),
        (EditorTool::Load, "Load"),
        (EditorTool::Undo, "Undo"),
        (EditorTool::Redo, "Redo"),
        (EditorTool::Play, "Play"),
        (EditorTool::Pause, "Pause"),
        (EditorTool::Stop, "Stop"),
    ];

    // Find the existing ToolbarNode and populate it
    if let Ok(toolbar_entity) = toolbar_query.single() {
        commands.entity(toolbar_entity).insert((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(6.0)),
                column_gap: Val::Px(6.0),
                ..default()
            },
            // CSS: #1F1F24 with border #303030
            BackgroundColor(theme.general.background_color.0),
            BorderColor::all(Color::srgb(0.188, 0.188, 0.188)),
            theme.general.border_radius,
        )).with_children(|parent| {
        // Left side - tool buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
        )).with_children(|tools| {
            for (tool, icon) in toolbar_tools {
                tools.spawn((
                    Button,
                    ToolbarButton { tool },
                    Node {
                        width: Val::Px(58.0), // Slightly wider for keyboard shortcuts
                        height: Val::Px(18.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::horizontal(Val::Px(1.0)),
                        ..default()
                    },
                    // CSS: #36373B - Button background
                    theme.button.background_color,
                    theme.button.border_radius,
                )).with_children(|button| {
                    button.spawn((
                        Text(icon.to_string()),
                        TextFont {
                            font: theme.text.font.clone(),
                            font_size: 10.0, // Slightly smaller to fit shortcuts
                            ..default()
                        },
                        // CSS: #E6E6E6 - Button text
                        TextColor(theme.text.text_color),
                    ));
                });
            }
        });

        // Spacer to push everything to the sides
        parent.spawn((
            Node {
                flex_grow: 1.0,
                ..default()
            },
        ));

        // Right side - hotkeys and snap status
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                padding: UiRect::horizontal(Val::Px(12.0)),
                column_gap: Val::Px(16.0),
                ..default()
            },
        )).with_children(|status| {
            // Hotkeys help
            status.spawn((
                Text("W=Move | E=Rotate | R=Scale | Ctrl=Snap".to_string()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)), // Dimmed help text
            ));
            
            // Snap status
            status.spawn((
                Text("Snap: ON".to_string()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.36, 0.7, 0.05)), // Green for enabled snap
                GizmoModeStatus,
            ));
        });
        });
    } else {
        error!("Could not find ToolbarNode entity to populate with toolbar content");
    }
}

fn handle_toolbar_actions(
    mut interactions: Query<
        (&Interaction, &ToolbarButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    theme: Res<Theme>,
    mut active_tool: ResMut<ActiveTool>,
    mut gizmo_settings: ResMut<TransformGizmoSettings>,
    mut commands: Commands,
    _selection: Res<EditorSelection>,
) {
    for (interaction, toolbar_button, mut background) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                active_tool.0 = toolbar_button.tool;
                
                // Sync toolbar actions with gizmo modes
                match toolbar_button.tool {
                    EditorTool::Move => {
                        gizmo_settings.mode = GizmoMode::Translate;
                    },
                    EditorTool::Rotate => {
                        gizmo_settings.mode = GizmoMode::Rotate;
                    },
                    EditorTool::Scale => {
                        gizmo_settings.mode = GizmoMode::Scale;
                    },
                    EditorTool::NewEntity => {
                        spawn_new_entity(&mut commands);
                    },
                    EditorTool::Save => {
                        // Save scene functionality - to be implemented
                    },
                    EditorTool::Load => {
                        // Load scene functionality - to be implemented
                    },
                    _ => {}
                }
            },
            Interaction::Hovered => {
                if active_tool.0 != toolbar_button.tool {
                    // CSS: hover state
                    *background = BackgroundColor(Color::srgb(0.255, 0.259, 0.278));
                }
            },
            Interaction::None => {
                if active_tool.0 == toolbar_button.tool {
                    // CSS: #206EC9 - Active tool
                    *background = BackgroundColor(theme.button.hover_color);
                } else {
                    // CSS: #36373B - Default button
                    *background = theme.button.background_color;
                }
            },
        }
    }
}

fn update_button_colors(
    active_tool: Res<ActiveTool>,
    gizmo_settings: Res<TransformGizmoSettings>,
    theme: Res<Theme>,
    mut buttons: Query<(&ToolbarButton, &mut BackgroundColor, &Interaction), With<Button>>,
) {
    if active_tool.is_changed() || gizmo_settings.is_changed() {
        for (toolbar_button, mut background, interaction) in &mut buttons {
            let is_active = active_tool.0 == toolbar_button.tool || 
                (toolbar_button.tool == EditorTool::Move && gizmo_settings.mode == GizmoMode::Translate) ||
                (toolbar_button.tool == EditorTool::Rotate && gizmo_settings.mode == GizmoMode::Rotate) ||
                (toolbar_button.tool == EditorTool::Scale && gizmo_settings.mode == GizmoMode::Scale);
                
            match *interaction {
                Interaction::Hovered => {
                    if is_active {
                        // CSS: #206EC9 - Active tool, slightly brighter on hover
                        *background = BackgroundColor(Color::srgb(0.145, 0.471, 0.828));
                    } else {
                        // CSS: hover state - slightly lighter
                        *background = BackgroundColor(Color::srgb(0.255, 0.259, 0.278));
                    }
                },
                _ => {
                    if is_active {
                        // CSS: #206EC9 - Active tool
                        *background = BackgroundColor(theme.button.hover_color);
                    } else {
                        // CSS: #36373B - Default button background
                        *background = theme.button.background_color;
                    }
                }
            }
        }
    }
}

fn sync_gizmo_mode(
    gizmo_settings: Res<TransformGizmoSettings>,
    mut active_tool: ResMut<ActiveTool>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<GizmoModeStatus>>,
) {
    if gizmo_settings.is_changed() {
        // Update active tool to match gizmo mode
        match gizmo_settings.mode {
            GizmoMode::Translate => active_tool.0 = EditorTool::Move,
            GizmoMode::Rotate => active_tool.0 = EditorTool::Rotate,
            GizmoMode::Scale => active_tool.0 = EditorTool::Scale,
        }

        // Update status text and color to show snap status
        if let Ok((mut text, mut color)) = status_query.single_mut() {
            if gizmo_settings.snap_enabled {
                text.0 = "Snap: ON".to_string();
                color.0 = Color::srgb(0.36, 0.7, 0.05); // Green for enabled
            } else {
                text.0 = "Snap: OFF".to_string();
                color.0 = Color::srgb(0.8, 0.3, 0.3); // Red for disabled
            }
        }
    }
}

fn spawn_new_entity(commands: &mut Commands) {
    let _entity = commands.spawn((
        Name::new("New Entity"),
        Transform::default(),
        Visibility::default(),
    )).id();
    
}

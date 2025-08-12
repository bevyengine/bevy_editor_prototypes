//! A toolbar widget for Bevy applications.
//!
//! Toolbars are a common UI element in many applications, providing quick access to frequently used commands,
//! and typically display small icons with on-hover tooltips.

use bevy::prelude::*;
use bevy_editor_core::selection::EditorSelection;
use bevy_editor_styles::Theme;

/// Plugin for the editor toolbar.
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_toolbar.in_set(ToolbarSet))
            .add_systems(Update, (handle_toolbar_actions, update_button_colors));
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

fn setup_toolbar(mut commands: Commands, theme: Res<Theme>) {
    let toolbar_tools = [
        (EditorTool::Select, "Select"),
        (EditorTool::Move, "Move"),
        (EditorTool::Rotate, "Rotate"),
        (EditorTool::Scale, "Scale"),
        (EditorTool::NewEntity, "New"),
        (EditorTool::Save, "Save"),
        (EditorTool::Load, "Load"),
        (EditorTool::Undo, "Undo"),
        (EditorTool::Redo, "Redo"),
        (EditorTool::Play, "Play"),
        (EditorTool::Pause, "Pause"),
        (EditorTool::Stop, "Stop"),
    ];

    commands.spawn((
        ToolbarNode,
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
        for (tool, icon) in toolbar_tools {
            parent.spawn((
                Button,
                ToolbarButton { tool },
                Node {
                    width: Val::Px(48.0),
                    height: Val::Px(18.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::horizontal(Val::Px(2.0)),
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
                        font_size: 11.0,
                        ..default()
                    },
                    // CSS: #E6E6E6 - Button text
                    TextColor(theme.text.text_color),
                ));
            });
        }
    });
}

fn handle_toolbar_actions(
    mut interactions: Query<
        (&Interaction, &ToolbarButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    theme: Res<Theme>,
    mut active_tool: ResMut<ActiveTool>,
    mut commands: Commands,
    _selection: Res<EditorSelection>,
) {
    for (interaction, toolbar_button, mut background) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                active_tool.0 = toolbar_button.tool;
                
                match toolbar_button.tool {
                    EditorTool::NewEntity => {
                        spawn_new_entity(&mut commands);
                    },
                    EditorTool::Save => {
                        info!("Save scene functionality - to be implemented");
                    },
                    EditorTool::Load => {
                        info!("Load scene functionality - to be implemented");
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
    theme: Res<Theme>,
    mut buttons: Query<(&ToolbarButton, &mut BackgroundColor, &Interaction), With<Button>>,
) {
    if active_tool.is_changed() {
        for (toolbar_button, mut background, interaction) in &mut buttons {
            match *interaction {
                Interaction::Hovered => {
                    if active_tool.0 == toolbar_button.tool {
                        // CSS: #206EC9 - Active tool
                        *background = BackgroundColor(theme.button.hover_color);
                    } else {
                        // CSS: hover state - slightly lighter
                        *background = BackgroundColor(Color::srgb(0.255, 0.259, 0.278));
                    }
                },
                _ => {
                    if active_tool.0 == toolbar_button.tool {
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

fn spawn_new_entity(commands: &mut Commands) {
    let entity = commands.spawn((
        Name::new("New Entity"),
        Transform::default(),
        Visibility::default(),
    )).id();
    
    info!("Created new entity: {:?}", entity);
}

//! Modern inspector widgets using standard Bevy UI patterns

use crate::theme::*;
use bevy::prelude::*;

/// Properties for creating an inspector panel
#[derive(Default)]
pub struct InspectorPanelProps {
    /// Title of the inspector panel
    pub title: String,
    /// Width of the panel
    pub width: Val,
    /// Height of the panel
    pub height: Val,
    /// Whether the panel is collapsible
    pub collapsible: bool,
    /// Whether the panel is initially collapsed
    pub collapsed: bool,
}

/// Component that marks the inspector panel container
#[derive(Component)]
pub struct InspectorPanel;

/// Component that marks the inspector header
#[derive(Component)]
pub struct InspectorHeader;

/// Component that marks the inspector content area
#[derive(Component)]
pub struct InspectorContent;

/// Creates a modern inspector panel using standard Bevy UI
pub fn create_inspector_panel(
    commands: &mut Commands,
    props: InspectorPanelProps,
    theme: &InspectorTheme,
) -> Entity {
    let panel = commands
        .spawn((
            Node {
                width: props.width,
                height: props.height,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(theme.background_color),
            BorderColor::all(theme.border_color),
            BorderRadius::all(Val::Px(4.0)),
            InspectorPanel,
            InspectorThemedComponent,
        ))
        .id();

    // Add header
    let header = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(theme.container_padding)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.header_color),
            BorderColor::all(theme.border_color),
            InspectorHeader,
        ))
        .id();

    // Add header text
    let header_text = commands
        .spawn((
            Text::new(props.title),
            TextFont {
                font_size: theme.header_font_size,
                ..default()
            },
            TextColor(theme.header_text_color),
        ))
        .id();

    commands.entity(header).add_child(header_text);

    // Add content area
    let content = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.container_padding)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            InspectorContent,
        ))
        .id();

    // Add header and content to panel
    commands.entity(panel).add_child(header);
    commands.entity(panel).add_child(content);

    panel
}

/// Properties for creating an inspector field
#[derive(Default)]
pub struct InspectorFieldProps {
    /// Name of the field
    pub name: String,
    /// Value of the field
    pub value: String,
    /// Type of the field (for styling)
    pub field_type: InspectorFieldType,
    /// Whether the field is editable
    pub editable: bool,
}

/// Type of inspector field for styling purposes
#[derive(Default, Clone, Copy)]
pub enum InspectorFieldType {
    /// Text field, default type
    #[default]
    Text,
    /// Numeric field
    Number,
    /// Boolean field
    Boolean,
    /// Vector field
    Vector,
    /// Color field
    Color,
    /// Entity field
    Entity,
    /// Component field
    Component,
}

/// Component that marks an inspector field
#[derive(Component)]
pub struct InspectorField {
    /// Name of the field
    pub name: String,
    /// Type of the field (for styling)
    pub field_type: InspectorFieldType,
}

/// Creates an inspector field widget using standard Bevy UI
pub fn create_inspector_field(
    commands: &mut Commands,
    props: InspectorFieldProps,
    theme: &InspectorTheme,
) -> Entity {
    let value_color = match props.field_type {
        InspectorFieldType::Number => Color::srgb(0.6, 0.8, 1.0),
        InspectorFieldType::Boolean => Color::srgb(1.0, 0.8, 0.6),
        InspectorFieldType::Vector => Color::srgb(0.8, 1.0, 0.6),
        InspectorFieldType::Color => Color::srgb(1.0, 0.6, 0.8),
        InspectorFieldType::Entity => Color::srgb(0.8, 0.6, 1.0),
        InspectorFieldType::Component => Color::srgb(0.6, 1.0, 0.8),
        _ => theme.text_secondary_color,
    };

    let field = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(theme.node_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            },
            InspectorField {
                name: props.name.clone(),
                field_type: props.field_type,
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(Val::Px(2.0)),
        ))
        .id();

    // Add field name
    let name_text = commands
        .spawn((
            Text::new(props.name),
            TextFont {
                font_size: theme.font_size,
                ..default()
            },
            TextColor(theme.text_color),
        ))
        .id();

    let name_container = commands
        .spawn((Node {
            flex_grow: 1.0,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..default()
        },))
        .id();

    commands.entity(name_container).add_child(name_text);

    // Add field value
    let value_text = commands
        .spawn((
            Text::new(props.value),
            TextFont {
                font_size: theme.font_size,
                ..default()
            },
            TextColor(value_color),
        ))
        .id();

    let value_container = commands
        .spawn((Node {
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
            ..default()
        },))
        .id();

    commands.entity(value_container).add_child(value_text);

    // Add containers to field
    commands.entity(field).add_child(name_container);
    commands.entity(field).add_child(value_container);

    field
}

/// System to handle inspector field interactions
pub fn update_inspector_field_style(
    mut query: Query<(&mut BackgroundColor, &InspectorField, &Interaction), Changed<Interaction>>,
    theme: Res<InspectorTheme>,
) {
    for (mut bg_color, _field, interaction) in query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                bg_color.0 = theme.hover_color;
            }
            Interaction::Pressed => {
                bg_color.0 = theme.selected_color;
            }
            Interaction::None => {
                bg_color.0 = Color::NONE;
            }
        }
    }
}

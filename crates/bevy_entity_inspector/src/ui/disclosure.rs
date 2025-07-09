//! Disclosure triangle UI components for collapsible tree nodes

use bevy::{ecs::event::BufferedEvent, prelude::*};

/// Component representing a collapsible disclosure triangle
#[derive(Component, Clone, Debug)]
pub struct DisclosureTriangle {
    /// Whether the disclosure triangle is currently expanded
    pub is_expanded: bool,
    /// Target ID for the disclosure triangle, used to identify which section it controls
    pub target_id: String,
}

/// Properties for disclosure triangle creation
#[derive(Default)]
pub struct DisclosureProps {
    /// Size of the disclosure triangle
    pub size: f32,
    /// Initial expanded state
    pub is_expanded: bool,
    /// Target ID for the disclosure
    pub target_id: String,
}

/// Event fired when a disclosure triangle is toggled
#[derive(Event, BufferedEvent)]
pub struct DisclosureToggled {
    /// Target ID of the disclosure triangle that was toggled
    pub target_id: String,
    /// New expanded state after the toggle
    pub is_expanded: bool,
}

/// Creates a modern disclosure triangle widget using standard Bevy UI
#[allow(dead_code)]
pub fn create_disclosure_triangle(commands: &mut Commands, props: DisclosureProps) -> Entity {
    let triangle_char = if props.is_expanded { "v" } else { ">" };

    let button = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(props.size + 8.0),
                height: Val::Px(props.size + 8.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::right(Val::Px(4.0)),
                ..default()
            },
            DisclosureTriangle {
                is_expanded: props.is_expanded,
                target_id: props.target_id,
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(Val::Px(2.0)),
        ))
        .id();

    let text = commands
        .spawn((
            Text::new(triangle_char),
            TextFont {
                font_size: props.size,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ))
        .id();

    commands.entity(button).add_child(text);
    button
}

/// System to handle disclosure triangle interaction styling
pub fn update_disclosure_style(
    mut query: Query<
        (&mut BackgroundColor, &DisclosureTriangle, &Interaction),
        (Changed<Interaction>, With<DisclosureTriangle>),
    >,
) {
    for (mut bg_color, _disclosure, interaction) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                bg_color.0 = Color::srgb(0.3, 0.3, 0.3);
            }
            Interaction::Hovered => {
                bg_color.0 = Color::srgb(0.2, 0.2, 0.2);
            }
            Interaction::None => {
                bg_color.0 = Color::NONE;
            }
        }
    }
}

/// System to handle disclosure triangle clicks
pub fn handle_disclosure_clicks(
    query: Query<(&Interaction, &DisclosureTriangle), (Changed<Interaction>, With<Button>)>,
    mut toggle_events: EventWriter<DisclosureToggled>,
) {
    for (interaction, disclosure) in query.iter() {
        if *interaction == Interaction::Pressed {
            toggle_events.write(DisclosureToggled {
                target_id: disclosure.target_id.clone(),
                is_expanded: !disclosure.is_expanded,
            });
        }
    }
}

/// Legacy function for spawning disclosure triangles (for backward compatibility)
#[allow(dead_code)]
pub fn spawn_disclosure_triangle(
    commands: &mut Commands,
    target_id: String,
    is_expanded: bool,
    size: f32,
) -> Entity {
    create_disclosure_triangle(
        commands,
        DisclosureProps {
            size,
            is_expanded,
            target_id,
        },
    )
}

/// Plugin for disclosure triangle functionality
pub struct DisclosurePlugin;

impl Plugin for DisclosurePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DisclosureToggled>()
            .add_systems(Update, (update_disclosure_style, handle_disclosure_clicks));
    }
}

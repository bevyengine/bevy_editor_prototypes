//! Contains code for rendering text fields.
//!

use crate::{cursor::Cursor, LineTextField, LineTextFieldLinks, BORDER_COLOR, HIGHLIGHT_COLOR};
use bevy::prelude::*;

#[derive(Event)]
pub struct RenderTextField;

pub fn trigger_render_on_change(
    mut commands: Commands,
    q_fields: Query<Entity, Changed<LineTextField>>,
) {
    if q_fields.is_empty() {
        return;
    }
    info!("Changed: {:?}", q_fields.iter().collect::<Vec<_>>());
    commands.trigger_targets(RenderTextField, q_fields.iter().collect::<Vec<_>>());
}

pub fn render_text_field(
    mut trigger: Trigger<RenderTextField>,
    mut commands: Commands,
    q_text_fields: Query<(&LineTextField, &LineTextFieldLinks, &Interaction)>,
    mut q_border_color: Query<&mut BorderColor>,
) {
    let entity = trigger.entity();
    trigger.propagate(false);

    let Ok((text_field, links, highlighted)) = q_text_fields.get(entity) else {
        return;
    };

    info!("Render: {:?}", trigger.entity());

    let border_color =
        if *highlighted == Interaction::Hovered || *highlighted == Interaction::Pressed {
            HIGHLIGHT_COLOR
        } else {
            BORDER_COLOR
        };

    let Ok(mut canvas_border_color) = q_border_color.get_mut(links.canvas) else {
        return;
    };

    *canvas_border_color = BorderColor(border_color);

    if let Some(cursor) = text_field.cursor_position {
        let (left_text, right_text) = text_field.text.split_at(cursor);
        commands
            .entity(links.text)
            .insert(TextBundle::from_section(left_text, TextStyle::default()));
        commands
            .entity(links.text_right)
            .insert(TextBundle::from_section(right_text, TextStyle::default()));
        commands
            .entity(links.cursor)
            .insert(NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Px(2.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(Cursor::default());
    } else {
        commands.entity(links.text).insert(TextBundle::from_section(
            text_field.text.clone(),
            TextStyle::default(),
        ));
        commands
            .entity(links.cursor)
            .insert(NodeBundle::default())
            .remove::<Cursor>();
        commands
            .entity(links.text_right)
            .insert(TextBundle::from_section("", TextStyle::default()));
    }
}

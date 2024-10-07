//! Contains major input logic for text field
//!

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_focus::Focus;

use crate::{render::RenderTextField, LineTextField, LineTextFieldLinks};

pub(crate) fn text_field_on_over(
    over: Trigger<Pointer<Over>>,
    mut commands: Commands,
    q_text_fields: Query<&LineTextField>,
) {
    let entity = over.entity();
    info!("Over: {:?}", entity);

    // Observers attached to child of LineTextField, so we must get text field entity from parent componeny
    let Ok(_) = q_text_fields.get(entity) else {
        return;
    };

    commands.trigger_targets(RenderTextField, entity);
}

pub(crate) fn text_field_on_out(
    out: Trigger<Pointer<Out>>,
    mut commands: Commands,
    q_text_fields: Query<&LineTextField>,
) {
    let entity = out.entity();
    info!("Out: {:?}", entity);
    // Observers attached to child of LineTextField, so we must get text field entity from parent componeny
    let Ok(_) = q_text_fields.get(entity) else {
        return;
    };

    commands.trigger_targets(RenderTextField, entity);
}

pub(crate) fn text_field_on_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut q_text_fields: Query<(&mut LineTextField, &LineTextFieldLinks, Option<&Focus>)>,
    q_nodes: Query<(&GlobalTransform, &Node)>
) {
    info!("Click: {:?}", click.entity());
    let entity = click.entity();
    let click_data = click.event();

    let Ok((mut text_field, links, focus)) = q_text_fields.get_mut(entity) else {
        return;
    };

    let mut cursor_pos = text_field.text.len();
    if let Ok((pos, text_left)) = q_nodes.get(links.text) {
        let rect = text_left.logical_rect(pos);
        if rect.contains(click_data.pointer_location.position) {
            let dx = click_data.pointer_location.position.x - rect.min.x;
            let dx_relative = dx / rect.width();

            if let Some(cursor) = text_field.cursor_position {
                cursor_pos = (cursor as f32 * dx_relative).round() as usize;
            } else {
                cursor_pos = (dx_relative * text_field.text.len() as f32).round() as usize;
            }
        }
    }

    if let Ok((pos, text_right)) = q_nodes.get(links.text_right) {
        let rect = text_right.logical_rect(pos);
        if rect.contains(click_data.pointer_location.position) {
            let dx = click_data.pointer_location.position.x - rect.min.x;
            let dx_relative = dx / rect.width();
            
            if let Some(cursor) = text_field.cursor_position {
                let text_right_width = text_field.text.len() - cursor;
                let relative_cursor = (dx_relative * text_right_width as f32).round() as usize;
                cursor_pos = cursor + relative_cursor;
            } else {
                // Unexpected
            }
        }
    }

    commands.entity(entity).insert(Focus);
    text_field.cursor_position = Some(cursor_pos);
    commands.trigger_targets(RenderTextField, entity);
}

pub(crate) fn keyboard_input(
    mut commands: Commands,
    mut q_text_fields: Query<(Entity, &mut LineTextField), With<Focus>>,
    mut events: EventReader<KeyboardInput>,
) {
    let Ok((entity, mut text_field)) = q_text_fields.get_single_mut() else {
        return;
    };

    let Some(mut current_cursor) = text_field.cursor_position.clone() else {
        return;
    };

    let mut need_render = false;

    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Space => {
                need_render = true;
                text_field.text.insert(current_cursor, ' ');
                current_cursor += 1;
            }
            Key::Backspace => {
                need_render = true;
                if current_cursor > 0 {
                    let prev_char_index = text_field.text[..current_cursor]
                        .chars()
                        .next_back()
                        .map(char::len_utf8)
                        .unwrap_or(0);
                    text_field.text.remove(current_cursor - prev_char_index);
                    current_cursor -= prev_char_index;
                }
            }
            Key::Character(c) => {
                need_render = true;
                for c in c.chars() {
                    text_field.text.insert(current_cursor, c);
                    current_cursor += c.len_utf8();
                }
            }
            Key::ArrowLeft => {
                if current_cursor > 0 {
                    let prev_char_index = text_field.text[..current_cursor]
                        .chars()
                        .next_back()
                        .map(char::len_utf8)
                        .unwrap_or(0);
                    current_cursor -= prev_char_index;
                    need_render = true;
                }
            }
            Key::ArrowRight => {
                if current_cursor < text_field.text.len() {
                    let next_char_index = text_field.text[current_cursor..]
                        .chars()
                        .next()
                        .map(char::len_utf8)
                        .unwrap_or(0);
                    current_cursor += next_char_index;
                    need_render = true;
                }
            }
            _ => {}
        }
    }

    if need_render {
        // cursor position changed hided in if to prevert infinite change triggering
        text_field.cursor_position = Some(current_cursor);
        commands.trigger_targets(RenderTextField, entity);
    }
}

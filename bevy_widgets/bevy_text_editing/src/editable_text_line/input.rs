use super::*;
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_clipboard::BevyClipboard;
use bevy_focus::Focus;

pub fn on_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut q_editable_texts: Query<(&mut EditableTextLine, &mut EditableTextInner)>,
    q_texts: Query<(&ComputedNode, &GlobalTransform)>,
    key_states: Res<ButtonInput<KeyCode>>,
) {
    let entity = click.entity();
    let Ok((mut text_line, mut inner)) = q_editable_texts.get_mut(entity) else {
        return;
    };

    let Ok((node, global_transform)) = q_texts.get(inner.text) else {
        return;
    };

    let shift_pressed = key_states.pressed(KeyCode::ShiftLeft);

    info!("Clicked on editable text line {}", entity);

    let click_pos = click.pointer_location.position;

    let self_pos = global_transform.translation();

    let self_size = node.size();

    let dx_relative = (click_pos.x - self_pos.x) / self_size.x + 0.5;

    let mut cursor_pos = (text_line.text.chars().count() as f32 * dx_relative).round() as usize;
    if cursor_pos < text_line.text.chars().count() {
    } else {
        cursor_pos = text_line.text.chars().count();
    }

    if shift_pressed && text_line.selection_start.is_none() {
        // Set selection start on previous cursor position
        text_line.selection_start = text_line.cursor_position;
    } else if !shift_pressed {
        text_line.selection_start = None;
    }
    text_line.cursor_position = Some(CharPosition(cursor_pos));

    commands.trigger_targets(SetFocus, entity);
    commands.trigger_targets(RenderWidget::show_cursor(), entity);
}

pub fn keyboard_input(
    mut commands: Commands,
    mut q_text_fields: Query<(Entity, &mut EditableTextLine), With<Focus>>,
    mut events: EventReader<KeyboardInput>,
    key_states: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<BevyClipboard>,
) {
    let Ok((entity, mut text_field)) = q_text_fields.get_single_mut() else {
        return;
    };

    let Some(mut current_cursor) = text_field.cursor_position else {
        return;
    };
    current_cursor.0 = current_cursor.0.clamp(0, text_field.text.chars().count());

    let mut need_render = false;

    // check for Ctrl-C, Ctrl-V, Ctrl-A etc
    if key_states.pressed(KeyCode::ControlLeft) {
        if key_states.just_pressed(KeyCode::KeyC) {
            need_render = true;
            events.clear(); // clear events that were triggered by pasting

            if let Some(selected_text) = text_field.get_selected_text() {
                if let Err(e) = clipboard.set_text(selected_text) {
                    warn!("Clipboard error: {}", e);
                }
            }
        } else if key_states.pressed(KeyCode::KeyC) {
            events.clear(); // clear events that were triggered by pasting (for example it can be holded and we need to process it only once)
        } else if key_states.just_pressed(KeyCode::KeyV) {
            need_render = true;
            events.clear(); // clear events that were triggered by pasting

            match clipboard.get_text() {
                Ok(text) => {
                    if text_field.selection_start.is_none() {
                        let currsor_byte_pos = text_field.get_byte_position(current_cursor);
                        text_field.text.insert_str(currsor_byte_pos, &text);
                        current_cursor = CharPosition(currsor_byte_pos + text.chars().count());
                    } else {
                        let selected_range = text_field.selection_range().unwrap();
                        let start_byte_pos = text_field.get_byte_position(selected_range.0);
                        let end_byte_pos = text_field.get_byte_position(selected_range.1);
                        text_field
                            .text
                            .replace_range(start_byte_pos..end_byte_pos, &text);
                        current_cursor = selected_range.0 + CharPosition(text.chars().count());

                        text_field.selection_start = None;
                    }
                }
                Err(e) => {
                    warn!("Clipboard error: {}", e);
                }
            }
        } else if key_states.pressed(KeyCode::KeyV) {
            events.clear(); // clear events that were triggered by pasting (for example it can be holded and we need to process it only once)
        } else if key_states.just_pressed(KeyCode::KeyX) {
            need_render = true;
            events.clear(); // clear events that were triggered by cut
            if let Some(selected_text) = text_field.get_selected_text() {
                if let Err(e) = clipboard.set_text(selected_text) {
                    warn!("Clipboard error: {}", e);
                }
            }
            if let Some(selected_range) = text_field.selection_range() {
                let start_byte_pos = text_field.get_byte_position(selected_range.0);
                let end_byte_pos = text_field.get_byte_position(selected_range.1);
                text_field
                    .text
                    .replace_range(start_byte_pos..end_byte_pos, "");
                current_cursor = selected_range.0;
                text_field.selection_start = None;
            }
        } else if key_states.pressed(KeyCode::KeyX) {
            events.clear(); // clear events that were triggered by pasting (for example it can be holded and we need to process it only once)
        } else if key_states.just_pressed(KeyCode::KeyA) {
            // Select all text
            need_render = true;
            text_field.selection_start = Some(CharPosition(0));
            current_cursor = CharPosition(text_field.text.chars().count());
            events.clear();
        } else if key_states.pressed(KeyCode::KeyA) {
            events.clear(); // clear events that were triggered by pasting (for example it can be holded and we need to process it only once)
        }
    }

    if !need_render {
        for event in events.read() {
            if !event.state.is_pressed() {
                continue;
            }
            match &event.logical_key {
                Key::Space => {
                    need_render = true;

                    if let Some((start, end)) = text_field.selection_range() {
                        let start_byte_pos = text_field.get_byte_position(start);
                        let end_byte_pos = text_field.get_byte_position(end);
                        text_field
                            .text
                            .replace_range(start_byte_pos..end_byte_pos, " ");
                        current_cursor = start + CharPosition(1);
                    } else {
                        let currsor_byte_pos = text_field.get_byte_position(current_cursor);
                        text_field.text.insert(currsor_byte_pos, ' ');
                        current_cursor = CharPosition(currsor_byte_pos + 1);
                    }

                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Backspace => {
                    need_render = true;

                    if let Some((start, end)) = text_field.selection_range() {
                        let start_byte_pos = text_field.get_byte_position(start);
                        let end_byte_pos = text_field.get_byte_position(end);
                        text_field
                            .text
                            .replace_range(start_byte_pos..end_byte_pos, "");
                        current_cursor = start;
                    } else if current_cursor > CharPosition(0) {
                        let currsor_byte_pos = text_field.get_byte_position(current_cursor);
                        let prev_char_index = text_field.text[..currsor_byte_pos]
                            .chars()
                            .next_back()
                            .map(char::len_utf8)
                            .unwrap_or(0);
                        text_field.text.remove(currsor_byte_pos - prev_char_index);
                        current_cursor = CharPosition(currsor_byte_pos - 1);
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Delete => {
                    need_render = true;
                    if let Some((start, end)) = text_field.selection_range() {
                        let start_byte_pos = text_field.get_byte_position(start);
                        let end_byte_pos = text_field.get_byte_position(end);
                        text_field
                            .text
                            .replace_range(start_byte_pos..end_byte_pos, "");
                        current_cursor = start;
                    } else if current_cursor < CharPosition(text_field.text.chars().count()) {
                        let currsor_byte_pos = text_field.get_byte_position(current_cursor);
                        text_field.text.remove(currsor_byte_pos);
                        current_cursor = CharPosition(currsor_byte_pos);
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Character(c) => {
                    if key_states.pressed(KeyCode::ControlLeft) {
                        continue; // ignore control characters
                    }
                    let mut chars = c.chars().collect::<Vec<_>>();
                    need_render = true;

                    if let Some((start, end)) = text_field.selection_range() {
                        let start_byte_pos = text_field.get_byte_position(start);
                        let end_byte_pos = text_field.get_byte_position(end);
                        text_field.text.replace_range(
                            start_byte_pos..end_byte_pos,
                            chars.iter().collect::<String>().as_str(),
                        );
                        current_cursor = start + CharPosition(chars.len());
                    } else {
                        for c in chars {
                            let currsor_byte_pos = text_field.get_byte_position(current_cursor);
                            text_field.text.insert(currsor_byte_pos, c);
                            current_cursor = CharPosition(currsor_byte_pos + c.len_utf8());
                        }
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::ArrowLeft => {
                    if current_cursor > 0 {
                        if key_states.pressed(KeyCode::ShiftLeft) {
                            if text_field.selection_start.is_none() {
                                text_field.selection_start = Some(current_cursor);
                            }
                        } else {
                            text_field.selection_start = None;
                        }
                        current_cursor = current_cursor - 1;
                        need_render = true;
                    }
                }
                Key::ArrowRight => {
                    if current_cursor < CharPosition(text_field.text.chars().count()) {
                        if key_states.pressed(KeyCode::ShiftLeft) {
                            if text_field.selection_start.is_none() {
                                text_field.selection_start = Some(current_cursor);
                            }
                        } else {
                            text_field.selection_start = None;
                        }

                        current_cursor = current_cursor + 1;
                        need_render = true;
                    }
                }
                _ => {}
            }
        }
    }

    if need_render {
        // cursor position changed hided in if to pervert infinite change triggering
        text_field.cursor_position = Some(current_cursor);
        commands.trigger_targets(RenderWidget::show_cursor(), entity);
    }
}

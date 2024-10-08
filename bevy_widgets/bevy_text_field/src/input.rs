//! Contains major input logic for text field
//!

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_focus::{Focus, GotFocus, LostFocus};

use crate::{clipboard::BevyClipboard, render::RenderTextField, LineTextField, LineTextFieldLinks};

pub(crate) fn text_field_on_over(
    over: Trigger<Pointer<Over>>,
    mut commands: Commands,
    q_text_fields: Query<&LineTextField>,
) {
    let entity = over.entity();

    // Observers attached to child of LineTextField, so we must get text field entity from parent componeny
    let Ok(_) = q_text_fields.get(entity) else {
        return;
    };

    commands.trigger_targets(RenderTextField::default(), entity);
}

pub(crate) fn text_field_on_out(
    out: Trigger<Pointer<Out>>,
    mut commands: Commands,
    q_text_fields: Query<&LineTextField>,
) {
    let entity = out.entity();
    // Observers attached to child of LineTextField, so we must get text field entity from parent componeny
    let Ok(_) = q_text_fields.get(entity) else {
        return;
    };

    commands.trigger_targets(RenderTextField::default(), entity);
}

pub(crate) fn text_field_on_click(
    click: Trigger<GotFocus>,
    mut commands: Commands,
    mut q_text_fields: Query<(&mut LineTextField, &LineTextFieldLinks)>,
    q_nodes: Query<(&GlobalTransform, &Node)>,
    pressed_keys: Res<ButtonInput<KeyCode>>,
) {
    let entity = click.entity();
    let click_data = &click.event().0;

    let Ok((mut text_field, links)) = q_text_fields.get_mut(entity) else {
        return;
    };
    info!("Click: {:?}", click.entity());

    let mut char_cursor_pos = text_field.text.chars().count();
    // If we got focus by mouse click, we need to calculate cursor position
    if let Some(click_data) = click_data {
        if let Ok((pos, text_left)) = q_nodes.get(links.text) {
            let rect = text_left.logical_rect(pos);
            if rect.contains(click_data.pointer_location.position) {
                let dx = click_data.pointer_location.position.x - rect.min.x;
                let dx_relative = dx / rect.width();

                if let Some(cursor) = text_field.cursor_position {
                    let char_cursor = text_field.text[..cursor].chars().count();
                    char_cursor_pos = (char_cursor as f32 * dx_relative).round() as usize;
                } else {
                    char_cursor_pos =
                        (dx_relative * text_field.text.chars().count() as f32).round() as usize;
                }
            }
        }

        if let Ok((pos, text_right)) = q_nodes.get(links.text_right) {
            let rect = text_right.logical_rect(pos);
            if rect.contains(click_data.pointer_location.position) {
                let dx = click_data.pointer_location.position.x - rect.min.x;
                let dx_relative = dx / rect.width();

                if let Some(cursor) = text_field.cursor_position {
                    let char_cursor = text_field.text[..cursor].chars().count();
                    let text_right_width = text_field.text.chars().count() - char_cursor;
                    let relative_cursor = (dx_relative * text_right_width as f32).round() as usize;
                    char_cursor_pos = char_cursor + relative_cursor;
                } else {
                    // Unexpected
                }
            }
        }
    }

    if pressed_keys.pressed(KeyCode::ShiftLeft) {
        if text_field.selection_start.is_none() {
            if let Some(last_cursor) = text_field.cursor_position {
                text_field.selection_start = Some(last_cursor); // selection by mouse
            }
        }
    } else {
        text_field.selection_start = None;
    }

    let cursor_pos = text_field
        .text
        .char_indices()
        .nth(char_cursor_pos)
        .map(|(i, _)| i)
        .unwrap_or(text_field.text.len());
    text_field.cursor_position = Some(cursor_pos);

    commands.trigger_targets(
        RenderTextField {
            force_show_cursor: true,
        },
        entity,
    );
}

pub(crate) fn lost_focus(
    lost_focus: Trigger<LostFocus>,
    mut commands: Commands,
    mut q_text_fields: Query<(&mut LineTextField, &LineTextFieldLinks)>,
) {
    let entity = lost_focus.entity();
    info!("Lost focus {:?}", entity);
    let Ok((mut text_field, _)) = q_text_fields.get_mut(entity) else {
        return;
    };

    text_field.cursor_position = None;
    text_field.selection_start = None;
    commands.trigger_targets(RenderTextField::default(), entity);
}

pub(crate) fn keyboard_input(
    mut commands: Commands,
    mut q_text_fields: Query<(Entity, &mut LineTextField), With<Focus>>,
    mut events: EventReader<KeyboardInput>,
    key_states: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<BevyClipboard>,
) {
    let Ok((entity, mut text_field)) = q_text_fields.get_single_mut() else {
        return;
    };

    let Some(mut current_cursor) = text_field.cursor_position.clone() else {
        return;
    };
    current_cursor = current_cursor.clamp(0, text_field.text.len());

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
                        text_field.text.insert_str(current_cursor, &text);
                        current_cursor += text.len();
                    } else {
                        let selected_range = text_field.get_selected_range().unwrap();

                        text_field
                            .text
                            .replace_range(selected_range.0..selected_range.1, &text);
                        current_cursor = selected_range.0 + text.len();

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
            if let Some(selected_range) = text_field.get_selected_range() {
                text_field
                    .text
                    .replace_range(selected_range.0..selected_range.1, "");
                current_cursor = selected_range.0;
                text_field.selection_start = None;
            }
        } else if key_states.pressed(KeyCode::KeyX) {
            events.clear(); // clear events that were triggered by pasting (for example it can be holded and we need to process it only once)
        } else if key_states.just_pressed(KeyCode::KeyA) {
            // Select all text
            need_render = true;
            text_field.selection_start = Some(0);
            text_field.cursor_position = Some(text_field.text.len());
            current_cursor = text_field.text.len();
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

                    if let Some((start, end)) = text_field.get_selected_range() {
                        text_field.text.replace_range(start..end, " ");
                        current_cursor = start + 1;
                    } else {
                        text_field.text.insert(current_cursor, ' ');
                        current_cursor += 1;
                    }

                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Backspace => {
                    need_render = true;

                    if let Some((start, end)) = text_field.get_selected_range() {
                        text_field.text.replace_range(start..end, "");
                        current_cursor = start;
                    } else if current_cursor > 0 {
                        let prev_char_index = text_field.text[..current_cursor]
                            .chars()
                            .next_back()
                            .map(char::len_utf8)
                            .unwrap_or(0);
                        text_field.text.remove(current_cursor - prev_char_index);
                        current_cursor -= prev_char_index;
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Delete => {
                    need_render = true;
                    if let Some((start, end)) = text_field.get_selected_range() {
                        text_field.text.replace_range(start..end, "");
                        current_cursor = start;
                    } else if current_cursor < text_field.text.len() {
                        text_field.text.remove(current_cursor);
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Character(c) => {
                    if key_states.pressed(KeyCode::ControlLeft) {
                        continue; // ignore control characters
                    }
                    let mut chars = c.chars().collect::<Vec<_>>();
                    if let Some(allowed) = text_field.allowed_chars.as_ref() {
                        chars = chars
                            .iter()
                            .map(|c| *c)
                            .filter(|c| allowed.contains(c))
                            .collect();
                    }
                    need_render = true;

                    if let Some((start, end)) = text_field.get_selected_range() {
                        text_field
                            .text
                            .replace_range(start..end, chars.iter().collect::<String>().as_str());
                        current_cursor = start + c.len();
                    } else {
                        for c in chars {
                            text_field.text.insert(current_cursor, c);
                            current_cursor += c.len_utf8();
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
                        if key_states.pressed(KeyCode::ShiftLeft) {
                            if text_field.selection_start.is_none() {
                                text_field.selection_start = Some(current_cursor);
                            }
                        } else {
                            text_field.selection_start = None;
                        }

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
    }

    if need_render {
        // cursor position changed hided in if to pervert infinite change triggering
        text_field.cursor_position = Some(current_cursor);
        commands.trigger_targets(
            RenderTextField {
                force_show_cursor: true,
            },
            entity,
        );
    }
}

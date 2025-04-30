use crate::text_change::TextChange;

use super::*;
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_clipboard::BevyClipboard;
use bevy_focus::{Focus, LostFocus};

pub fn on_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut q_editable_texts: Query<(&mut EditableTextLine, &mut EditableTextInner)>,
    q_texts: Query<(&ComputedNode, &GlobalTransform)>,
    key_states: Res<ButtonInput<KeyCode>>,
) {
    let entity = click.target();
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
    inner.skip_cursor_overflow_check = true;

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
    let Ok((entity, mut text_field)) = q_text_fields.single_mut() else {
        return;
    };

    let Some(mut current_cursor) = text_field.cursor_position else {
        return;
    };
    current_cursor.0 = current_cursor.0.clamp(0, text_field.text.chars().count());

    let mut need_render = false;
    let mut text_change = TextChange::nop_change();

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
                Ok(mut text) => {
                    if let Some(allowed_chars) = &text_field.allowed_chars {
                        text.retain(|c| allowed_chars.contains(&c));
                    }
                    if text_field.selection_start.is_none() {
                        text_change = TextChange::insert_change(current_cursor, text.clone());
                        current_cursor = CharPosition(current_cursor.0 + text.chars().count());
                    } else {
                        let selected_range = text_field.selection_range().unwrap();
                        text_change = TextChange::new(selected_range, text.clone());
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
                text_change = TextChange::remove_change(selected_range);
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

                    if let Some(allowed_chars) = &text_field.allowed_chars {
                        if !allowed_chars.contains(&' ') {
                            continue;
                        }
                    }

                    if let Some((start, end)) = text_field.selection_range() {
                        text_change = TextChange::new((start, end), " ");
                        current_cursor = start + CharPosition(1);
                    } else {
                        text_change = TextChange::insert_change(current_cursor, " ");
                        current_cursor = CharPosition(current_cursor.0 + 1);
                    }

                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Backspace => {
                    need_render = true;

                    if let Some((start, end)) = text_field.selection_range() {
                        text_change = TextChange::remove_change((start, end));
                        current_cursor = start;
                    } else if current_cursor > CharPosition(0) {
                        text_change = TextChange::remove_change((
                            current_cursor - CharPosition(1),
                            current_cursor,
                        ));
                        current_cursor = current_cursor - 1;
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Delete => {
                    need_render = true;
                    if let Some((start, end)) = text_field.selection_range() {
                        text_change = TextChange::remove_change((start, end));
                        current_cursor = start;
                    } else if current_cursor < CharPosition(text_field.text.chars().count()) {
                        text_change = TextChange::remove_change((
                            current_cursor,
                            current_cursor + CharPosition(1),
                        ));
                    }
                    text_field.selection_start = None; // clear selection if we write any text
                }
                Key::Character(c) => {
                    if key_states.pressed(KeyCode::ControlLeft) {
                        continue; // ignore control characters
                    }
                    let mut chars = c.chars().collect::<Vec<_>>();
                    if let Some(allowed_chars) = &text_field.allowed_chars {
                        chars.retain(|c| allowed_chars.contains(c));
                    }
                    need_render = true;

                    if let Some((start, end)) = text_field.selection_range() {
                        text_change =
                            TextChange::new((start, end), chars.iter().collect::<String>());
                        current_cursor = start + CharPosition(chars.len());
                    } else {
                        text_change = TextChange::insert_change(
                            current_cursor,
                            chars.iter().collect::<String>(),
                        );
                        current_cursor = current_cursor + CharPosition(chars.len());
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
        let old_cursor_position = text_field.cursor_position;
        // cursor position changed hided in if to pervert infinite change triggering
        text_field.cursor_position = Some(current_cursor);
        commands.trigger_targets(RenderWidget::show_cursor(), entity);

        if text_field.text != text_change.new_text {
            // Send the text change event with the new text
            let mut new_text = text_field.text.clone();
            text_change.apply(&mut new_text);
            commands.trigger_targets(
                TextChanged {
                    change: text_change.clone(),
                    new_text,
                    old_cursor_position,
                    new_cursor_position: Some(current_cursor),
                },
                entity,
            );

            // If the text field is not controlled, apply the text change to the text field
            if !text_field.controlled_widget {
                text_change.apply(&mut text_field.text);
            }
        }
    }
}

pub fn on_focus_lost(
    trigger: Trigger<LostFocus>,
    mut commands: Commands,
    mut q_editable_texts: Query<&mut EditableTextLine>,
) {
    let entity = trigger.target();
    let Ok(mut text_field) = q_editable_texts.get_mut(entity) else {
        return;
    };

    text_field.cursor_position = None;
    text_field.selection_start = None;

    info!("Focus lost from {:?}", entity);

    commands.trigger_targets(RenderWidget::default(), entity);
}

pub fn on_set_cursor_position(
    trigger: Trigger<SetCursorPosition>,
    mut commands: Commands,
    mut q_editable_texts: Query<&mut EditableTextLine>,
) {
    let entity = trigger.target();
    let Ok(mut text_field) = q_editable_texts.get_mut(entity) else {
        return;
    };

    text_field.cursor_position = Some(trigger.0);
    commands.trigger_targets(RenderWidget::show_cursor(), entity);
}

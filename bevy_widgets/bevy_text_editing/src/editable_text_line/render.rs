use super::*;
use bevy::prelude::*;

pub fn render_system(
    trigger: On<RenderWidget>,
    mut commands: Commands,
    q_editable_texts: Query<(&EditableTextLine, &EditableTextInner)>,
    mut q_texts: Query<&mut Text>,
    q_cursors: Query<Entity, With<Cursor>>,
) {
    for (text_line, inner) in q_editable_texts.iter() {
        let Ok(mut text) = q_texts.get_mut(inner.text) else {
            continue;
        };

        // info!("Render {} with text \"{}\"", entity, text_line.text);
        // info!("Cursor position: {:?}", text_line.cursor_position);
        // info!("Selection range: {:?}", text_line.selection_range());

        // Change text to stored in text line state
        text.0 = text_line.text.clone();

        // Render cursor
        if let Some(cursor_pos) = text_line.cursor_position {
            let Ok(mut cursor_fake_text) = q_texts.get_mut(inner.fake_cursor_text) else {
                continue;
            };
            cursor_fake_text.0 = text_line
                .get_text_range((CharPosition(0), cursor_pos))
                .unwrap_or_default();

            if !q_cursors.contains(inner.cursor) {
                commands.entity(inner.cursor).insert((
                    Cursor::default(),
                    Visibility::Visible,
                    BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                ));
            }

            if trigger.show_cursor {
                commands
                    .entity(inner.cursor)
                    .insert(Cursor::default())
                    .insert(Visibility::Visible);
            }
        } else {
            commands
                .entity(inner.cursor)
                .remove::<Cursor>()
                .insert(Visibility::Hidden);
        }

        // Render selection
        if let Some((selection_start, selection_end)) = text_line.selection_range() {
            let Ok(mut fake_text_before_selection) =
                q_texts.get_mut(inner.fake_text_before_selection)
            else {
                continue;
            };

            let start_byte_pos = text_line.get_byte_position(selection_start);
            let end_byte_pos = text_line.get_byte_position(selection_end);

            fake_text_before_selection.0 = text_line.text[0..start_byte_pos].to_string();

            let Ok(mut fake_selection_text) = q_texts.get_mut(inner.fake_selection_text) else {
                continue;
            };
            fake_selection_text.0 = text_line.text[start_byte_pos..end_byte_pos].to_string();
            commands
                .entity(inner.fake_selection_text)
                .insert(Visibility::Visible);
            info!("Rendered selection with text \"{}\"", fake_selection_text.0);
        } else {
            commands
                .entity(inner.fake_selection_text)
                .insert(Visibility::Hidden);
        }
    }
}

pub(crate) fn set_cursor_pos(
    q_text_fields: Query<(&EditableTextLine, &EditableTextInner)>,
    mut q_nodes: Query<&mut Node>,
    q_computed_nodes: Query<&ComputedNode>,
) {
    for (text_field, inner) in q_text_fields.iter() {
        if text_field.cursor_position.is_some() {
            let Ok(mut cursor_node) = q_nodes.get_mut(inner.cursor) else {
                continue;
            };

            let Ok(fake_cursor_text_node) = q_computed_nodes.get(inner.fake_cursor_text) else {
                continue;
            };

            cursor_node.left = Val::Px(fake_cursor_text_node.size().x);
        }
    }
}

pub(crate) fn check_cursor_overflow(
    mut commands: Commands,
    mut q_text_fields: Query<(Entity, &EditableTextLine, &mut EditableTextInner)>,
    q_transforms: Query<&UiGlobalTransform>,
    mut q_nodes: Query<&mut Node>,
    q_computed_nodes: Query<&ComputedNode>,
) {
    for (entity, text_field, mut inner) in q_text_fields.iter_mut() {
        let Ok(text_field_node) = q_computed_nodes.get(entity) else {
            return;
        };
        let Ok(text_field_transform) = q_transforms.get(entity) else {
            return;
        };

        let Ok(mut canvas_node) = q_nodes.get_mut(inner.canvas) else {
            return;
        };

        if inner.skip_cursor_overflow_check {
            inner.skip_cursor_overflow_check = false;
            continue;
        }

        if text_field.cursor_position.is_some() {
            let Ok(cursor_transform) = q_transforms.get(inner.cursor) else {
                return;
            };

            // Check that we have computed size of nodes
            // Check that we can see the cursor
            let padding = 10.0;

            // info!(
            //     "Cursor dpos: {:?}",
            //     cursor_transform.translation().x - text_field_transform.translation().x
            // );

            if (cursor_transform.translation.x - text_field_transform.translation.x)
                > text_field_node.size().x / 2.0 - padding
            {
                //Debug info ith all values
                info!(
                    "{} {}",
                    cursor_transform.translation.x, text_field_transform.translation.x
                );
                info!(
                    "{} {}",
                    text_field_node.size().x,
                    text_field_node.size().x / 2.0
                );

                inner.text_shift += cursor_transform.translation.x
                    - text_field_transform.translation.x
                    - text_field_node.size().x / 2.0
                    + padding;
                canvas_node.left = Val::Px(-inner.text_shift);
                commands.trigger(RenderWidget::hide_cursor(entity));
            } else if (cursor_transform.translation.x - text_field_transform.translation.x)
                < -text_field_node.size().x / 2.0 + padding
            {
                inner.text_shift += cursor_transform.translation.x
                    - text_field_transform.translation.x
                    + text_field_node.size().x / 2.0
                    - padding;
                canvas_node.left = Val::Px(-inner.text_shift);
                commands.trigger(RenderWidget::hide_cursor(entity));
            }
        } else if inner.text_shift != 0.0 {
            info!("Reset shift {:?}", entity);
            inner.text_shift = 0.0;
            canvas_node.left = Val::Px(0.0);
            commands.trigger(RenderWidget::hide_cursor(entity));
        }
    }
}

use super::*;
use bevy::prelude::*;

pub fn render_system(
    trigger: Trigger<RenderWidget>,
    mut commands: Commands,
    q_editable_texts: Query<(Entity, &EditableTextLine, &EditableTextInner)>,
    mut q_nodes: Query<&mut Node>,
    mut q_texts: Query<&mut Text>,
    q_cursors: Query<Entity, With<Cursor>>,
) {
    for (entity, text_line, inner) in q_editable_texts.iter() {
        let Ok(mut text) = q_texts.get_mut(inner.text) else {
            continue;
        };

        info!("Render {} with text \"{}\"", entity, text_line.text);
        info!("Cursor position: {:?}", text_line.cursor_position);
        info!("Selection range: {:?}", text_line.selection_range());

        // Change text to stored in text line state
        text.0 = text_line.text.clone();

        // Render cursor
        if let Some(cursor_pos) = text_line.cursor_position {
            let Ok(mut cursor_fake_text) = q_texts.get_mut(inner.fake_cursor_text) else {
                continue;
            };
            cursor_fake_text.0 = text_line
                .get_text_range((CharPosition(0), cursor_pos))
                .unwrap();

            if !q_cursors.contains(inner.cursor) {
                commands.entity(inner.cursor).insert((
                    Cursor::default(),
                    Visibility::Visible,
                    BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                ));
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

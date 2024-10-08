//! Contains code for rendering text fields.
//!

#![allow(clippy::too_many_arguments)]

use crate::{
    cursor::Cursor, InnerFieldParams, LineTextField, LineTextFieldLinks, TextChanged, BORDER_COLOR,
    HIGHLIGHT_COLOR,
};
use bevy::prelude::*;
use bevy_editor_styles::Theme;

/// Trigger for rerender text field
#[derive(Event, Default)]
pub struct RenderTextField {
    /// Force show cursor (disable cursor blinking if spamming with this event)
    pub force_show_cursor: bool,
}

#[derive(Component)]
pub(crate) struct SkipCursorCheck(pub(crate) usize);

pub(crate) fn trigger_render_on_change(
    mut commands: Commands,
    mut q_fields: Query<(Entity, &LineTextField, &mut InnerFieldParams), Changed<LineTextField>>,
) {
    if q_fields.is_empty() {
        return;
    }

    for (entity, text_field, mut inner_field_param) in q_fields.iter_mut() {
        commands.trigger_targets(RenderTextField::default(), entity);
        if inner_field_param.last_text != text_field.text() {
            commands.trigger_targets(TextChanged, entity);
            inner_field_param.last_text = text_field.text().to_string();
        }
    }
}

pub(crate) fn render_text_field(
    mut trigger: Trigger<RenderTextField>,
    mut commands: Commands,
    q_text_fields: Query<(
        &LineTextField,
        &LineTextFieldLinks,
        &InnerFieldParams,
        &Interaction,
    )>,
    mut q_border_color: Query<&mut BorderColor>,
    mut q_styles: Query<&mut Style>,
    mut q_texts: Query<&mut Text>,
    q_cursors: Query<Entity, With<Cursor>>,
    theme: Option<Res<Theme>>,
) {
    let text_style = if let Some(theme) = theme {
        theme.normal_text_style()
    } else {
        TextStyle::default()
    };
    let entity = trigger.entity();
    trigger.propagate(false);

    let Ok((text_field, links, inner, highlighted)) = q_text_fields.get(entity) else {
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

    let Ok(mut sub_canvas_style) = q_styles.get_mut(links.sub_canvas) else {
        return;
    };

    let Ok([mut selection_text, mut pre_selection_text]) =
        q_texts.get_many_mut([links.selection, links.selection_shift])
    else {
        return;
    };

    *canvas_border_color = BorderColor(border_color);

    if let Some(cursor) = text_field.cursor_position {
        let cursor = cursor.clamp(0, text_field.text.len());
        let (left_text, right_text) = text_field.text.split_at(cursor);
        info!("Text {} | {}", left_text, right_text);

        info!("Shift {:?}", inner.text_shift);
        sub_canvas_style.left = Val::Px(-inner.text_shift);

        commands
            .entity(links.text)
            .insert(TextBundle::from_section(left_text, text_style.clone()).with_no_wrap());

        if !q_cursors.contains(links.cursor) {
            // If we spawn new cursor than we need to skip checks for cursor overflow for some frames needed to compute correct cursor position by bevy systems
            commands.entity(links.cursor).insert(SkipCursorCheck(2));
        }

        commands.entity(links.cursor).insert((
            Style {
                height: Val::Percent(100.0),
                width: Val::Px(2.0),
                margin: UiRect::horizontal(Val::Px(-1.0)),
                ..default()
            },
            BackgroundColor(Color::WHITE),
        ));

        if trigger.event().force_show_cursor {
            commands
                .entity(links.cursor)
                .insert(Cursor::default())
                .insert(Visibility::Visible);
        } else {
            commands
                .entity(links.cursor)
                .insert_if_new(Cursor::default());
        }

        commands
            .entity(links.text_right)
            .insert(TextBundle::from_section(right_text, text_style.clone()).with_no_wrap());

        if let Some(selection_start) = text_field.selection_start {
            // Show text selection
            let selection_text_part = if selection_start < cursor {
                &text_field.text[selection_start..cursor]
            } else {
                &text_field.text[cursor..selection_start]
            };

            let selection_pre = if selection_start < cursor {
                &text_field.text[..selection_start]
            } else {
                &text_field.text[..cursor]
            };

            // Set fake text
            selection_text.sections[0].value = selection_text_part.to_string();
            pre_selection_text.sections[0].value = selection_pre.to_string();

            selection_text.sections[0].style.font = text_style.font.clone();
            selection_text.sections[0].style.font_size = text_style.font_size;
            pre_selection_text.sections[0].style.font = text_style.font.clone();
            pre_selection_text.sections[0].style.font_size = text_style.font_size;
        } else {
            selection_text.sections[0].value = "".to_string();
            pre_selection_text.sections[0].value = "".to_string();
        }
    } else {
        commands.entity(links.text).insert(
            TextBundle::from_section(text_field.text.clone(), text_style.clone()).with_no_wrap(),
        );
        commands
            .entity(links.cursor)
            .insert(NodeBundle::default())
            .remove::<Cursor>();
        commands
            .entity(links.text_right)
            .insert(TextBundle::from_section("", text_style.clone()));

        selection_text.sections[0].value = "".to_string();
        pre_selection_text.sections[0].value = "".to_string();
    }
}

pub(crate) fn update_skip_cursor_check(
    mut commands: Commands,
    mut q_skip_check: Query<(Entity, &mut SkipCursorCheck)>,
) {
    for (entity, mut skip_check) in q_skip_check.iter_mut() {
        skip_check.0 -= 1;
        if skip_check.0 == 0 {
            commands.entity(entity).remove::<SkipCursorCheck>();
        }
    }
}

pub(crate) fn check_cursor_overflow(
    mut commands: Commands,
    mut q_text_fields: Query<(
        Entity,
        &LineTextField,
        &LineTextFieldLinks,
        &mut InnerFieldParams,
    )>,
    q_nodes: Query<&Node>,
    q_transforms: Query<&GlobalTransform>,
    q_cursors: Query<&GlobalTransform, Without<SkipCursorCheck>>,
    mut q_styles: Query<&mut Style>,
) {
    for (entity, text_field, links, mut inner) in q_text_fields.iter_mut() {
        let Ok(text_field_node) = q_nodes.get(entity) else {
            return;
        };
        let Ok(text_field_transform) = q_transforms.get(entity) else {
            return;
        };

        let Ok(mut sub_canvas_style) = q_styles.get_mut(links.sub_canvas) else {
            return;
        };

        let Ok(cursor_node) = q_nodes.get(links.cursor) else {
            return;
        };

        if text_field.cursor_position.is_some() {
            let Ok(cursor_transform) = q_cursors.get(links.cursor) else {
                return;
            };

            // Check that we have computed size of nodes
            if cursor_node.size().x != 0.0 {
                // Check that we can see the cursor
                let padding = 10.0;

                if (cursor_transform.translation().x - text_field_transform.translation().x)
                    > text_field_node.size().x / 2.0 - padding
                {
                    //Debug info ith all values
                    info!(
                        "{} {}",
                        cursor_transform.translation().x,
                        text_field_transform.translation().x
                    );
                    info!(
                        "{} {}",
                        text_field_node.size().x,
                        text_field_node.size().x / 2.0
                    );

                    inner.text_shift += cursor_transform.translation().x
                        - text_field_transform.translation().x
                        - text_field_node.size().x / 2.0
                        + padding;
                    sub_canvas_style.left = Val::Px(-inner.text_shift);
                    commands.trigger_targets(RenderTextField::default(), entity);
                } else if (cursor_transform.translation().x - text_field_transform.translation().x)
                    < -text_field_node.size().x / 2.0 + padding
                {
                    inner.text_shift += cursor_transform.translation().x
                        - text_field_transform.translation().x
                        + text_field_node.size().x / 2.0
                        - padding;
                    sub_canvas_style.left = Val::Px(-inner.text_shift);
                    commands.trigger_targets(RenderTextField::default(), entity);
                }
            }
        } else if inner.text_shift != 0.0 {
            info!("Reset shift {:?}", entity);
            inner.text_shift = 0.0;
            sub_canvas_style.left = Val::Px(0.0);
            commands.trigger_targets(RenderTextField::default(), entity);
        }
    }
}

use bevy::prelude::*;
use bevy_editor_styles::Theme;

use crate::{
    ui::{spawn_divider, spawn_pane, spawn_resize_handle},
    Divider, PaneRootNode, RootPaneLayoutNode, Size,
};

/// Middle clicking removes the pane.
pub(crate) fn on_pane_header_middle_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    parent_query: Query<&Parent>,
    children_query: Query<&Children>,
    root_query: Query<(), With<RootPaneLayoutNode>>,
    mut size_query: Query<&mut Size>,
) {
    if trigger.event().button != PointerButton::Middle {
        return;
    }

    // Grab the id of the pane root
    let target = parent_query
        .iter_ancestors(trigger.entity())
        .nth(1)
        .unwrap();

    let parent = parent_query.get(target).unwrap().get();

    // Prevent the removal of the last panel
    if root_query.contains(parent) {
        return;
    }

    // Find the index of this pane among its siblings
    let siblings = children_query.get(parent).unwrap();
    let index = siblings
        .iter()
        .position(|entity| *entity == target)
        .unwrap();

    let size = size_query.get(target).unwrap().0;

    let not_first_child = index != 0;

    let a = not_first_child.then(|| siblings.get(index - 2)).flatten();
    let b = siblings.get(index + 2);

    match (a, b) {
        (None, None) => unreachable!(),
        (None, Some(e)) | (Some(e), None) => {
            size_query.get_mut(*e).unwrap().0 += size;
        }
        (Some(a), Some(b)) => {
            size_query.get_mut(*a).unwrap().0 += size / 2.;
            size_query.get_mut(*b).unwrap().0 += size / 2.;
        }
    }

    // Despawn the resize handle next to this pane
    let resize_handle_index = if not_first_child { index - 1 } else { 1 };
    commands
        .entity(siblings[resize_handle_index])
        .despawn_recursive();
    // Despawn this pane
    commands.entity(target).despawn_recursive();
}

/// Right clicking dividers the pane horizontally
/// Holding left shift and right clicking dividers the pane vertically
pub(crate) fn on_pane_header_right_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    theme: Res<Theme>,
    divider_query: Query<&Divider>,
    pane_root_query: Query<&PaneRootNode>,
    mut size_query: Query<&mut Size>,
    children_query: Query<&Children>,
    parent_query: Query<&Parent>,
) {
    if trigger.event().button != PointerButton::Secondary {
        return;
    }

    let divider = if input.pressed(KeyCode::ShiftLeft) {
        Divider::Vertical
    } else {
        Divider::Horizontal
    };

    // Grab the id of the pane root
    let target = parent_query
        .iter_ancestors(trigger.entity())
        .nth(1)
        .unwrap();

    let pane = pane_root_query.get(target).unwrap();

    let parent = parent_query.get(target).unwrap().get();

    // Find the index of this pane among its siblings
    let siblings = children_query.get(parent).unwrap();
    let index = siblings
        .iter()
        .position(|entity| *entity == target)
        .unwrap();

    // Parent has a matching divider direction
    let matching_direction = divider_query
        .get(parent)
        .map(|parent_divider| *parent_divider == divider)
        .unwrap_or(false);

    let mut size = size_query.get_mut(target).unwrap();
    let new_size = if matching_direction { size.0 / 2. } else { 0.5 };

    // TODO The new pane should inherit the state of the existing pane
    let new_pane = spawn_pane(&mut commands, &theme, new_size, &pane.name).id();

    let resize_handle = spawn_resize_handle(&mut commands, divider).id();

    if matching_direction {
        commands
            .entity(parent)
            .insert_children(index + 1, &[resize_handle, new_pane]);
    } else {
        let divider = spawn_divider(&mut commands, divider, size.0)
            .add_children(&[target, resize_handle, new_pane])
            .id();
        commands.entity(parent).insert_children(index, &[divider]);
    }
    size.0 = new_size;
}

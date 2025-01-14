use bevy::prelude::*;

use crate::{
    prelude::DividerCommandsQuery,
    ui::{
        pane::PaneNode,
        pane_group::{PaneGroup, PaneGroupCommandsQuery},
    },
};

pub(crate) fn remove_pane(
    target_pane_id: In<Entity>,
    pane_query: Query<&PaneNode>,
    mut pane_group_query: PaneGroupCommandsQuery,
) {
    // Grab the pane information
    let target_pane = pane_query.get(target_pane_id.0).unwrap();

    // Get the pane's group and remove the pane from it
    let mut pane_group = pane_group_query.get(target_pane.group).unwrap();
    pane_group.remove_pane(target_pane_id.0);
}

pub(crate) fn remove_pane_group(
    target_group_id: In<Entity>,
    mut divider_query: DividerCommandsQuery,
    parent_query: Query<&Parent, With<PaneGroup>>,
) {
    // Grab the divider commands for the divider containing this group
    let target_parent = parent_query.get(target_group_id.0).unwrap();
    let mut divider = divider_query.get(target_parent.get()).unwrap();

    // Remove the group from the divider.
    divider.remove(target_group_id.0);
}

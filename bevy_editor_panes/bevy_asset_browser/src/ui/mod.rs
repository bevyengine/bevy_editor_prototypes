//! Module for all the UI components of the Asset Browser

use bevy::prelude::*;
use bevy_editor_styles::Theme;
use bevy_pane_layout::PaneContentNode;

use crate::{AssetBrowserLocation, DirectoryContent};

pub mod directory_content;
mod nodes;
pub mod top_bar;

/// The root node for the asset browser.
#[derive(Component)]
pub struct AssetBrowserNode;

/// Spawn [`AssetBrowserNode`] once the pane is created
#[allow(clippy::too_many_arguments)]
pub fn on_pane_creation(
    trigger: Trigger<OnAdd, AssetBrowserNode>,
    mut commands: Commands,
    theme: Res<Theme>,
    children_query: Query<&Children>,
    content: Query<&PaneContentNode>,
    location: Res<AssetBrowserLocation>,
    asset_server: Res<AssetServer>,
    directory_content: Res<DirectoryContent>,
) {
    let pane_root = trigger.entity();
    let content_node = children_query
        .iter_descendants(pane_root)
        .find(|e| content.contains(*e))
        .unwrap();

    let asset_browser = commands
        .entity(content_node)
        .insert(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    top_bar::spawn_top_bar(&mut commands, &theme, &location).set_parent(asset_browser);
    directory_content::spawn_directory_content(
        &mut commands,
        &directory_content,
        &theme,
        &asset_server,
    )
    .set_parent(asset_browser);
}

pub(crate) const DEFAULT_SOURCE_ID_NAME: &str = "Default";

pub(crate) fn source_id_to_string(source_id: &crate::AssetSourceId) -> String {
    match source_id {
        crate::AssetSourceId::Default => DEFAULT_SOURCE_ID_NAME.to_string(),
        crate::AssetSourceId::Name(name) => name.to_string(),
    }
}

//! Module for all the UI components of the Asset Browser

use bevy::prelude::*;
use bevy_editor_styles::Theme;
use bevy_pane_layout::prelude::*;

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
    structure: In<PaneStructure>,
    mut commands: Commands,
    theme: Res<Theme>,
    location: Res<AssetBrowserLocation>,
    asset_server: Res<AssetServer>,
    directory_content: Res<DirectoryContent>,
) {
    let asset_browser = commands
        .entity(structure.content)
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
        &location,
    )
    .set_parent(asset_browser);

    commands.entity(structure.root).insert(AssetBrowserNode);
}

pub(crate) const DEFAULT_SOURCE_ID_NAME: &str = "Default";

pub(crate) fn source_id_to_string(source_id: &crate::AssetSourceId) -> String {
    match source_id {
        crate::AssetSourceId::Default => DEFAULT_SOURCE_ID_NAME.to_string(),
        crate::AssetSourceId::Name(name) => name.to_string(),
    }
}

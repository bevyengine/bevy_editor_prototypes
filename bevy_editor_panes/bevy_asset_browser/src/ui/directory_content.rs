use bevy::{asset::io::AssetSourceId, prelude::*};
use bevy_context_menu::{ContextMenu, ContextMenuOption};
use bevy_editor_styles::Theme;
use bevy_scroll_box::{spawn_scroll_box, ScrollBox, ScrollBoxContent};

use crate::{io, AssetBrowserLocation, DefaultSourceFilePath, DirectoryContent, Entry};

use crate::ui::nodes::{spawn_file_node, spawn_folder_node, spawn_source_node};

/// Spawn the directory content UI
pub(crate) fn spawn_directory_content<'a>(
    commands: &'a mut Commands,
    directory_content: &Res<DirectoryContent>,
    theme: &Res<Theme>,
    asset_server: &Res<AssetServer>,
) -> EntityCommands<'a> {
    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .id();
    spawn_scroll_box(
        commands,
        &theme,
        Overflow::scroll_y(),
        Some(|commands: &mut Commands, content_list: Entity| {
            commands
                .entity(content_list)
                .insert(ContextMenu::new([ContextMenuOption::new(
                    "Create Folder",
                    |mut commands, _entity| {
                        commands.run_system_cached(create_new_folder);
                    },
                )]));
            populate_directory_content(
                commands,
                content_list,
                directory_content,
                asset_server,
                theme,
            );
        }),
    )
    .insert(ContentBrowserScrollBox)
    .set_parent(root);
    commands.entity(root)
}

/// Tag for all the asset browser scroll boxes
#[derive(Component)]
pub(crate) struct ContentBrowserScrollBox;

/// Refresh the UI with the content of the current [`AssetBrowserLocation`]
pub(crate) fn refresh_ui(
    mut commands: Commands,
    content_list_query: Query<(Entity, Option<&Children>), With<ScrollBoxContent>>,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
    directory_content: Res<DirectoryContent>,
    mut query_scrollbox: Query<&mut ScrollBox, With<ContentBrowserScrollBox>>,
) {
    for (content_list_entity, content_list_children) in content_list_query.iter() {
        despawn_content_entries(&mut commands, content_list_entity, content_list_children);
        populate_directory_content(
            &mut commands,
            content_list_entity,
            &directory_content,
            &asset_server,
            &theme,
        );
    }
    // Reset scroll boxes
    for mut scrollbox in query_scrollbox.iter_mut() {
        scrollbox.scroll_to_top();
    }
}

/// Despawn all the content [entries](Entry)
fn despawn_content_entries(commands: &mut Commands, container: Entity, entries: Option<&Children>) {
    if let Some(entries) = entries {
        for entry in entries {
            commands.entity(*entry).despawn_recursive();
        }
        commands.entity(container).clear_children();
    }
}

/// Spawn all the content [entries](Entry) based on [`DirectoryContent`]
fn populate_directory_content(
    commands: &mut Commands,
    parent_entity: Entity,
    directory_content: &Res<DirectoryContent>,
    asset_server: &Res<AssetServer>,
    theme: &Res<Theme>,
) {
    for entry in &directory_content.0 {
        match entry {
            Entry::Source(id) => {
                spawn_source_node(commands, id, asset_server, theme).set_parent(parent_entity);
            }
            Entry::Folder(name) => {
                spawn_folder_node(commands, name.clone(), asset_server, theme)
                    .set_parent(parent_entity);
            }
            Entry::File(name) => {
                spawn_file_node(commands, name.clone(), asset_server, theme)
                    .set_parent(parent_entity);
            }
        }
    }
}

pub(crate) fn create_new_folder(
    mut commands: Commands,
    default_source_file_path: Res<DefaultSourceFilePath>,
    location: Res<AssetBrowserLocation>,
    directory_content: Res<DirectoryContent>,
) {
    if location.source_id.is_none() || location.source_id != Some(AssetSourceId::Default) {
        error!("Cannot create folder: Invalid source id, make sure your inside the Default source");
        return;
    }
    let mut path = default_source_file_path.0.clone();
    path.push(location.path.as_path());
    match io::create_new_folder(path) {
        Ok(folder_name) => {
            let mut updated_content = directory_content.0.clone();
            updated_content.push(Entry::Folder(folder_name));
            commands.insert_resource(DirectoryContent(updated_content));
        }
        Err(e) => eprintln!("Failed to create directory: {}", e),
    }
}

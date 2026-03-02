//! Contain function to spawn the different elements of the Asset Browser UI

use atomicow::CowArc;
use bevy::{
    asset::io::{AssetSource, AssetSourceBuilders, AssetSourceId},
    feathers::cursor::EntityCursor,
    prelude::*,
    window::SystemCursorIcon,
};
use bevy_context_menu::{ContextMenu, ContextMenuOption};
use bevy_editor_styles::Theme;

use crate::{AssetBrowserLocation, io, ui::source_id_to_string};

use super::{
    DEFAULT_SOURCE_ID_NAME,
    directory_content::{delete_file, delete_folder},
};

pub(crate) fn spawn_source_node<'a>(
    commands: &'a mut Commands,
    source_id: &AssetSourceId,
    asset_server: &Res<AssetServer>,
    theme: &Res<Theme>,
) -> EntityCommands<'a> {
    let base_node = spawn_base_node(commands, theme)
        .observe(
            move |trigger: On<Pointer<Release>>,
                  mut commands: Commands,
                  mut location: ResMut<AssetBrowserLocation>,
                  mut asset_source_builder: ResMut<AssetSourceBuilders>,
                  query_text: Query<&Text>,
                  query_children: Query<&Children>| {
                if trigger.event().button != PointerButton::Primary {
                    return;
                }
                let button = trigger.event().event_target();
                let button_children = query_children.get(button).unwrap();
                let source_name = &query_text
                    .get(button_children[1])
                    .expect("Child 1 of source node to have a text component")
                    .0;
                if source_name == DEFAULT_SOURCE_ID_NAME {
                    location.source_id = Some(AssetSourceId::Default);
                } else {
                    location.source_id = asset_source_builder
                        .build_sources(false, false)
                        .iter()
                        .find(|source| match source.id() {
                            AssetSourceId::Name(CowArc::Static(name)) => name == source_name,
                            _ => false,
                        })
                        .map(AssetSource::id);
                }
                commands.run_system_cached(io::task::fetch_directory_content);
            },
        )
        .id();

    // Icon
    commands.spawn((
        ImageNode::new(asset_server.load("embedded://bevy_asset_browser/assets/source_icon.png")),
        Node {
            height: Val::Px(50.0),
            ..default()
        },
        ChildOf(base_node),
    ));
    // Source Name
    commands.spawn((
        Text::new(source_id_to_string(source_id)),
        TextFont {
            font: theme.text.font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(theme.text.text_color),
        ChildOf(base_node),
    ));

    commands.entity(base_node)
}

pub(crate) fn spawn_folder_node<'a>(
    commands: &'a mut Commands,
    folder_name: String,
    asset_server: &Res<AssetServer>,
    location: &Res<AssetBrowserLocation>,
    theme: &Res<Theme>,
) -> EntityCommands<'a> {
    let base_node = {
        let mut ec = spawn_base_node(commands, theme);
        ec.observe(
            |trigger: On<Pointer<Release>>,
             mut commands: Commands,
             mut location: ResMut<AssetBrowserLocation>,
             query_text: Query<&Text>,
             query_children: Query<&Children>| {
                if trigger.event().button != PointerButton::Primary {
                    return;
                }
                let button = trigger.event().event_target();
                let button_children = query_children.get(button).unwrap();
                let folder_name = &query_text
                    .get(button_children[1])
                    .expect("Child 1 of folder node to have a text component")
                    .0;
                location.path.push(folder_name.clone());
                commands.run_system_cached(io::task::fetch_directory_content);
            },
        );
        if location.source_id == Some(AssetSourceId::Default) {
            ec.insert(ContextMenu::new([
                // ContextMenuOption::new("Rename", |mut commands, entity| {
                //     commands.run_system_cached_with(rename_asset, entity);
                // }),
                ContextMenuOption::new("Delete", |mut commands, entity| {
                    commands.run_system_cached_with(delete_folder, entity);
                }),
            ]));
        }
        ec.id()
    };

    // Icon
    commands.spawn((
        ImageNode::new(
            asset_server.load("embedded://bevy_asset_browser/assets/directory_icon.png"),
        ),
        Node {
            height: Val::Px(50.0),
            ..default()
        },
        ChildOf(base_node),
    ));
    // Folder Name
    commands.spawn((
        Text::new(folder_name),
        TextFont {
            font: theme.text.font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(theme.text.text_color),
        ChildOf(base_node),
    ));

    commands.entity(base_node)
}

pub(crate) fn spawn_file_node<'a>(
    commands: &'a mut Commands,
    file_name: String,
    asset_server: &Res<AssetServer>,
    location: &Res<AssetBrowserLocation>,
    theme: &Res<Theme>,
) -> EntityCommands<'a> {
    let base_node = {
        let mut ec = spawn_base_node(commands, theme);
        if location.source_id == Some(AssetSourceId::Default) {
            ec.insert(ContextMenu::new([
                // ContextMenuOption::new("Rename", |mut commands, entity| {
                //     commands.run_system_cached_with(rename_asset, entity);
                // }),
                ContextMenuOption::new("Delete", |mut commands, entity| {
                    commands.run_system_cached_with(delete_file, entity);
                }),
                // TODO: add this to the folders as well
                // TODO: fix this, doesn't yet work, it opens the file instead of revealing it in the file manager (at least on linux)
                // ContextMenuOption::new("Reveal in File Manager", |mut commands, entity| {
                //     commands.run_system_cached_with(reveal_in_file_manager, entity);
                // }),
            ]));
        }
        ec.id()
    };

    // Icon
    commands.spawn((
        ImageNode::new(asset_server.load("embedded://bevy_asset_browser/assets/file_icon.png")),
        Node {
            height: Val::Px(50.0),
            ..default()
        },
        ChildOf(base_node),
    ));
    // Folder Name
    commands.spawn((
        Text::new(file_name),
        TextFont {
            font: theme.text.font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(theme.text.text_color),
        ChildOf(base_node),
    ));

    commands.entity(base_node)
}

fn spawn_base_node<'a>(commands: &'a mut Commands, theme: &Res<Theme>) -> EntityCommands<'a> {
    commands.spawn((
        Button,
        Node {
            margin: UiRect::all(Val::Px(5.0)),
            padding: UiRect::all(Val::Px(5.0)),
            height: Val::Px(100.0),
            width: Val::Px(100.0),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(3.0)),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ZIndex(1),
        theme.general.border_radius,
        EntityCursor::System(SystemCursorIcon::Pointer),
    ))
}

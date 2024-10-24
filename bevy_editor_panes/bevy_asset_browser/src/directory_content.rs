use bevy::{
    asset::io::AssetSourceBuilders,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    tasks::{
        block_on,
        futures_lite::{future, StreamExt},
        IoTaskPool, Task,
    },
    window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};
use bevy_editor_styles::Theme;

use crate::{AssetBrowserLocation, AssetType};

/// The root node for the directory content view
#[derive(Component)]
pub struct DirectoryContentNode;

/// One entry of [`DirectoryContent`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetEntry {
    pub name: String,
    pub asset_type: AssetType,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct DirectoryContent(pub Vec<AssetEntry>);

#[derive(Component)]
/// The task that fetches the content of current [`AssetBrowserLocation`]
pub(crate) struct FetchDirectoryContentTask(Task<DirectoryContent>);

pub(crate) fn run_if_fetch_task_is_running(
    task_query: Query<(Entity, &FetchDirectoryContentTask)>,
) -> bool {
    task_query.iter().count() > 0
}

/// Poll the [`FetchDirectoryContentTask`] to check if it's done
/// If it's done, despawn the task entity and insert the result into [`DirectoryContent`]
pub(crate) fn poll_fetch_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut FetchDirectoryContentTask)>,
) {
    let (task_entity, mut task) = task_query.single_mut();
    if let Some(content) = block_on(future::poll_once(&mut task.0)) {
        commands.entity(task_entity).despawn();
        commands.insert_resource(content);
    }
}

/// Spawn a new IO [`FetchDirectoryContentTask`] to fetch the content of the current [`AssetBrowserLocation`]
pub fn fetch_directory_content(
    mut commands: Commands,
    mut asset_source_builder: ResMut<AssetSourceBuilders>,
    location: Res<AssetBrowserLocation>,
) {
    let sources = asset_source_builder.build_sources(false, false);
    if location.source_id.is_none() {
        commands.insert_resource(DirectoryContent(
            sources
                .iter()
                .map(|source| AssetEntry {
                    name: crate::asset_source_id_to_string(&source.id()),
                    asset_type: AssetType::EngineSource,
                })
                .collect(),
        ));
        return;
    }
    let location = location.clone();
    let task = IoTaskPool::get().spawn(async move {
        let source = sources.get(location.source_id.unwrap()).unwrap();
        let reader = source.reader();

        let mut content = DirectoryContent::default();
        let dir_stream = reader.read_directory(location.path.as_path()).await;
        if dir_stream.is_err() {
            return content;
        }
        let mut dir_stream = dir_stream.unwrap();

        while let Some(entry) = dir_stream.next().await {
            let asset_type = if reader.is_directory(&entry).await.unwrap() {
                AssetType::Directory
            } else {
                AssetType::Unknown
            };
            content.0.push(AssetEntry {
                name: entry
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy()
                    .to_string(),
                asset_type,
            });
        }
        content
    });

    commands
        .spawn_empty()
        .insert(FetchDirectoryContentTask(task));
}

#[derive(Component, Default)]
pub(crate) struct ScrollingList {
    position: f32,
}

// TODO: Replace with an editor widget
/// Handle the scrolling of the content list
pub(crate) fn scrolling(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &ComputedNode, &Parent, &mut Node)>,
    query_node: Query<&ComputedNode>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, list_computed_node, parent, mut list_node) in &mut query_list {
            let items_height = list_computed_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;
            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.0,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);

            list_node.top = Val::Px(scrolling_list.position);
        }
    }
}

/// Check if the [`DirectoryContent`] has changed, which relate to the content of the current [`AssetBrowserLocation`]
pub(crate) fn run_if_content_as_changed(directory_content: Res<DirectoryContent>) -> bool {
    directory_content.is_changed()
}

/// Refresh the UI with the content of the current [`AssetBrowserLocation`]
pub(crate) fn refresh_ui(
    mut commands: Commands,
    content_list_query: Query<(Entity, Option<&Children>), With<ScrollingList>>,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
    directory_content: Res<DirectoryContent>,
    location: Res<AssetBrowserLocation>,
    mut asset_sources_builder: ResMut<AssetSourceBuilders>,
) {
    for (content_list_entity, content_list_children) in content_list_query.iter() {
        // Clear content list
        if let Some(content_list_children) = content_list_children {
            for child in content_list_children {
                commands.entity(*child).despawn_recursive();
            }
            commands
                .entity(content_list_entity)
                .remove_children(content_list_children);
        }
        // Regenerate content list
        let mut content_list_ec = commands.entity(content_list_entity);
        spawn_content_list_ui(
            &mut content_list_ec,
            &theme,
            &asset_server,
            &directory_content,
            &location,
            &mut asset_sources_builder,
        );
    }
}

pub fn spawn_content_list_ui(
    parent: &mut EntityCommands,
    theme: &Res<Theme>,
    asset_server: &Res<AssetServer>,
    directory_content: &Res<DirectoryContent>,
    location: &Res<AssetBrowserLocation>,
    asset_sources_builder: &mut AssetSourceBuilders,
) {
    parent.with_children(|parent| {
        if location.source_id.is_none() {
            let sources = asset_sources_builder.build_sources(false, false);
            sources.iter().for_each(|source| {
                spawn_asset_button(
                    parent,
                    AssetType::EngineSource,
                    crate::asset_source_id_to_string(&source.id()),
                    theme,
                    asset_server,
                );
            });
        } else {
            for entry in &directory_content.0 {
                spawn_asset_button(
                    parent,
                    entry.asset_type,
                    entry.name.clone(),
                    theme,
                    asset_server,
                );
            }
        }
    });
}

/// Spawn a new asset button UI element
fn spawn_asset_button(
    parent: &mut ChildBuilder,
    asset_type: AssetType,
    name: String,
    theme: &Res<Theme>,
    asset_server: &Res<AssetServer>,
) {
    let mut entity_commands = parent.spawn((
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
        theme.border_radius,
        crate::ButtonType::AssetButton(asset_type),
    ));
    entity_commands.with_children(|parent| {
        parent.spawn((
            UiImage::new(crate::content_button_to_icon(&asset_type, asset_server)),
            Node {
                height: Val::Px(50.0),
                ..default()
            },
        ));
        parent.spawn((
            Text::new(name),
            TextFont {
                font: theme.font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(theme.text_color),
        ));
    });

    match asset_type {
        AssetType::Directory | AssetType::EngineSource => {
            entity_commands
                .observe(
                    move |_trigger: Trigger<Pointer<Move>>,
                          window_query: Query<Entity, With<Window>>,
                          mut commands: Commands| {
                        let window = window_query.single();
                        commands
                            .entity(window)
                            .insert(CursorIcon::System(SystemCursorIcon::Pointer));
                    },
                )
                .observe(
                    move |_trigger: Trigger<Pointer<Out>>,
                          window_query: Query<Entity, With<Window>>,
                          mut commands: Commands| {
                        let window = window_query.single();
                        commands
                            .entity(window)
                            .insert(CursorIcon::System(SystemCursorIcon::Default));
                    },
                );
        }
        _ => {}
    }
}

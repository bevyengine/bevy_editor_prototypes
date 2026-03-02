use crate::{AssetBrowserLocation, DirectoryContent, DirectoryContentOrder, Entry};
use bevy::{
    asset::io::AssetSourceBuilders,
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures_lite::StreamExt, poll_once},
};

#[derive(Component)]
/// The task that fetches the content of current [`AssetBrowserLocation`]
pub(crate) struct FetchDirectoryContentTask(Task<DirectoryContent>);

pub(crate) fn fetch_task_is_running(
    task_query: Query<(Entity, &FetchDirectoryContentTask)>,
) -> bool {
    task_query.iter().next().is_some()
}

/// Poll the [`FetchDirectoryContentTask`] to check if it's done
/// If it's done, despawn the task entity and insert the result into [`DirectoryContent`]
pub(crate) fn poll_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut FetchDirectoryContentTask)>,
    content_order: Res<DirectoryContentOrder>,
) {
    let (task_entity, mut task) = task_query.single_mut().unwrap();
    if let Some(mut content) = block_on(poll_once(&mut task.0)) {
        content_order.sort(&mut content);

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
                .map(|source| Entry::Source(source.id()))
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
            let entry_name = entry
                .components()
                .next_back()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .to_string();
            content
                .0
                .push(if reader.is_directory(&entry).await.unwrap() {
                    Entry::Folder(entry_name)
                } else {
                    Entry::File(entry_name)
                });
        }
        content
    });

    commands
        .spawn_empty()
        .insert(FetchDirectoryContentTask(task));
}

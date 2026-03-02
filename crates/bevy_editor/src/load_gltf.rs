//! This is a temporary solution for loading GLTF scenes into the editor.

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use bevy_editor_core::prelude::*;
use rfd::{AsyncFileDialog, FileHandle};

pub(crate) struct LoadGltfPlugin;

impl Plugin for LoadGltfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GltfFilepickerTask>()
            .add_systems(Update, (poll_pick_gltf, file_dropped))
            .register_action("load-gltf", "Load GLTF", pick_gltf_action)
            .register_keybinding(Keybinding::new("load-gltf", KeyCode::KeyL).ctrl());
    }
}

fn file_dropped(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_reader: MessageReader<FileDragAndDrop>,
) {
    for event in event_reader.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event
            && let Some(extension) = path_buf.extension()
            && (extension == "gtlf" || extension == "glb")
        {
            let asset_path = GltfAssetLabel::Scene(0).from_asset(path_buf.clone());
            commands.spawn(SceneRoot(asset_server.load_override(asset_path)));
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct GltfFilepickerTask(Option<Task<Option<FileHandle>>>);

pub(crate) fn pick_gltf_action(mut file_picker_task: ResMut<GltfFilepickerTask>) {
    if file_picker_task.0.is_some() {
        return;
    }
    file_picker_task.0 = Some(
        AsyncComputeTaskPool::get().spawn(
            AsyncFileDialog::new()
                .set_title("Load GLTF file")
                .add_filter("gltf/glb", &["gltf", "glb"])
                .pick_file(),
        ),
    );
}

fn poll_pick_gltf(
    mut file_picker_task: ResMut<GltfFilepickerTask>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Some(task) = &mut file_picker_task.0
        && let Some(result) = block_on(future::poll_once(task))
    {
        file_picker_task.0 = None;
        if let Some(file) = result {
            let path = file.path().to_owned();
            let asset_path = GltfAssetLabel::Scene(0).from_asset(path);
            commands.spawn(SceneRoot(asset_server.load_override(asset_path)));
        }
    }
}

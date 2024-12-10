//! This is a temporary solution for loading GLTF scenes into the editor.

use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use rfd::{AsyncFileDialog, FileHandle};

pub(crate) struct LoadGltfPlugin;

impl Plugin for LoadGltfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GltfFilepickerTask>()
            .add_systems(Update, (pick_gltf, poll_pick_gltf, file_dropped));
    }
}

fn file_dropped(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_reader: EventReader<FileDragAndDrop>,
) {
    for event in event_reader.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let asset_path = GltfAssetLabel::Scene(0).from_asset(path_buf.clone());
            commands.spawn(SceneRoot(asset_server.load(asset_path)));
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct GltfFilepickerTask(Option<Task<Option<FileHandle>>>);

pub(crate) fn pick_gltf(
    mut file_picker_task: ResMut<GltfFilepickerTask>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if file_picker_task.0.is_some() {
        return;
    }

    if keyboard_input.pressed(KeyCode::ControlLeft) && keyboard_input.just_pressed(KeyCode::KeyL) {
        file_picker_task.0 = Some(
            AsyncComputeTaskPool::get().spawn(
                AsyncFileDialog::new()
                    .set_title("Load GLTF file")
                    .add_filter("gltf/glb", &["gltf", "glb"])
                    .pick_file(),
            ),
        );
    }
}

fn poll_pick_gltf(
    mut file_picker_task: ResMut<GltfFilepickerTask>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(task) = &mut file_picker_task.0 else {
        return;
    };

    let Some(result) = block_on(future::poll_once(task)) else {
        return;
    };
    file_picker_task.0 = None;

    if let Some(file) = result {
        let path = file.path().to_owned();
        let asset_path = GltfAssetLabel::Scene(0).from_asset(path);
        commands.spawn(SceneRoot(asset_server.load(asset_path)));
    }
}

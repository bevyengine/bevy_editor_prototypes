use std::{
    env,
    io::{BufWriter, Cursor},
    path::{Path, PathBuf},
};

use bevy::{
    asset::{
        io::{file::FileAssetWriter, AssetWriter},
        AssetServer, Assets,
    },
    prelude::{Image, Res, ResMut},
    render::{renderer::RenderDevice, texture::TextureFormatPixelInfo},
    tasks::IoTaskPool,
};
use image::ImageEncoder;

use crate::render::{
    receive::{MainWorldPreviewImageReceiver, PreviewImageCopies},
    RenderedScenePreviews,
};

pub(crate) fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(manifest_dir)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
            .unwrap()
    }
}

pub fn receive_preview(
    mut previews: ResMut<RenderedScenePreviews>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    receiver: Res<MainWorldPreviewImageReceiver>,
    mut image_copies: ResMut<PreviewImageCopies>,
) {
    let thread_pool = IoTaskPool::get();

    while let Ok((id, data)) = receiver.try_recv() {
        let image = images.get_mut(id).unwrap();
        let row_bytes = image.width() as usize * image.texture_descriptor.format.pixel_size();
        let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);
        if row_bytes == aligned_row_bytes {
            image.data = data;
        } else {
            image.data = data
                .chunks(aligned_row_bytes)
                .take(image.height() as usize)
                .flat_map(|row| &row[..row_bytes.min(row.len())])
                .cloned()
                .collect()
        }

        image_copies.remove(&id);

        let Some(scene_handle) = previews.changed.remove(&id) else {
            continue;
        };
        let Some(path) = asset_server.get_path(scene_handle) else {
            continue;
        };

        let image_buffer = image.clone().try_into_dynamic().unwrap().into_rgba8();
        let image_path =
            Path::new("assets/cache/asset_preview").join(path.path().with_extension("png"));

        thread_pool
            .spawn(async move {
                let image_path_full = get_base_path().join(&image_path);
                FileAssetWriter::new("", true)
                    .write(&image_path)
                    .await
                    .unwrap();
                image_buffer.save(image_path_full).unwrap();

                // TODO use the following code once know why it fails sometimes.
                // let mut writer = BufWriter::new(Cursor::new(Vec::new()));
                // image_buffer
                //     .write_to(&mut writer, image::ImageFormat::Png)
                //     .unwrap();
                // FileAssetWriter::new("", true)
                //     .write_bytes(&image_path, writer.buffer())
                //     .await
                //     .unwrap();
            })
            .detach();
    }
}

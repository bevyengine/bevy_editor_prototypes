use std::{
    io::{BufWriter, Cursor},
    path::Path,
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

use crate::render::{receive::{MainWorldPreviewImageReceiver, PreviewImageCopies}, RenderedScenePreviews};

pub fn save_preview(
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

        let Some(scene_handle)=previews.changed.remove(&id) else {
            continue;
        };
        let Some(path) = asset_server.get_path(scene_handle) else {
            continue;
        };

        let image_buffer = match image.clone().try_into_dynamic() {
            Ok(img) => img.to_rgba8(),
            Err(err) => panic!("Failed to create image buffer {err:?}"),
        };
        let image_path = Path::new("cache")
            .join("asset_preview")
            .join(path.path().with_extension("png"));

        thread_pool
            .spawn(async move {
                let image_path = image_path.clone();
                let mut writer = BufWriter::new(Cursor::new(Vec::new()));
                image_buffer
                    .write_to(&mut writer, image::ImageFormat::Png)
                    .unwrap();
                FileAssetWriter::new("", true)
                    .write_bytes(&image_path, writer.buffer())
                    .await
                    .unwrap();
            })
            .detach();
    }
}

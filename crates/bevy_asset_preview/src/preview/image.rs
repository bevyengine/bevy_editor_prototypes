use core::time::Duration;

use bevy::{
    asset::{AssetPath, RenderAssetUsages, UntypedAssetId},
    prelude::*,
};

use crate::preview::{
    PreviewConfig, PreviewMode, PreviewRequestType, PreviewScene3D,
    cache::PreviewCache,
    task::{PendingPreviewRequest, PreviewFailed, PreviewReady, PreviewTaskManager},
};

/// Generates preview images for the specified resolutions and caches them.
pub fn generate_previews_for_resolutions(
    images: &mut Assets<Image>,
    original_image: &Image,
    original_handle: Handle<Image>,
    path: &AssetPath<'static>,
    asset_id: UntypedAssetId,
    resolutions: &[u32],
    cache: &mut PreviewCache,
    timestamp: Duration,
) {
    for &resolution in resolutions {
        if cache.get_by_path(path, Some(resolution)).is_some() {
            continue;
        }

        let preview_handle = match resize_image_for_preview(original_image, resolution) {
            Some(compressed) => images.add(compressed),
            None => original_handle.clone(),
        };

        cache.insert(path, asset_id, resolution, preview_handle, timestamp);
    }
}

/// Requests a preview for an image asset path.
/// Returns the task ID for tracking.
/// Uses highest configured resolution if resolution is None.
pub fn request_image_preview<'a>(
    mut commands: Commands,
    mut task_manager: ResMut<PreviewTaskManager>,
    mut cache: ResMut<PreviewCache>,
    config: Res<PreviewConfig>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut preview_ready_events: EventWriter<PreviewReady>,
    time: Res<Time<Real>>,
    path: impl Into<AssetPath<'a>>,
    resolution: Option<u32>,
) -> u64 {
    let path: AssetPath<'static> = path.into().into_owned();

    if let Some(cache_entry) = cache.get_by_path(&path, resolution) {
        let task_id = task_manager.create_task_id();
        preview_ready_events.write(PreviewReady {
            task_id,
            path: path.clone(),
            image_handle: cache_entry.image_handle.clone(),
        });
        return task_id;
    }

    let task_id = task_manager.create_task_id();
    let handle: Handle<Image> = asset_server.load(&path);

    if let Some(image) = images.get(&handle) {
        let image_clone = image.clone();
        let asset_id = handle.id().untyped();

        generate_previews_for_resolutions(
            &mut images,
            &image_clone,
            handle.clone(),
            &path,
            asset_id,
            &config.resolutions,
            &mut cache,
            time.elapsed(),
        );

        let preview_handle = cache
            .get_by_path(&path, resolution)
            .map(|entry| entry.image_handle.clone())
            .unwrap_or_else(|| handle.clone());

        preview_ready_events.write(PreviewReady {
            task_id,
            path: path.clone(),
            image_handle: preview_handle,
        });
        return task_id;
    }

    let entity = commands
        .spawn(PendingPreviewRequest {
            task_id,
            path: path.clone(),
            request_type: PreviewRequestType::Image2D,
            mode: PreviewMode::Image,
        })
        .id();
    task_manager.register_task(task_id, entity);
    task_id
}

/// System that handles image asset events for previews.
pub fn handle_image_preview_events(
    mut commands: Commands,
    mut cache: ResMut<PreviewCache>,
    config: Res<PreviewConfig>,
    asset_server: Res<AssetServer>,
    mut preview_ready_events: EventWriter<PreviewReady>,
    mut preview_failed_events: EventWriter<PreviewFailed>,
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    requests: Query<(Entity, &PendingPreviewRequest), Without<PreviewScene3D>>,
    mut task_manager: ResMut<PreviewTaskManager>,
    time: Res<Time<Real>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                for (entity, request) in requests.iter() {
                    let handle: Handle<Image> = asset_server.load(&request.path);
                    if handle.id() == *id {
                        if let Some(image) = images.get(&handle) {
                            let image_clone = image.clone();
                            let asset_id = handle.id().untyped();

                            generate_previews_for_resolutions(
                                &mut images,
                                &image_clone,
                                handle.clone(),
                                &request.path,
                                asset_id,
                                &config.resolutions,
                                &mut cache,
                                time.elapsed(),
                            );

                            let preview_image = cache
                                .get_by_path(&request.path, None)
                                .map(|entry| entry.image_handle.clone())
                                .unwrap_or_else(|| handle.clone());

                            preview_ready_events.write(PreviewReady {
                                task_id: request.task_id,
                                path: request.path.clone(),
                                image_handle: preview_image,
                            });

                            task_manager.remove_task(request.task_id);
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
            AssetEvent::Removed { id } => {
                cache.remove_by_id(id.untyped(), None);

                for (entity, request) in requests.iter() {
                    let handle: Handle<Image> = asset_server.load(&request.path);
                    if handle.id() == *id {
                        preview_failed_events.write(PreviewFailed {
                            task_id: request.task_id,
                            path: request.path.clone(),
                            error: "Image asset was removed".to_string(),
                        });
                        task_manager.remove_task(request.task_id);
                        commands.entity(entity).despawn();
                    }
                }
            }
            _ => {}
        }
    }
}

/// Resizes an image to a specific preview size.
/// Returns None if the image is already small enough.
pub fn resize_image_for_preview(image: &Image, target_size: u32) -> Option<Image> {
    let width = image.width();
    let height = image.height();

    // If image is already small enough, return None (use original)
    if width <= target_size && height <= target_size {
        return None;
    }

    // Calculate new size maintaining aspect ratio
    let (new_width, new_height) = if width > height {
        (
            target_size,
            (height as f32 * target_size as f32 / width as f32) as u32,
        )
    } else {
        (
            (width as f32 * target_size as f32 / height as f32) as u32,
            target_size,
        )
    };

    // Convert to dynamic image for resizing
    let dynamic_image = match image.clone().try_into_dynamic() {
        Ok(img) => img,
        Err(_) => return None,
    };

    // Resize using high-quality filter
    let resized =
        dynamic_image.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Convert back to Image
    Some(Image::from_dynamic(
        resized,
        true, // is_srgb
        RenderAssetUsages::RENDER_WORLD,
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use bevy::{
        asset::AssetPlugin,
        prelude::*,
        render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        },
    };
    use tempfile::TempDir;

    use super::*;

    fn create_test_image(
        images: &mut Assets<Image>,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Handle<Image> {
        let pixel_data: Vec<u8> = (0..(width * height))
            .flat_map(|_| color.iter().copied())
            .collect();

        let image = Image::new_fill(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &pixel_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        images.add(image)
    }

    #[test]
    fn test_image_compression() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let assets_dir = temp_dir.path().join("assets");
        fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: assets_dir.display().to_string(),
                ..Default::default()
            })
            .init_asset::<Image>();

        let mut images = app.world_mut().resource_mut::<Assets<Image>>();

        // Test small image (should not be compressed)
        let small_handle = create_test_image(&mut images, 64, 64, [128, 128, 128, 255]);
        let small_image = images.get(&small_handle).unwrap();
        let compressed_small = resize_image_for_preview(small_image, 256);
        assert!(
            compressed_small.is_none(),
            "Small image should not be compressed"
        );

        // Test large image (should be compressed)
        let large_handle = create_test_image(&mut images, 512, 512, [128, 128, 128, 255]);
        let large_image = images.get(&large_handle).unwrap();
        let compressed_large = resize_image_for_preview(large_image, 256);
        assert!(
            compressed_large.is_some(),
            "Large image should be compressed"
        );
        let compressed = compressed_large.unwrap();
        assert!(
            compressed.width() <= 256 && compressed.height() <= 256,
            "Compressed image should be <= 256x256, got {}x{}",
            compressed.width(),
            compressed.height()
        );

        // Test wide image (maintain aspect ratio)
        let wide_handle = create_test_image(&mut images, 800, 200, [128, 128, 128, 255]);
        let wide_image = images.get(&wide_handle).unwrap();
        let compressed_wide = resize_image_for_preview(wide_image, 256);
        assert!(compressed_wide.is_some(), "Wide image should be compressed");
        let compressed = compressed_wide.unwrap();
        assert_eq!(compressed.width(), 256, "Wide image width should be 256");
        assert!(
            compressed.height() < 256,
            "Wide image height should be < 256"
        );
        // Verify aspect ratio: 800/200 = 4:1, after compression should be 256:64
        let expected_height = (200.0 * 256.0 / 800.0) as u32;
        assert_eq!(
            compressed.height(),
            expected_height,
            "Wide image should maintain aspect ratio"
        );

        // Test tall image (maintain aspect ratio)
        let tall_handle = create_test_image(&mut images, 200, 800, [128, 128, 128, 255]);
        let tall_image = images.get(&tall_handle).unwrap();
        let compressed_tall = resize_image_for_preview(tall_image, 256);
        assert!(compressed_tall.is_some(), "Tall image should be compressed");
        let compressed = compressed_tall.unwrap();
        assert_eq!(compressed.height(), 256, "Tall image height should be 256");
        assert!(compressed.width() < 256, "Tall image width should be < 256");
        // Verify aspect ratio: 200/800 = 1:4, after compression should be 64:256
        let expected_width = (200.0 * 256.0 / 800.0) as u32;
        assert_eq!(
            compressed.width(),
            expected_width,
            "Tall image should maintain aspect ratio"
        );
    }
}

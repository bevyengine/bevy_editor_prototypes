use std::path::{Path, PathBuf};

use bevy::{
    asset::{AssetPath, AssetServer, LoadState},
    gltf::GltfAssetLabel,
    image::Image,
    prelude::*,
    scene::Scene,
};

use crate::{
    asset::{AssetLoader, LoadPriority},
    preview::{
        ModelFormat, PendingPreviewRequest, PreviewCache, PreviewConfig, PreviewMode,
        PreviewRequestType, PreviewTaskManager,
    },
};

#[derive(Component, Deref)]
pub struct PreviewAsset(pub PathBuf);

/// Component to track pending preview load requests
#[derive(Component, Debug)]
pub struct PendingPreviewLoad {
    pub task_id: u64,
    pub asset_path: AssetPath<'static>,
}

const FILE_PLACEHOLDER: &'static str = "embedded://bevy_asset_browser/assets/file_icon.png";

/// Checks if a file path represents an image file based on its extension.
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "png"
                    | "jpg"
                    | "jpeg"
                    | "bmp"
                    | "gif"
                    | "ico"
                    | "pnm"
                    | "pam"
                    | "pbm"
                    | "pgm"
                    | "ppm"
                    | "tga"
                    | "webp"
                    | "tiff"
                    | "tif"
                    | "dds"
                    | "exr"
                    | "hdr"
                    | "ktx2"
                    | "basis"
                    | "qoi"
            )
        })
        .unwrap_or(false)
}

/// Checks if a file path represents a GLTF model file based on its extension.
pub fn is_gltf_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "gltf" | "glb"))
        .unwrap_or(false)
}

/// Checks if a file path represents an OBJ model file based on its extension.
pub fn is_obj_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase().as_str() == "obj")
        .unwrap_or(false)
}

/// Checks if a file path represents a 3D model file based on its extension.
pub fn is_model_file(path: &Path) -> bool {
    is_gltf_file(path) || is_obj_file(path)
    // Add other model formats here as needed (fbx, 3ds, etc.)
}

/// System that handles PreviewAsset components and initiates preview loading.
pub fn preview_handler(
    mut commands: Commands,
    mut requests_query: Query<
        (Entity, &PreviewAsset),
        (
            Added<PreviewAsset>,
            Without<PendingPreviewLoad>,
            Without<ImageNode>,
        ),
    >,
    asset_server: Res<AssetServer>,
    mut loader: ResMut<AssetLoader>,
    mut task_manager: ResMut<PreviewTaskManager>,
    cache: Res<PreviewCache>,
) {
    for (entity, preview_asset) in &mut requests_query {
        let path = &preview_asset.0;

        // Convert PathBuf to AssetPath
        let asset_path: AssetPath<'static> = path.clone().into();

        // Check if it's an image file
        if is_image_file(path) {
            // Check cache first
            if let Some(cache_entry) = cache.get_by_path(&asset_path, None) {
                // Cache hit - use cached preview immediately
                commands
                    .entity(entity)
                    .insert(ImageNode::new(cache_entry.image_handle.clone()));
                commands.entity(entity).remove::<PreviewAsset>();
                continue;
            }

            // Cache miss - submit to AssetLoader with Preload priority
            let task_id = loader.submit(&asset_path, LoadPriority::Preload);

            // Mark as pending
            commands.entity(entity).insert(PendingPreviewLoad {
                task_id,
                asset_path: asset_path.clone(),
            });

            // Insert placeholder temporarily
            let placeholder = asset_server.load(FILE_PLACEHOLDER);
            commands.entity(entity).insert(ImageNode::new(placeholder));
            continue;
        }

        // Check if it's a 3D model file
        if is_model_file(path) {
            // Check cache first
            if let Some(cache_entry) = cache.get_by_path(&asset_path, Some(256)) {
                // Cache hit - use cached preview immediately
                commands
                    .entity(entity)
                    .insert(ImageNode::new(cache_entry.image_handle.clone()));
                commands.entity(entity).remove::<PreviewAsset>();
                continue;
            }

            // Request 3D preview
            let _task_id = request_3d_preview(
                &mut commands,
                &mut task_manager,
                &asset_server,
                &asset_path,
                PreviewMode::Image, // Default to image mode for folder views
            );

            // Insert placeholder temporarily
            let placeholder = asset_server.load(FILE_PLACEHOLDER);
            commands.entity(entity).insert(ImageNode::new(placeholder));
            continue;
        }

        // Not a supported file type, use placeholder
        let placeholder = asset_server.load(FILE_PLACEHOLDER);
        commands.entity(entity).insert(ImageNode::new(placeholder));
        commands.entity(entity).remove::<PreviewAsset>();
    }
}

/// Requests a 3D preview for the given asset path.
pub fn request_3d_preview(
    commands: &mut Commands,
    task_manager: &mut PreviewTaskManager,
    asset_server: &AssetServer,
    path: &AssetPath<'static>,
    mode: PreviewMode,
) -> u64 {
    let task_id = task_manager.create_task_id();

    // Determine request type based on file extension
    let request_type = if is_gltf_file(path.path()) {
        // Load GLTF Scene directly using GltfAssetLabel
        let scene_path = GltfAssetLabel::Scene(0).from_asset(path.clone());
        let scene_handle: Handle<Scene> = asset_server.load(scene_path);
        PreviewRequestType::ModelFile {
            handle: scene_handle,
            format: ModelFormat::Gltf,
        }
    } else if is_obj_file(path.path()) {
        let scene_handle: Handle<Scene> = asset_server.load(path);
        PreviewRequestType::ModelFile {
            handle: scene_handle,
            format: ModelFormat::Obj,
        }
    } else {
        // Try as generic scene
        let scene_handle: Handle<Scene> = asset_server.load(path);
        PreviewRequestType::ModelFile {
            handle: scene_handle,
            format: ModelFormat::Other(
                path.path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ),
        }
    };

    // Spawn pending request entity
    let entity = commands
        .spawn(PendingPreviewRequest {
            task_id,
            path: path.clone(),
            request_type,
            mode,
        })
        .id();

    task_manager.register_task(task_id, entity);
    task_id
}

/// System that handles completed asset loads and updates previews.
pub fn handle_preview_load_completed(
    mut commands: Commands,
    mut cache: ResMut<PreviewCache>,
    config: Res<PreviewConfig>,
    mut images: ResMut<Assets<Image>>,
    mut load_completed_events: EventReader<crate::asset::AssetLoadCompleted>,
    pending_query: Query<(Entity, &PendingPreviewLoad), With<ImageNode>>,
    mut image_node_query: Query<&mut ImageNode>,
    time: Res<Time<Real>>,
) {
    for event in load_completed_events.read() {
        // Find entities waiting for this task
        for (entity, pending) in pending_query.iter() {
            if pending.task_id == event.task_id {
                // Check if image is loaded
                if let Some(image) = images.get(&event.handle) {
                    // Clone image data before mutable operations
                    let image_clone = image.clone();
                    let asset_id = event.handle.id().untyped();

                    // Generate previews for all configured resolutions
                    crate::preview::generate_previews_for_resolutions(
                        &mut images,
                        &image_clone,
                        event.handle.clone(),
                        &pending.asset_path,
                        asset_id,
                        &config.resolutions,
                        &mut cache,
                        time.elapsed(),
                    );

                    // Get the highest resolution preview (or original if none generated)
                    let preview_image = cache
                        .get_by_path(&pending.asset_path, None)
                        .map(|entry| entry.image_handle.clone())
                        .unwrap_or_else(|| event.handle.clone());

                    // Update ImageNode
                    if let Ok(mut image_node) = image_node_query.get_mut(entity) {
                        image_node.image = preview_image;
                    }

                    // Cleanup
                    commands.entity(entity).remove::<PreviewAsset>();
                    commands.entity(entity).remove::<PendingPreviewLoad>();
                }
                break;
            }
        }
    }
}

/// System that handles failed asset loads and cleans up pending requests.
pub fn handle_preview_load_failed(
    mut commands: Commands,
    mut load_failed_events: EventReader<crate::asset::AssetLoadFailed>,
    pending_query: Query<(Entity, &PendingPreviewLoad), With<PreviewAsset>>,
) {
    for event in load_failed_events.read() {
        // Find entities waiting for this task
        for (entity, pending) in pending_query.iter() {
            if pending.task_id == event.task_id {
                // Cleanup - remove PendingPreviewLoad, keep placeholder image
                commands.entity(entity).remove::<PendingPreviewLoad>();
                commands.entity(entity).remove::<PreviewAsset>();
                break;
            }
        }
    }
}

/// System that periodically checks for failed asset loads and cleans them up.
/// This is a fallback for cases where AssetEvent::Removed is not emitted.
/// It checks both active tasks and directly loads the asset to check its state.
pub fn check_failed_loads(
    mut commands: Commands,
    mut loader: ResMut<crate::asset::AssetLoader>,
    asset_server: Res<AssetServer>,
    pending_query: Query<(Entity, &PendingPreviewLoad), With<PreviewAsset>>,
    task_query: Query<(Entity, &crate::asset::ActiveLoadTask)>,
) {
    for (entity, pending) in pending_query.iter() {
        // Try to find the active task for this pending load to get the correct handle
        let mut active_task_entity_opt = None;
        let mut handle_opt = None;
        for (task_entity, active_task) in task_query.iter() {
            if active_task.task_id == pending.task_id {
                handle_opt = Some(active_task.handle.clone());
                active_task_entity_opt = Some((task_entity, active_task));
                break;
            }
        }

        // Get handle - use active task's handle if available, otherwise load directly
        // Note: asset_server.load() is idempotent - calling it multiple times returns the same handle
        let handle: Handle<Image> =
            handle_opt.unwrap_or_else(|| asset_server.load(&pending.asset_path));
        let load_state = asset_server.load_state(&handle);

        // Check if the asset load has failed
        if let LoadState::Failed(_) = load_state {
            // Load failed - cleanup PendingPreviewLoad
            commands.entity(entity).remove::<PendingPreviewLoad>();
            commands.entity(entity).remove::<PreviewAsset>();

            // Also cleanup ActiveLoadTask if it exists
            if let Some((task_entity, active_task)) = active_task_entity_opt {
                loader.finish_task();
                loader.cleanup_task(active_task.task_id, active_task.handle.id());
                commands.entity(task_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(PathBuf::from("test.png").as_path()));
        assert!(is_image_file(PathBuf::from("test.jpg").as_path()));
        assert!(is_image_file(PathBuf::from("test.webp").as_path()));
        assert!(is_image_file(PathBuf::from("test.TGA").as_path())); // Case insensitive

        assert!(!is_image_file(PathBuf::from("test.txt").as_path()));
        assert!(!is_image_file(PathBuf::from("test.rs").as_path()));
        assert!(!is_image_file(PathBuf::from("test").as_path())); // No extension
    }
}

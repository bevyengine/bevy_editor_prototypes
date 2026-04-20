use bevy::{asset::AssetPath, prelude::*, render::view::screenshot::Screenshot};

use crate::preview::{
    PreviewConfig, PreviewMode, PreviewRequestType,
    cache::PreviewCache,
    renderer::*,
    task::{PendingPreviewRequest, PreviewReady, PreviewTaskManager},
};

/// Component marking a 3D preview scene that's being set up.
#[derive(Component, Debug)]
pub struct PreviewScene3D {
    /// Task ID.
    pub task_id: u64,
    /// Asset path.
    pub path: AssetPath<'static>,
    /// Camera entity.
    pub camera_entity: Entity,
    /// Render target image handle.
    pub render_target: Handle<Image>,
    /// Preview entity (GLTF scene, mesh, etc.).
    pub preview_entity: Option<Entity>,
}

/// Component marking that we're waiting for a screenshot.
#[derive(Component, Debug)]
pub struct WaitingForScreenshot {
    /// Task ID.
    pub task_id: u64,
    /// Asset path.
    pub path: AssetPath<'static>,
}

/// Processes 3D preview requests and sets up preview scenes.
pub fn process_3d_preview_requests(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut task_manager: ResMut<PreviewTaskManager>,
    cache: ResMut<PreviewCache>,
    config: Res<PreviewConfig>,
    _asset_server: Res<AssetServer>,
    scene_assets: Res<Assets<Scene>>,
    requests: Query<
        (Entity, &PendingPreviewRequest),
        (
            Added<PendingPreviewRequest>,
            Without<PreviewScene3D>,
            Without<WaitingForScreenshot>,
        ),
    >,
    mut preview_ready_events: EventWriter<PreviewReady>,
) {
    for (entity, request) in requests.iter() {
        // Only process 3D requests
        match &request.request_type {
            PreviewRequestType::Image2D => continue,
            _ => {}
        }

        // Check cache first (for image mode)
        let preview_resolution = config.resolutions.iter().max().copied().unwrap_or(256);
        if request.mode == PreviewMode::Image {
            if let Some(cache_entry) = cache.get_by_path(&request.path, Some(preview_resolution)) {
                preview_ready_events.write(PreviewReady {
                    task_id: request.task_id,
                    path: request.path.clone(),
                    image_handle: cache_entry.image_handle.clone(),
                });
                task_manager.remove_task(request.task_id);
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Create preview scene using the highest resolution from config
        let (camera_entity, render_target, _) =
            create_preview_scene(&mut commands, &mut images, &config, preview_resolution);

        // Spawn preview entity based on request type
        let preview_entity = match &request.request_type {
            PreviewRequestType::ModelFile { handle, .. } => {
                // Direct Scene loading (GLTF, OBJ, FBX, etc.)
                if scene_assets.get(handle).is_some() {
                    Some(spawn_model_scene_preview(
                        &mut commands,
                        handle.clone(),
                        config.render_layer,
                    ))
                } else {
                    // Asset not loaded yet, wait for it
                    commands.entity(entity).insert(PreviewScene3D {
                        task_id: request.task_id,
                        path: request.path.clone(),
                        camera_entity,
                        render_target: render_target.clone(),
                        preview_entity: None,
                    });
                    continue;
                }
            }
            PreviewRequestType::Material { handle } => {
                if materials.get(handle).is_some() {
                    Some(spawn_material_preview(
                        &mut commands,
                        &mut meshes,
                        handle.clone(),
                        config.render_layer,
                    ))
                } else {
                    commands.entity(entity).insert(PreviewScene3D {
                        task_id: request.task_id,
                        path: request.path.clone(),
                        camera_entity,
                        render_target: render_target.clone(),
                        preview_entity: None,
                    });
                    continue;
                }
            }
            PreviewRequestType::Mesh { handle } => {
                if meshes.get(handle).is_some() {
                    Some(spawn_mesh_preview(
                        &mut commands,
                        &mut materials,
                        handle.clone(),
                        config.render_layer,
                    ))
                } else {
                    commands.entity(entity).insert(PreviewScene3D {
                        task_id: request.task_id,
                        path: request.path.clone(),
                        camera_entity,
                        render_target: render_target.clone(),
                        preview_entity: None,
                    });
                    continue;
                }
            }
            PreviewRequestType::Image2D => unreachable!(),
        };

        // Update preview scene component
        commands.entity(entity).insert(PreviewScene3D {
            task_id: request.task_id,
            path: request.path.clone(),
            camera_entity,
            render_target: render_target.clone(),
            preview_entity,
        });

        // For image mode, wait a frame then capture screenshot
        if request.mode == PreviewMode::Image {
            commands.entity(entity).insert(WaitingForScreenshot {
                task_id: request.task_id,
                path: request.path.clone(),
            });
        }
    }
}

/// Waits for assets to load and updates preview scenes.
pub fn wait_for_asset_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<PreviewConfig>,
    scene_assets: Res<Assets<Scene>>,
    mut preview_scenes: Query<(Entity, &mut PreviewScene3D, &PendingPreviewRequest)>,
) {
    for (entity, mut scene, request) in preview_scenes.iter_mut() {
        if scene.preview_entity.is_some() {
            continue; // Already spawned
        }

        let preview_entity = match &request.request_type {
            PreviewRequestType::ModelFile { handle, .. } => {
                // Direct Scene loading (GLTF, OBJ, FBX, etc.)
                if scene_assets.get(handle).is_some() {
                    Some(spawn_model_scene_preview(
                        &mut commands,
                        handle.clone(),
                        config.render_layer,
                    ))
                } else {
                    continue;
                }
            }
            PreviewRequestType::Material { handle } => {
                if materials.get(handle).is_some() {
                    Some(spawn_material_preview(
                        &mut commands,
                        &mut meshes,
                        handle.clone(),
                        config.render_layer,
                    ))
                } else {
                    continue;
                }
            }
            PreviewRequestType::Mesh { handle } => {
                if meshes.get(handle).is_some() {
                    Some(spawn_mesh_preview(
                        &mut commands,
                        &mut materials,
                        handle.clone(),
                        config.render_layer,
                    ))
                } else {
                    continue;
                }
            }
            PreviewRequestType::Image2D => unreachable!(),
        };

        scene.preview_entity = preview_entity;

        // If image mode, mark for screenshot
        if request.mode == PreviewMode::Image {
            commands.entity(entity).insert(WaitingForScreenshot {
                task_id: request.task_id,
                path: request.path.clone(),
            });
        }
    }
}

/// Captures screenshots for image preview mode.
pub fn capture_preview_screenshot(
    mut commands: Commands,
    preview_scenes: Query<
        (Entity, &PreviewScene3D, &WaitingForScreenshot),
        Added<WaitingForScreenshot>,
    >,
) {
    for (entity, scene, _waiting) in preview_scenes.iter() {
        // Spawn screenshot component on camera
        commands
            .entity(scene.camera_entity)
            .insert(Screenshot::image(scene.render_target.clone()));

        // Remove waiting component (screenshot will be handled by event)
        commands.entity(entity).remove::<WaitingForScreenshot>();
    }
}

/// Handles screenshot capture events and caches previews.
/// Note: ScreenshotCaptured is an EntityEvent, which requires observers.
/// This is a simplified placeholder - proper implementation would use observers.
pub fn handle_preview_screenshots(
    _commands: Commands,
    _images: ResMut<Assets<Image>>,
    _cache: ResMut<PreviewCache>,
    _preview_ready_events: EventWriter<PreviewReady>,
    _task_manager: ResMut<PreviewTaskManager>,
    _preview_scenes: Query<(Entity, &PreviewScene3D)>,
    _time: Res<Time<Real>>,
    // Note: ScreenshotCaptured is an EntityEvent, not a regular event
    // We'll need to handle it via observers in the future
) {
    // Placeholder: In a real implementation, we'd use observers to listen for ScreenshotCaptured
    // For now, this system is registered but won't process events until observers are set up
}

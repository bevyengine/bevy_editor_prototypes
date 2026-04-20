mod common;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    asset::{AssetPath, AssetPlugin},
    image::{CompressedImageFormats, ImageLoader},
    prelude::*,
    render::view::screenshot::Screenshot,
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_asset_preview::{
    PendingPreviewLoad, PendingPreviewRequest, PreviewAsset, PreviewCache, PreviewConfig,
    PreviewReady, PreviewScene3D, WaitingForScreenshot,
};
use tempfile::TempDir;

use common::{
    create_test_image, create_test_model, save_test_image, save_test_model,
    wait_for_load_completion, wait_for_preview_cached,
};

#[test]
fn test_image_preview_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let assets_dir = temp_dir.path().join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    unsafe {
        std::env::set_var(
            "BEVY_ASSET_ROOT",
            temp_dir
                .path()
                .to_str()
                .expect("Failed to convert path to string"),
        );
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: assets_dir.display().to_string(),
            ..Default::default()
        })
        .add_plugins(bevy_asset_preview::AssetPreviewPlugin)
        .init_asset::<Image>()
        .init_asset::<bevy::mesh::Mesh>()
        .init_asset::<bevy::pbr::StandardMaterial>()
        .init_asset::<bevy::scene::Scene>()
        .register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE))
        .add_event::<PreviewReady>();

    app.finish();
    app.update();

    // Create test image file
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let filename = "test_preview.png";
    let handle = create_test_image(&mut images, 512, 512, [255, 0, 0, 255]);
    let image = images.get(&handle).unwrap();
    let image_path = assets_dir.join(filename);
    save_test_image(image, &image_path).expect("Failed to save test image");
    drop(images);

    // Request preview
    let entity = app
        .world_mut()
        .spawn(PreviewAsset(PathBuf::from(filename)))
        .id();

    app.update();

    // Verify initial state
    let world = app.world();
    assert!(
        world.entity(entity).contains::<ImageNode>(),
        "Should have ImageNode placeholder"
    );
    assert!(
        world.entity(entity).contains::<PendingPreviewLoad>(),
        "Should have PendingPreviewLoad component"
    );

    // Wait for load completion
    let completed_events = wait_for_load_completion(&mut app, 1, 3000);
    assert_eq!(completed_events.len(), 1);

    // Wait for preview generation to complete
    let asset_path: AssetPath<'static> = AssetPath::from(filename).into_owned();
    let config = app.world().resource::<PreviewConfig>();
    let preview_resolution = config.resolutions.iter().max().copied().unwrap_or(256);
    assert!(
        wait_for_preview_cached(&mut app, &asset_path, Some(preview_resolution), 1000),
        "Preview should be cached within max iterations"
    );

    // Verify preview was generated and cached
    let world = app.world();
    let cache = world.resource::<PreviewCache>();
    let config = world.resource::<PreviewConfig>();

    // Check that all configured resolutions are cached
    for &resolution in &config.resolutions {
        let cache_entry = cache.get_by_path(&asset_path, Some(resolution));
        assert!(
            cache_entry.is_some(),
            "Should have cached preview at {}px resolution",
            resolution
        );
    }

    // Verify ImageNode was updated with preview
    let image_node = world.entity(entity).get::<ImageNode>();
    assert!(
        image_node.is_some(),
        "Entity should have ImageNode after preview generation"
    );
    let image_node = image_node.unwrap();
    let images = world.resource::<Assets<Image>>();
    assert!(
        images.get(&image_node.image).is_some(),
        "ImageNode should reference a valid image asset"
    );

    // Verify cleanup
    assert!(
        !world.entity(entity).contains::<PreviewAsset>(),
        "PreviewAsset should be removed after preview generation"
    );
    assert!(
        !world.entity(entity).contains::<PendingPreviewLoad>(),
        "PendingPreviewLoad should be removed after preview generation"
    );

    // Verify entity still exists and is valid
    assert!(
        world.entity(entity).contains::<ImageNode>(),
        "Entity should still exist with ImageNode after cleanup"
    );
}

#[test]
fn test_image_preview_cache_hit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let assets_dir = temp_dir.path().join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    unsafe {
        std::env::set_var(
            "BEVY_ASSET_ROOT",
            temp_dir
                .path()
                .to_str()
                .expect("Failed to convert path to string"),
        );
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: assets_dir.display().to_string(),
            ..Default::default()
        })
        .add_plugins(bevy_asset_preview::AssetPreviewPlugin)
        .init_asset::<Image>()
        .init_asset::<bevy::mesh::Mesh>()
        .init_asset::<bevy::pbr::StandardMaterial>()
        .init_asset::<bevy::scene::Scene>()
        .register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE))
        .add_event::<PreviewReady>();

    app.finish();
    app.update();

    // Create test image file
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let filename = "cached_preview.png";
    let handle = create_test_image(&mut images, 256, 256, [0, 255, 0, 255]);
    let image = images.get(&handle).unwrap();
    let image_path = assets_dir.join(filename);
    save_test_image(image, &image_path).expect("Failed to save test image");
    drop(images);

    // First request - should generate preview
    let entity1 = app
        .world_mut()
        .spawn(PreviewAsset(PathBuf::from(filename)))
        .id();

    app.update();

    // Wait for first preview to complete
    wait_for_load_completion(&mut app, 1, 3000);

    // Wait for preview generation to complete
    let asset_path: AssetPath<'static> = AssetPath::from(filename).into_owned();
    let config = app.world().resource::<PreviewConfig>();
    let preview_resolution = config.resolutions.iter().max().copied().unwrap_or(256);
    assert!(
        wait_for_preview_cached(&mut app, &asset_path, Some(preview_resolution), 1000),
        "Preview should be cached within max iterations"
    );

    // Second request - should hit cache immediately
    let entity2 = app
        .world_mut()
        .spawn(PreviewAsset(PathBuf::from(filename)))
        .id();

    app.update();

    // Verify cache hit - should not have PendingPreviewLoad
    let world = app.world();
    assert!(
        world.entity(entity2).contains::<ImageNode>(),
        "Second request should have ImageNode immediately"
    );
    assert!(
        !world.entity(entity2).contains::<PendingPreviewLoad>(),
        "Second request should not have PendingPreviewLoad (cache hit)"
    );
    assert!(
        !world.entity(entity2).contains::<PreviewAsset>(),
        "PreviewAsset should be removed immediately on cache hit"
    );

    // Verify both entities have ImageNode with valid images
    let image_node1 = world.entity(entity1).get::<ImageNode>();
    let image_node2 = world.entity(entity2).get::<ImageNode>();
    assert!(
        image_node1.is_some() && image_node2.is_some(),
        "Both entities should have ImageNode"
    );

    let images = world.resource::<Assets<Image>>();
    let image_node1 = image_node1.unwrap();
    let image_node2 = image_node2.unwrap();

    // Verify both ImageNodes reference valid images
    assert!(
        images.get(&image_node1.image).is_some(),
        "Entity1 ImageNode should reference a valid image"
    );
    assert!(
        images.get(&image_node2.image).is_some(),
        "Entity2 ImageNode should reference a valid image"
    );

    // Verify both entities reference the same cached image
    assert_eq!(
        image_node1.image.id(),
        image_node2.image.id(),
        "Both entities should reference the same cached preview image"
    );

    // Verify first entity was also cleaned up
    assert!(
        !world.entity(entity1).contains::<PreviewAsset>(),
        "First entity should have PreviewAsset removed"
    );
    assert!(
        !world.entity(entity1).contains::<PendingPreviewLoad>(),
        "First entity should have PendingPreviewLoad removed"
    );
}

#[test]
fn test_model_preview_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let assets_dir = temp_dir.path().join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    unsafe {
        std::env::set_var(
            "BEVY_ASSET_ROOT",
            temp_dir
                .path()
                .to_str()
                .expect("Failed to convert path to string"),
        );
    }

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: assets_dir.display().to_string(),
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: None,
                // Don't automatically exit due to having no windows
                exit_condition: ExitCondition::DontExit,
                ..Default::default()
            })
            // WinitPlugin will panic in environments without a display server
            .disable::<WinitPlugin>(),
    )
    .add_plugins(bevy_asset_preview::AssetPreviewPlugin)
    .register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE))
    .add_event::<PreviewReady>()
    // ScheduleRunnerPlugin provides an alternative to the default bevy_winit app runner
    // which manages the loop without creating a window
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1.0 / 60.0,
    )));

    app.finish();
    // Run a few updates to allow render pipeline to initialize
    for _ in 0..3 {
        app.update();
    }

    // Create a simple glb model file
    let filename = "test_model.glb";
    let model_path = assets_dir.join(filename);
    let (root, buffer_data) = create_test_model();
    save_test_model(&root, &buffer_data, &model_path).expect("Failed to save glTF model");

    // Request preview for model
    let entity = app
        .world_mut()
        .spawn(PreviewAsset(PathBuf::from(filename)))
        .id();

    app.update();

    // Verify initial state - should have ImageNode placeholder
    let world = app.world();
    assert!(
        world.entity(entity).contains::<ImageNode>(),
        "Should have ImageNode placeholder for model"
    );

    // PreviewAsset may or may not be removed immediately for model files
    // depending on cache state, so we don't assert on this

    // Find the entity with PendingPreviewRequest (created by request_3d_preview)
    let mut request_entity = None;
    let mut request_task_id = None;
    let mut request_handle = None;

    {
        let mut preview_request_query = app.world_mut().query::<(Entity, &PendingPreviewRequest)>();
        let world = app.world();
        for (req_entity, request) in preview_request_query.iter(world) {
            if request.path.to_string() == filename {
                request_entity = Some(req_entity);
                request_task_id = Some(request.task_id);

                // Extract handle for later use
                match &request.request_type {
                    bevy_asset_preview::PreviewRequestType::ModelFile { handle, format } => {
                        assert_eq!(
                            format,
                            &bevy_asset_preview::ModelFormat::Gltf,
                            "Model file should be recognized as Gltf format"
                        );
                        request_handle = Some(handle.clone());
                    }
                    _ => panic!(
                        "Expected ModelFile request type, got {:?}",
                        request.request_type
                    ),
                }
                break;
            }
        }
    }

    assert!(
        request_entity.is_some(),
        "Should have created PendingPreviewRequest for model file"
    );
    let request_entity = request_entity.unwrap();
    let request_task_id = request_task_id.unwrap();
    let request_handle = request_handle.unwrap();

    // Wait for PreviewScene3D to be created
    let mut iterations = 0;
    let mut preview_scene_created = false;
    while !preview_scene_created && iterations < 1000 {
        app.update();
        iterations += 1;
        let world = app.world();
        if world.entity(request_entity).contains::<PreviewScene3D>() {
            preview_scene_created = true;
        }
    }
    assert!(
        preview_scene_created,
        "PreviewScene3D should be created within max iterations"
    );

    // Verify PreviewScene3D properties
    let world = app.world();
    let scene_3d = world
        .entity(request_entity)
        .get::<PreviewScene3D>()
        .unwrap();
    assert_eq!(
        scene_3d.path.to_string(),
        filename,
        "PreviewScene3D should have correct path"
    );
    assert_eq!(
        scene_3d.task_id, request_task_id,
        "PreviewScene3D should have correct task_id"
    );

    // Wait for scene to load
    let mut iterations = 0;
    let mut scene_loaded = false;
    while !scene_loaded && iterations < 3000 {
        app.update();
        iterations += 1;
        let world = app.world();
        let scene_assets = world.resource::<Assets<bevy::scene::Scene>>();
        if scene_assets.get(&request_handle).is_some() {
            scene_loaded = true;
        }
    }

    // If scene is loaded, verify preview_entity is set
    if scene_loaded {
        let world = app.world();
        let scene_3d = world
            .entity(request_entity)
            .get::<PreviewScene3D>()
            .unwrap();
        assert!(
            scene_3d.preview_entity.is_some(),
            "PreviewScene3D should have preview_entity set after scene loads"
        );
    }

    // Wait for WaitingForScreenshot component to be added (indicates scene is ready for screenshot)
    let mut iterations = 0;
    let mut waiting_for_screenshot = false;
    while !waiting_for_screenshot && iterations < 1000 {
        app.update();
        iterations += 1;
        let world = app.world();
        if world
            .entity(request_entity)
            .contains::<WaitingForScreenshot>()
        {
            waiting_for_screenshot = true;
        }
    }
    assert!(
        waiting_for_screenshot,
        "WaitingForScreenshot should be added when scene is ready"
    );

    // Wait for Screenshot component to be added to camera (capture_preview_screenshot system)
    let world = app.world();
    let scene_3d = world
        .entity(request_entity)
        .get::<PreviewScene3D>()
        .unwrap();
    let camera_entity = scene_3d.camera_entity;

    iterations = 0;
    let mut screenshot_added = false;
    while !screenshot_added && iterations < 100 {
        app.update();
        iterations += 1;
        let world = app.world();
        if world.entity(camera_entity).contains::<Screenshot>() {
            screenshot_added = true;
        }
    }
    assert!(
        screenshot_added,
        "Screenshot component should be added to camera entity"
    );

    // Wait for rendering to complete and preview to be cached
    // Render pipeline needs several frames to process the screenshot
    let asset_path: AssetPath<'static> = AssetPath::from(filename).into_owned();
    let config = app.world().resource::<PreviewConfig>();
    let preview_resolution = config.resolutions.iter().max().copied().unwrap_or(256);

    // Give render pipeline time to process screenshot and cache preview
    // ScreenshotCaptured is an EntityEvent, so we wait for the cache to be populated
    let preview_cached =
        wait_for_preview_cached(&mut app, &asset_path, Some(preview_resolution), 2000);

    // Verify final state
    let world = app.world();
    let cache = world.resource::<PreviewCache>();
    let config = world.resource::<PreviewConfig>();
    let preview_resolution = config.resolutions.iter().max().copied().unwrap_or(256);

    // With render pipeline enabled, preview should be cached
    assert!(
        preview_cached,
        "Preview should be cached after rendering with render pipeline enabled"
    );

    // Verify preview was actually cached
    let cache_entry = cache.get_by_path(&asset_path, Some(preview_resolution));
    assert!(
        cache_entry.is_some(),
        "Preview should be in cache after rendering"
    );

    // Verify the cached image is valid
    let cache_entry = cache_entry.unwrap();
    let images = world.resource::<Assets<Image>>();
    let cached_image = images.get(&cache_entry.image_handle);
    assert!(
        cached_image.is_some(),
        "Cached preview image should be a valid asset"
    );

    // Verify original entity's ImageNode references the cached preview
    let image_node = world.entity(entity).get::<ImageNode>();
    assert!(image_node.is_some(), "Entity should have ImageNode");
    let image_node = image_node.unwrap();

    // ImageNode should reference the cached preview image (not placeholder)
    let image_node_image = images.get(&image_node.image);
    assert!(
        image_node_image.is_some(),
        "ImageNode should reference a valid image"
    );

    // Verify ImageNode references the cached preview (same handle or same image data)
    // The image should not be the placeholder
    let placeholder_path =
        bevy::asset::AssetPath::from("embedded://bevy_asset_browser/assets/file_icon.png");
    let placeholder_handle: Handle<Image> = world
        .resource::<bevy::asset::AssetServer>()
        .load(placeholder_path);

    // ImageNode should reference the preview, not the placeholder
    assert_ne!(
        image_node.image.id(),
        placeholder_handle.id(),
        "ImageNode should reference preview image, not placeholder"
    );

    // Verify request entity was cleaned up after preview generation
    assert!(
        !world
            .entity(request_entity)
            .contains::<PendingPreviewRequest>(),
        "PendingPreviewRequest should be removed after preview generation"
    );
    assert!(
        !world
            .entity(request_entity)
            .contains::<WaitingForScreenshot>(),
        "WaitingForScreenshot should be removed after screenshot is captured"
    );

    // Verify PreviewScene3D still exists with all required components
    assert!(
        world.entity(request_entity).contains::<PreviewScene3D>(),
        "PreviewScene3D should still exist"
    );
    let final_scene_3d = world
        .entity(request_entity)
        .get::<PreviewScene3D>()
        .unwrap();
    assert!(
        final_scene_3d.preview_entity.is_some(),
        "PreviewScene3D should have preview_entity after scene loads"
    );
    // Verify camera entity still exists (by checking it has components)
    assert!(
        world
            .entity(final_scene_3d.camera_entity)
            .contains::<bevy::camera::Camera3d>(),
        "Camera entity should still exist with Camera3d component"
    );

    // Verify the preview was successfully generated and cached
    // This completes the full rendering pipeline test
}

#[test]
fn test_preview_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let assets_dir = temp_dir.path().join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    unsafe {
        std::env::set_var(
            "BEVY_ASSET_ROOT",
            temp_dir
                .path()
                .to_str()
                .expect("Failed to convert path to string"),
        );
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: assets_dir.display().to_string(),
            ..Default::default()
        })
        .add_plugins(bevy_asset_preview::AssetPreviewPlugin)
        .init_asset::<Image>()
        .init_asset::<bevy::mesh::Mesh>()
        .init_asset::<bevy::pbr::StandardMaterial>()
        .init_asset::<bevy::scene::Scene>()
        .register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE));

    app.finish();
    app.update();

    // Request preview for non-existent file
    let entity = app
        .world_mut()
        .spawn(PreviewAsset(PathBuf::from("non_existent.png")))
        .id();

    app.update();

    // Verify initial state
    let world = app.world();
    assert!(
        world.entity(entity).contains::<ImageNode>(),
        "Should have ImageNode placeholder for error case"
    );
    assert!(
        world.entity(entity).contains::<PendingPreviewLoad>(),
        "Should have PendingPreviewLoad for non-existent file"
    );

    // Wait for failure to be handled
    let mut iterations = 0;
    let mut component_removed = false;
    while !component_removed && iterations < 2000 {
        app.update();
        iterations += 1;
        let world = app.world();
        if !world.entity(entity).contains::<PendingPreviewLoad>() {
            component_removed = true;
        }
    }
    assert!(
        component_removed,
        "PendingPreviewLoad should be removed after error handling"
    );

    // Verify cleanup after error
    let world = app.world();
    assert!(
        !world.entity(entity).contains::<PendingPreviewLoad>(),
        "PendingPreviewLoad should be cleaned up after error"
    );
    assert!(
        !world.entity(entity).contains::<PreviewAsset>(),
        "PreviewAsset should be removed after error"
    );
    assert!(
        world.entity(entity).contains::<ImageNode>(),
        "ImageNode should remain after error (placeholder)"
    );

    // Verify ImageNode still references placeholder
    let image_node = world.entity(entity).get::<ImageNode>();
    assert!(
        image_node.is_some(),
        "Entity should have ImageNode after error"
    );

    // Verify entity still exists (by checking it has components)
    assert!(
        world.entity(entity).contains::<ImageNode>(),
        "Entity should still exist with ImageNode after error handling"
    );

    // Verify no cache entry was created for non-existent file
    let cache = world.resource::<PreviewCache>();
    let asset_path: AssetPath<'static> = AssetPath::from("non_existent.png").into_owned();
    assert!(
        cache.get_by_path(&asset_path, None).is_none(),
        "No cache entry should be created for non-existent file"
    );
}

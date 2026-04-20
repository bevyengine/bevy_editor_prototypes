mod common;

use std::fs;
use std::path::PathBuf;

use bevy::{
    asset::{AssetPath, AssetPlugin, io::file::FileAssetWriter},
    image::{CompressedImageFormats, ImageLoader},
    prelude::*,
};
use bevy_asset_preview::{
    ActiveLoadTask, ActiveSaveTask, AssetError, AssetHotReloaded, AssetLoadCompleted,
    AssetLoadFailed, AssetLoader, LoadPriority, SaveCompleted, SaveTaskTracker,
    handle_asset_events, monitor_save_completion, process_load_queue, save_image,
};
use tempfile::TempDir;

use common::{create_test_image, save_test_image, wait_for_load_completion};

#[test]
fn test_asset_loading_workflow() {
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
        .init_asset::<Image>()
        .register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE))
        .init_resource::<AssetLoader>()
        .add_event::<AssetLoadCompleted>()
        .add_event::<AssetLoadFailed>()
        .add_event::<AssetHotReloaded>()
        .add_systems(Update, (process_load_queue, handle_asset_events));

    app.finish();
    app.update();

    // Create test image files
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let mut image_files = Vec::new();
    for i in 0..3 {
        let filename = format!("test_image_{}.png", i);
        let handle = create_test_image(&mut images, 64, 64, [255, 0, 0, 255]);
        let image = images.get(&handle).unwrap();
        let image_path = assets_dir.join(&filename);
        save_test_image(image, &image_path).expect("Failed to save test image");
        image_files.push((filename, handle));
    }
    drop(images);

    // Submit multiple loading tasks
    let mut loader = app.world_mut().resource_mut::<AssetLoader>();
    let mut task_ids = Vec::new();
    for (filename, _) in &image_files {
        let task_id = loader.submit(filename, LoadPriority::Preload);
        task_ids.push(task_id);
    }
    assert_eq!(loader.queue_len(), 3);
    drop(loader);

    // Process queue - should start up to 4 concurrent tasks
    app.update();

    let world = app.world();
    let loader = world.resource::<AssetLoader>();
    assert_eq!(loader.active_tasks(), 3, "Should have 3 active tasks");
    assert_eq!(
        loader.queue_len(),
        0,
        "Queue should be empty after processing"
    );

    // Verify ActiveLoadTask entities have been created
    let mut query = app.world_mut().query::<&ActiveLoadTask>();
    let world = app.world();
    let task_count = query.iter(world).count();
    assert_eq!(task_count, 3, "Should have 3 ActiveLoadTask entities");

    // Wait for all loads to complete
    let completed_events = wait_for_load_completion(&mut app, 3, 3000);
    assert_eq!(completed_events.len(), 3);

    // Verify all tasks have been cleaned up
    let world = app.world();
    let loader = world.resource::<AssetLoader>();
    assert_eq!(
        loader.active_tasks(),
        0,
        "All active tasks should be cleaned up"
    );
    assert_eq!(loader.queue_len(), 0, "Queue should be empty");

    // Verify all ActiveLoadTask entities have been cleaned up
    let mut query = app.world_mut().query::<&ActiveLoadTask>();
    let world = app.world();
    let task_count = query.iter(world).count();
    assert_eq!(
        task_count, 0,
        "All ActiveLoadTask entities should be despawned"
    );

    // Verify all task paths have been cleaned up
    let loader = app.world().resource::<AssetLoader>();
    for task_id in task_ids {
        assert_eq!(
            loader.get_task_path(task_id),
            None,
            "Task path {} should be cleaned up",
            task_id
        );
    }

    // Test error handling: load non-existent file
    let mut loader = app.world_mut().resource_mut::<AssetLoader>();
    let error_task_id = loader.submit("non_existent.png", LoadPriority::CurrentAccess);
    drop(loader);

    app.update();

    let world = app.world();
    let loader = world.resource::<AssetLoader>();
    assert_eq!(
        loader.active_tasks(),
        1,
        "Should have 1 active task for error case"
    );

    // Wait for failure event
    let mut failed_count = 0;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 2000;

    while failed_count == 0 && iterations < MAX_ITERATIONS {
        app.update();
        iterations += 1;

        let world = app.world();
        let failed_events = world.resource::<Events<AssetLoadFailed>>();
        let mut cursor = failed_events.get_cursor();
        for event in cursor.read(failed_events) {
            if event.task_id == error_task_id {
                failed_count += 1;
                match &event.error {
                    AssetError::AssetRemoved { path } => {
                        assert_eq!(path.to_string(), "non_existent.png");
                    }
                    AssetError::AssetLoadFailed { path, .. } => {
                        assert_eq!(path.to_string(), "non_existent.png");
                    }
                    _ => panic!("Unexpected error type: {:?}", event.error),
                }
            }
        }
    }

    assert!(failed_count > 0, "Should receive AssetLoadFailed event");

    // Verify cleanup in error case
    let world = app.world();
    let loader = world.resource::<AssetLoader>();
    assert_eq!(
        loader.active_tasks(),
        0,
        "Active tasks should be cleaned up after error"
    );
    assert_eq!(
        loader.get_task_path(error_task_id),
        None,
        "Task path should be cleaned up after error"
    );

    // Verify ActiveLoadTask entity has been cleaned up
    let mut query = app.world_mut().query::<&ActiveLoadTask>();
    let world = app.world();
    let task_count = query.iter(world).count();
    assert_eq!(
        task_count, 0,
        "ActiveLoadTask entity should be despawned after error"
    );
}

#[test]
fn test_asset_saving_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let assets_dir = temp_dir.path().join("assets");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

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
        .init_asset::<Image>()
        .register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE))
        .init_resource::<SaveTaskTracker>()
        .add_event::<SaveCompleted>()
        .add_systems(Update, monitor_save_completion);

    app.finish();
    app.update();

    // Create test images
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let mut image_files = Vec::new();
    for i in 0..2 {
        let filename = format!("save_test_{}.png", i);
        let handle = create_test_image(&mut images, 64, 64, [255, 0, 0, 255]);
        image_files.push((filename, handle));
    }
    drop(images);

    // Create save tasks
    let save_tasks = app
        .world_mut()
        .resource_scope(|world, mut tracker: Mut<SaveTaskTracker>| {
            let images = world.get_resource::<Assets<Image>>().unwrap();
            let mut tasks = Vec::new();

            for (filename, handle) in &image_files {
                let writer = FileAssetWriter::new("", true);
                let target_path =
                    AssetPath::from_path_buf(PathBuf::from("cache/asset_preview").join(filename))
                        .into_owned();
                let task = save_image(handle.clone(), target_path.clone(), images, writer);
                let task_id = tracker.create_task_id();
                let path_asset: AssetPath<'static> =
                    AssetPath::from_path_buf(PathBuf::from(filename)).into_owned();
                tracker.register_pending(task_id, path_asset.clone());
                tasks.push((task_id, path_asset, target_path, task));
            }

            tasks
        });

    // Verify tracker state
    let tracker = app.world().resource::<SaveTaskTracker>();
    assert_eq!(tracker.pending_count(), 2, "Should have 2 pending saves");

    // Spawn ActiveSaveTask entities
    let mut commands = app.world_mut().commands();
    for (task_id, path, target_path, task) in save_tasks {
        commands.spawn(ActiveSaveTask {
            task_id,
            path,
            target_path,
            task,
        });
    }
    drop(commands);

    // Apply commands
    app.update();

    // Verify ActiveSaveTask entities have been created
    let mut query = app.world_mut().query::<&ActiveSaveTask>();
    let world = app.world();
    let task_count = query.iter(world).count();
    assert_eq!(task_count, 2, "Should have 2 ActiveSaveTask entities");

    // Wait for save completion
    let mut save_completed_count = 0;
    let mut processed_task_ids = std::collections::HashSet::new();
    let mut completed_events = Vec::new();
    let mut iterations = 0;
    const MAX_SAVE_ITERATIONS: usize = 1000;

    while save_completed_count < 2 && iterations < MAX_SAVE_ITERATIONS {
        app.update();
        iterations += 1;

        let world = app.world();
        let save_events = world.resource::<Events<SaveCompleted>>();
        let mut cursor = save_events.get_cursor();
        for event in cursor.read(save_events) {
            if processed_task_ids.insert(event.task_id) {
                match &event.result {
                    Ok(_) => {
                        save_completed_count += 1;
                        completed_events.push(event.clone());
                    }
                    Err(e) => panic!(
                        "Save task {} failed after {} iterations: {}",
                        event.task_id, iterations, e
                    ),
                }
            }
        }
    }

    assert_eq!(
        save_completed_count, 2,
        "Expected 2 save completions, got {} after {} iterations",
        save_completed_count, iterations
    );
    assert_eq!(completed_events.len(), 2);
    for event in &completed_events {
        assert!(event.result.is_ok(), "Save should succeed");
    }

    // Verify ActiveSaveTask entities have been cleaned up
    let mut query = app.world_mut().query::<&ActiveSaveTask>();
    let world = app.world();
    let task_count = query.iter(world).count();
    assert_eq!(task_count, 0, "ActiveSaveTask entities should be despawned");

    // Verify tracker cleanup
    let tracker = world.resource::<SaveTaskTracker>();
    assert_eq!(
        tracker.pending_count(),
        0,
        "Pending saves should be cleaned up"
    );

    // Test error handling: save non-existent handle
    let non_existent_handle = Handle::<Image>::default();
    let (error_task_id, error_path, error_target_path, error_task) = app
        .world_mut()
        .resource_scope(|world, mut tracker: Mut<SaveTaskTracker>| {
            let images = world.get_resource::<Assets<Image>>().unwrap();
            let writer = FileAssetWriter::new("", true);
            let target_path = AssetPath::from_path_buf(
                PathBuf::from("cache/asset_preview").join("non_existent.png"),
            )
            .into_owned();
            let task = save_image(
                non_existent_handle.clone(),
                target_path.clone(),
                images,
                writer,
            );
            let task_id = tracker.create_task_id();
            let path: AssetPath<'static> = AssetPath::from("non_existent.png").into_owned();
            tracker.register_pending(task_id, path.clone());
            (task_id, path, target_path, task)
        });

    app.world_mut().commands().spawn(ActiveSaveTask {
        task_id: error_task_id,
        path: error_path,
        target_path: error_target_path,
        task: error_task,
    });

    // Wait for save failure event
    let mut save_completed_count = 0;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 1000;

    while save_completed_count == 0 && iterations < MAX_ITERATIONS {
        app.update();
        iterations += 1;

        let world = app.world();
        let save_events = world.resource::<Events<SaveCompleted>>();
        let mut cursor = save_events.get_cursor();
        for event in cursor.read(save_events) {
            if event.task_id == error_task_id {
                save_completed_count += 1;
                assert!(
                    event.result.is_err(),
                    "Save should fail for non-existent handle"
                );
                match &event.result {
                    Err(AssetError::ImageNotFound { handle: _ }) => {
                        // Expected error
                    }
                    Err(e) => panic!("Unexpected error type: {:?}", e),
                    Ok(_) => panic!("Save should have failed"),
                }
            }
        }
    }

    assert!(
        save_completed_count > 0,
        "Should receive SaveCompleted event with error"
    );

    // Verify cleanup in error case
    let mut query = app.world_mut().query::<&ActiveSaveTask>();
    let world = app.world();
    let task_count = query.iter(world).count();
    assert_eq!(
        task_count, 0,
        "ActiveSaveTask entity should be despawned after error"
    );

    let tracker = world.resource::<SaveTaskTracker>();
    assert_eq!(
        tracker.pending_count(),
        0,
        "Pending saves should be cleaned up after error"
    );
}
